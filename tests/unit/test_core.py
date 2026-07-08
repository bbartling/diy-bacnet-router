"""Unit tests (no live bench)."""

import pytest

from app.auth import auth_path_exempt
from app.config import load_field_devices, load_objects_csv
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
    assert "OA-WEATHER-T" in names
    assert "openfdd-active-fault-count" in names


def test_load_field_devices():
    devs = load_field_devices()
    inst = {d.device_instance for d in devs}
    assert 5007 in inst
    assert 3456789 in inst


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
