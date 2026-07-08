"""Unit tests (no live bench)."""

import re

import pytest

from app.auth import auth_path_exempt
from app.bacnet_server import BacnetServerManager
from app.config import load_field_devices, load_objects_csv, load_settings
from app.haystack_client import HaystackClientService, HaystackNotAllowedError
from app.modbus_client import _decode_words, _apply_scale_offset, ModbusServiceError
from app.weather import dewpoint_f_from_db_rh


def test_auth_exempt_paths():
    assert auth_path_exempt("/health")
    assert auth_path_exempt("/docs")
    assert not auth_path_exempt("/bacnet/read")


def test_load_objects_csv():
    rows = load_objects_csv()
    names = {r.name for r in rows}
    assert "outside-air-temperature" in names
    assert "openfdd-active-fault-count" in names
    assert "openfdd-optimization-enabled" in names


def test_point_names_lowercase_hyphenated():
    """All hosted point names use the same lowercase-hyphenated caps convention."""
    pat = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")
    for row in load_objects_csv():
        assert pat.match(row.name), f"{row.name!r} is not lowercase-hyphenated"


def test_optimization_point_is_commandable():
    rows = {r.name: r for r in load_objects_csv()}
    assert rows["openfdd-optimization-enabled"].commandable is True


def test_weather_and_fault_points_are_server_owned():
    rows = {r.name: r for r in load_objects_csv()}
    for name in (
        "outside-air-temperature",
        "outside-air-humidity",
        "outside-air-wind-speed",
        "outside-air-dewpoint",
        "weather-location",
        "app-fault",
        "openfdd-active-fault-count",
        "openfdd-faults-present",
    ):
        assert rows[name].commandable is False, name


def test_api_writable_split():
    """Server-owned points are API-writable; commandable points are not."""
    rows = {r.name: r for r in load_objects_csv()}
    assert BacnetServerManager.api_writable(rows["outside-air-temperature"]) is True
    assert BacnetServerManager.api_writable(rows["openfdd-active-fault-count"]) is True
    assert BacnetServerManager.api_writable(rows["openfdd-optimization-enabled"]) is False


async def test_update_rejects_commandable_point():
    """A REST update of a commandable (BACnet-writable) point is rejected."""
    mgr = BacnetServerManager(load_settings())
    result = await mgr.update_points({"openfdd-optimization-enabled": True})
    assert "rejected" in result["openfdd-optimization-enabled"]


async def test_update_unknown_point_not_found():
    mgr = BacnetServerManager(load_settings())
    result = await mgr.update_points({"does-not-exist": 1})
    assert result["does-not-exist"] == "not found"


async def test_update_mixed_rejects_only_commandable():
    """Commandable rejected without ever touching the (unstarted) server."""
    mgr = BacnetServerManager(load_settings())
    result = await mgr.update_points(
        {"openfdd-optimization-enabled": True, "missing-point": 5}
    )
    assert "rejected" in result["openfdd-optimization-enabled"]
    assert result["missing-point"] == "not found"


def test_dewpoint_magnus():
    dp = dewpoint_f_from_db_rh(70.0, 50.0)
    assert 45.0 < dp < 55.0


def test_modbus_decode_uint16():
    assert _decode_words([0x00FF], "uint16") == 255


def test_modbus_decode_float32():
    import struct

    packed = struct.pack(">f", 71.27)
    hi, lo = struct.unpack(">HH", packed)
    val = _decode_words([hi, lo], "float32")
    assert abs(val - 71.27) < 0.01


def test_modbus_scale_offset():
    assert _apply_scale_offset(10, 0.1, 1.0) == 2.0


def test_haystack_readonly_blocks_write():
    svc = HaystackClientService("http://127.0.0.1:1", "u", "p")
    with pytest.raises(HaystackNotAllowedError):
        svc._check_op("pointWrite")
