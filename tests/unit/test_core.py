"""Unit tests (no live bench)."""

import re

import pytest

from app.auth import auth_path_exempt
from app.bacnet_server import BacnetServerManager
from app.config import git_sha, load_field_devices, load_objects_csv, load_settings
from app.haystack_client import HaystackClientService, HaystackNotAllowedError
from app.models import BacnetWriteRequest
from app.modbus_client import _decode_words, _apply_scale_offset, ModbusServiceError
from app.poll import PollEngine
from app.weather import WeatherService, dewpoint_f_from_db_rh


def test_auth_exempt_paths():
    assert auth_path_exempt("/health")
    assert auth_path_exempt("/api/health")
    assert auth_path_exempt("/docs")
    assert not auth_path_exempt("/bacnet/read")
    assert not auth_path_exempt("/api/bacnet/read")


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


# ---- Open-FDD tailoring: object map / env aliases / poll / write gate ----


def test_weather_instances_match_openfdd():
    """Hosted weather points use Open-FDD's AV instance map (9101-9103)."""
    rows = {r.name: r for r in load_objects_csv()}
    assert rows["outside-air-temperature"].instance == 9101
    assert rows["outside-air-humidity"].instance == 9102
    assert rows["outside-air-dewpoint"].instance == 9103
    # weather.py constants must agree with the CSV.
    assert WeatherService.AV_TEMP == 9101
    assert WeatherService.AV_RH == 9102
    assert WeatherService.AV_DP == 9103


def test_env_alias_precedence(monkeypatch):
    """OPENFDD_FIELDBUS_* wins; RUSTY_GATEWAY_* remains a fallback."""
    monkeypatch.setenv("RUSTY_GATEWAY_HTTP_PORT", "8080")
    monkeypatch.setenv("OPENFDD_FIELDBUS_HTTP_PORT", "9091")
    assert load_settings().http_port == 9091
    monkeypatch.delenv("OPENFDD_FIELDBUS_HTTP_PORT")
    assert load_settings().http_port == 8080


def test_env_poll_toggle(monkeypatch):
    monkeypatch.setenv("OPENFDD_FIELDBUS_POLL_ENABLED", "false")
    assert load_settings().poll.enabled is False
    monkeypatch.setenv("OPENFDD_FIELDBUS_POLL_INTERVAL_SECS", "12.5")
    assert load_settings().poll.interval_secs == 12.5


def test_git_sha_env(monkeypatch):
    monkeypatch.setenv("OPENFDD_FIELDBUS_GIT_SHA", "deadbeef")
    assert git_sha() == "deadbeef"
    monkeypatch.delenv("OPENFDD_FIELDBUS_GIT_SHA")
    monkeypatch.delenv("GIT_SHA", raising=False)
    assert git_sha() == "unknown"


def test_write_request_release_requires_priority():
    with pytest.raises(ValueError):
        BacnetWriteRequest(device_instance=5007, object_type="analog-output", object_instance=2466, value=None)
    # With a priority it validates.
    req = BacnetWriteRequest(
        device_instance=5007, object_type="analog-output", object_instance=2466, value=None, priority=8
    )
    assert req.priority == 8
    assert req.approved is True  # writes are approved by default


def test_write_request_approved_gate_default_true():
    req = BacnetWriteRequest(device_instance=5007, object_type="analog-output", object_instance=2466, value=55.0)
    assert req.approved is True
    req2 = BacnetWriteRequest(
        device_instance=5007, object_type="analog-output", object_instance=2466, value=55.0, approved=False
    )
    assert req2.approved is False


class _FakeClient:
    """Stand-in for BacnetClientService with a canned poll result."""

    def __init__(self, rows):
        self._rows = rows

    async def poll_points(self):
        return list(self._rows)


async def test_poll_engine_buffers_and_status():
    rows = [
        {"device_instance": 5007, "object_type": "analog-input", "object_instance": 1173,
         "point_name": "OA-T", "value": 71.5, "error": None, "ts": 1.0},
        {"device_instance": 5007, "object_type": "analog-output", "object_instance": 2466,
         "point_name": "ACTUATOR-0", "value": 55.0, "error": None, "ts": 1.0},
    ]
    engine = PollEngine(load_settings(), _FakeClient(rows))

    # Before any cycle: nothing tracked, loop not running.
    s0 = engine.status()
    assert s0["running"] is False
    assert s0["points_tracked"] == 0

    summary = await engine.poll_once()
    assert summary["points_polled"] == 2
    assert summary["points_errored"] == 0

    s1 = engine.status()
    assert s1["cycles_completed"] == 1
    assert s1["points_tracked"] == 2
    assert s1["points_healthy"] == 2
    assert s1["samples_buffered"] == 2


async def test_poll_engine_counts_errors():
    rows = [
        {"device_instance": 5007, "object_type": "analog-input", "object_instance": 1,
         "point_name": "x", "value": None, "error": "timeout", "ts": 1.0},
    ]
    engine = PollEngine(load_settings(), _FakeClient(rows))
    summary = await engine.poll_once()
    assert summary["points_errored"] == 1
    assert engine.status()["points_healthy"] == 0
