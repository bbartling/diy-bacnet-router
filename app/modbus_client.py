"""Modbus TCP client via rusty_modbus (PyO3)."""

from __future__ import annotations

import struct
from typing import Any, Literal, Optional

MAX_REGS_PER_OPERATION = 125
MAX_OPERATIONS_PER_REQUEST = 32

DecodeKind = Optional[Literal["raw", "uint16", "int16", "uint32", "int32", "float32"]]


class ModbusServiceError(ValueError):
    pass


def _require_rusty_modbus():
    try:
        import rusty_modbus  # noqa: F401
    except ImportError as e:
        raise ModbusServiceError(
            "rusty_modbus not installed (requires Python 3.14+ wheel). "
            "Use Docker image or: maturin develop in rusty-modbus-python."
        ) from e


def _decode_words(words: list[int], decode: DecodeKind) -> Any:
    if decode is None or decode == "raw":
        return None
    if not words:
        raise ModbusServiceError("No register words to decode")
    if decode == "uint16":
        return int(words[0]) & 0xFFFF
    if decode == "int16":
        return struct.unpack(">h", struct.pack(">H", int(words[0]) & 0xFFFF))[0]
    if decode in ("uint32", "int32", "float32"):
        if len(words) < 2:
            raise ModbusServiceError(f"{decode} needs count >= 2")
        hi, lo = int(words[0]) & 0xFFFF, int(words[1]) & 0xFFFF
        packed = struct.pack(">HH", hi, lo)
        if decode == "uint32":
            return struct.unpack(">I", packed)[0]
        if decode == "int32":
            return struct.unpack(">i", packed)[0]
        return struct.unpack(">f", packed)[0]
    raise ModbusServiceError(f"Unknown decode: {decode}")


def _apply_scale_offset(value: Any, scale: Optional[float], offset: Optional[float]) -> Any:
    if value is None:
        return None
    if not isinstance(value, (int, float)):
        return value
    out = float(value)
    if scale is not None:
        out *= scale
    if offset is not None:
        out += offset
    if isinstance(value, int) and scale is None and offset is None:
        return value
    return out


async def execute_modbus_read(payload: dict[str, Any]) -> dict[str, Any]:
    """Run Modbus TCP reads using rusty_modbus async client."""
    _require_rusty_modbus()
    import rusty_modbus

    host = payload["host"]
    port = int(payload["port"])
    unit_id = int(payload["unit_id"])
    registers = payload["registers"]

    if len(registers) > MAX_OPERATIONS_PER_REQUEST:
        raise ModbusServiceError(f"At most {MAX_OPERATIONS_PER_REQUEST} operations per request")

    endpoint = f"{host}:{port}"
    client = await rusty_modbus.ModbusClient.connect(endpoint)
    readings: list[dict[str, Any]] = []

    try:
        for spec in registers:
            address = int(spec["address"])
            count = int(spec["count"])
            fn = spec["function"]
            decode: DecodeKind = spec.get("decode")
            scale = spec.get("scale")
            offset = spec.get("offset")
            label = spec.get("label")

            if count < 1 or count > MAX_REGS_PER_OPERATION:
                raise ModbusServiceError(f"count must be 1..{MAX_REGS_PER_OPERATION}")

            try:
                if fn == "holding":
                    words = await client.read_holding_registers(unit_id, address, count)
                elif fn == "input":
                    words = await client.read_input_registers(unit_id, address, count)
                else:
                    raise ModbusServiceError(f"Invalid function: {fn}")
                words_list = [int(w) & 0xFFFF for w in words]
                decoded = _decode_words(words_list, decode)
                decoded = _apply_scale_offset(decoded, scale, offset)
                readings.append(
                    {
                        "address": address,
                        "function": fn,
                        "count": count,
                        "success": True,
                        "words": words_list,
                        "decoded": decoded,
                        "label": label,
                        "error": None,
                    }
                )
            except Exception as e:
                readings.append(
                    {
                        "address": address,
                        "function": fn,
                        "count": count,
                        "success": False,
                        "words": None,
                        "decoded": None,
                        "label": label,
                        "error": str(e),
                    }
                )
    finally:
        await client.shutdown()

    return {
        "ok": True,
        "host": host,
        "port": port,
        "unit_id": unit_id,
        "timeout": payload.get("timeout"),
        "readings": readings,
    }
