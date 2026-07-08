---
title: Modbus TCP
nav_order: 8
---

# Modbus TCP (REST)

The gateway exposes a batched Modbus TCP read endpoint backed by **rusty-modbus**:

- **`POST /modbus/read`**

## Request body (`ModbusReadRequest`)

| Field | Type | Notes |
|-------|------|--------|
| `host` | string | Modbus TCP host. |
| `port` | int | Default `502`. |
| `unit_id` | int | Unit / slave id `0–255`. |
| `timeout` | float | Seconds. |
| `registers` | array | **1–32** operations per HTTP request. |

Each register operation:

| Field | Notes |
|-------|--------|
| `address` | `0–65535` (wire vs documented addressing is device-specific). |
| `count` | `1–125` words. |
| `function` | `"holding"` (FC03) or `"input"` (FC04). |
| `decode` | Optional: `raw`, `uint16`, `int16`, `uint32`, `int32`, `float32` (32-bit types need `count >= 2`; big-endian, first word = high 16 bits). |
| `scale` / `offset` | Applied after decode. |
| `label` | Echoed in the response. |

## Authentication

When **`RUSTY_GATEWAY_API_KEY`** is set, send `Authorization: Bearer <key>`
(see [Environment](environment)). `/health` and `/docs` are exempt.

## Example

```bash
curl -sS -X POST "http://localhost:8080/modbus/read" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $RUSTY_GATEWAY_API_KEY" \
  -d '{
    "host": "10.200.200.170",
    "port": 502,
    "unit_id": 1,
    "timeout": 5.0,
    "registers": [
      {"address": 184, "count": 1, "function": "holding", "decode": "uint16", "label": "Battery SoC %"},
      {"address": 500, "count": 2, "function": "input", "decode": "float32", "label": "Example L1-N V"}
    ]
  }'
```
