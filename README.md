# DIY BACnet Server

A turnkey **FastAPI + Swagger** building-automation gateway where **Python is only
the web layer** and every protocol stack is **Rust via PyO3**:

- **rusty-bacnet** — hosts a BACnet server device (**599999**) on UDP `:47808`
  with Open-Meteo weather objects (20-min refresh) + diagnostic points, **and**
  provides the full BACnet client toolkit (read, write, RPM, Who-Is, discovery,
  priority-array, supervisory audit).
- **rusty-modbus** — Modbus TCP read API.
- **rusty-haystack** — read-only Haystack client (SCRAM).

Everything is exposed as a clean REST surface with a Swagger UI and optional
Bearer-token auth.

## Design

The hosted server and the client use **separate sockets**. The server binds
`0.0.0.0:47808` so it receives broadcast Who-Is from BMS discovery tools, while
the client sends Who-Is on `:47808` and performs unicast reads on ephemeral
ports. This keeps discovery reliable and avoids two consumers contending for the
same UDP socket.

## Quick start (local dev, Python 3.12+)

```bash
cd diy-bacnet-server
python3 -m venv .venv && . .venv/bin/activate
pip install -e . rusty-bacnet rusty-haystack
chmod +x scripts/*.sh
./scripts/run_dev.sh
# Swagger: http://127.0.0.1:8080/docs
```

`scripts/preflight_free_47808.sh` frees UDP `:47808` before the server binds.

> `rusty-modbus` currently ships a Python 3.14 wheel. On 3.12 the app boots and
> all BACnet / weather / Haystack routes work; `/modbus/*` returns a clear error
> until the wheel (or the Docker image) is available.

## Docker (Python 3.14 + all three Rust wheels)

Build from the parent directory (siblings `rusty-bacnet`, `rusty-haystack`):

```bash
cd ..
cp diy-bacnet-server/.env.example diy-bacnet-server/.env
docker compose -f diy-bacnet-server/docker-compose.yml up -d --build
```

## Services

| Service | What it does |
|---------|--------------|
| **BACnet server** | Hosts device 599999 on `:47808` with weather + diagnostic objects. |
| **BACnet client** | Read / write / RPM / Who-Is / discovery / priority-array / supervisory against field devices. |
| **Weather** | Polls Open-Meteo, caches it, and mirrors it into BACnet objects. |
| **Modbus** | Batched Modbus TCP register reads with decode / scale / offset. |
| **Haystack** | Read-only Haystack client (about / read / nav / hisRead). |

## API (Bearer `RUSTY_GATEWAY_API_KEY` when set)

**BACnet client (field bus)**

| Method | Path | Description |
|--------|------|-------------|
| POST | `/bacnet/read` | ReadProperty on a field device |
| POST | `/bacnet/write` | WriteProperty (priority + Null release) |
| POST | `/bacnet/rpm` | ReadPropertyMultiple |
| POST | `/bacnet/whois` | Who-Is range scan |
| POST | `/bacnet/whois-router` | Who-Is router-to-network (routed networks) |
| POST | `/bacnet/discover` | Point discovery (object-list + commandable scan) |
| POST | `/bacnet/priority-array` | Read a priority array (16 slots) |
| POST | `/bacnet/supervisory` | Supervisory override audit |
| GET | `/bacnet/points` | Configured field-device catalog |

**BACnet server (hosted device 599999)**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/bacnet/server/objects` | Read all hosted point values |
| GET | `/bacnet/server/commandable` | Read commandable hosted points |
| POST | `/bacnet/server/update` | Update hosted point present-values |

**Weather / Modbus / Haystack**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/weather` | Open-Meteo cache + BACnet mirror status |
| POST | `/weather/refresh` | Force a weather poll |
| POST | `/modbus/read` | Modbus TCP batch read |
| GET | `/haystack/about` | Haystack about |
| POST | `/haystack/read` | Haystack read (read-only) |
| POST | `/haystack/nav` | Haystack nav |
| POST | `/haystack/his-read` | Haystack hisRead |
| GET | `/health` | Liveness |

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
- `config/gateway.toml` — server / client bind + broadcast + timeouts.

The hosted server always binds `0.0.0.0:47808`; the client uses the NIC IP
(`RUSTY_GATEWAY_BIND`) with a derived directed broadcast. See [Environment](docs/environment.md)
for the full variable list.

## Security

Optional `RUSTY_GATEWAY_API_KEY` enables Bearer middleware. Send
`Authorization: Bearer <key>` on protected routes; `/`, `/health`, and the
Swagger docs stay reachable without a token, and Swagger's **Authorize** button
uses the same value.

## Tests

```bash
pip install -e ".[dev]" rusty-bacnet rusty-haystack
pytest tests/unit -q
pytest tests/integration -m integration -q   # needs live devices + :47808 free
```
