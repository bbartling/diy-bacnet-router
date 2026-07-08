# DIY BACnet Server (Rust)

A turnkey **FastAPI + Swagger** building-automation gateway where **Python is only
the web layer** and every protocol stack is **Rust via PyO3**. This is the
Rust-backed rewrite of the original bacpypes3 `diy-bacnet-server` — bacpypes3,
pyModbusTCP, and the MQTT/JSON-RPC layers have been **removed entirely** in favor
of a clean REST surface over:

- **rusty-bacnet** — hosts the BACnet server device (**599999**) on UDP `:47808`
  with Open-Meteo weather AVs (20-min refresh) + FDD diagnostic points, **and**
  provides the full BACnet client toolkit.
- **rusty-modbus** — Modbus TCP read API.
- **rusty-haystack** — read-only Haystack client (SCRAM).

## Why the rewrite

The legacy server ran its hosted device **and** its client on one shared
bacpypes3 `Application`. That is exactly what produces the Who-Is storms and the
"discovery returns 0 devices" failures seen in mixed poll/scan deployments (two
consumers fighting over UDP `:47808`). The Rust stack separates the hosted
server (bound to `0.0.0.0:47808` so it hears broadcast Who-Is) from client
operations (Who-Is on `:47808`, unicast reads on ephemeral ports), eliminating
the conflict.

## Quick start (local dev, Python 3.12+)

```bash
cd /home/ben/diy-bacnet-server
python3 -m venv .venv && . .venv/bin/activate
pip install -e . rusty-bacnet rusty-haystack
chmod +x scripts/*.sh
./scripts/run_dev.sh
# Swagger: http://127.0.0.1:8080/docs
```

`scripts/preflight_free_47808.sh` stops conflicting containers (e.g. Open-FDD)
and frees `:47808` before bind.

> `rusty-modbus` currently ships a Python 3.14 wheel. On 3.12 the app boots and
> all BACnet/weather/Haystack routes work; `/modbus/*` returns a clear error
> until the wheel (or the Docker image) is available.

## Docker (Python 3.14 + all three Rust wheels)

```bash
cd /home/ben
cp diy-bacnet-server/.env.example diy-bacnet-server/.env
docker compose -f diy-bacnet-server/docker-compose.yml build
docker compose -f diy-bacnet-server/docker-compose.yml up -d
```

## API (Bearer `RUSTY_GATEWAY_API_KEY` when set)

Full 1:1 parity with the original `client_utils.py`, on the Rust stack.

**BACnet client (field bus)**

| Method | Path | Legacy equivalent |
|--------|------|-------------------|
| POST | `/bacnet/read` | `bacnet_read` |
| POST | `/bacnet/write` | `bacnet_write` (priority + null release) |
| POST | `/bacnet/rpm` | `bacnet_rpm` |
| POST | `/bacnet/whois` | `perform_who_is` |
| POST | `/bacnet/whois-router` | `perform_who_is_router_to_network` |
| POST | `/bacnet/discover` | `point_discovery` (object-list + commandable scan) |
| POST | `/bacnet/priority-array` | `read_point_priority_arr` (16 slots) |
| POST | `/bacnet/supervisory` | `supervisory_logic_check` (override audit) |
| GET | `/bacnet/points` | configured field-device catalog |

**BACnet server (hosted device 599999)**

| Method | Path | Legacy equivalent |
|--------|------|-------------------|
| GET | `/bacnet/server/objects` | `server_read_all_values` |
| GET | `/bacnet/server/commandable` | `server_read_commandable` |
| POST | `/bacnet/server/update` | `server_update_points` |

**Weather / Modbus / Haystack**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/weather` | Open-Meteo cache + BACnet mirror status |
| POST | `/weather/refresh` | Force weather poll |
| POST | `/modbus/read` | Modbus TCP batch read (rusty_modbus) |
| GET | `/haystack/about` | Haystack about |
| POST | `/haystack/read` | Haystack read (read-only) |
| POST | `/haystack/nav` | Haystack nav |
| POST | `/haystack/his-read` | Haystack hisRead |
| GET | `/health` | Liveness |

> **Removed vs. legacy:** the MQTT command/ack gateway and JSON-RPC transport are
> gone (REST + Swagger only). Schedule read/update is not exposed — the current
> `rusty_bacnet` PyO3 surface has no weekly-`TimeValue` constructor, so it cannot
> be matched 1:1 yet.

### Write / release example

```bash
# Override analog-value 1 to 55.0 at priority 8
curl -X POST http://127.0.0.1:8080/bacnet/write -H "Content-Type: application/json" \
  -d '{"device_instance":3456790,"object_type":"analog-value","object_instance":1,"value":55.0,"priority":8}'

# Release that override (write Null @ priority 8)
curl -X POST http://127.0.0.1:8080/bacnet/write -H "Content-Type: application/json" \
  -d '{"device_instance":3456790,"object_type":"analog-value","object_instance":1,"value":null,"priority":8}'
```

## Configuration

- `config/objects.csv` — hosted server point catalog (Name, PointType, Units, Commandable, Default, Instance).
- `config/field_devices.toml` — client field devices + points.
- `config/gateway.toml` — server/client bind + broadcast + timeouts.

The hosted server always binds `0.0.0.0:47808`; the client uses the NIC IP
(`RUSTY_GATEWAY_BIND`) with a derived directed broadcast.

## Bench field devices (`config/field_devices.toml`)

- **5007** @ 192.168.204.200 (MSTP net 2000, via BASRT-B router) — AI:1173 OA-T
- **3456789** @ 192.168.204.13 — AI:2 SA-T
- **3456790** @ 192.168.204.14 — AI:1 ZoneTemp + commandable AV/AO

## Security

Optional `RUSTY_GATEWAY_API_KEY` enables Bearer middleware; Swagger `/docs`
stays reachable with **Authorize** for Try-it-out.

## Tests

```bash
pip install -e ".[dev]" rusty-bacnet rusty-haystack
pytest tests/unit -q
pytest tests/integration -m integration -q   # needs bench + :47808 free
```
