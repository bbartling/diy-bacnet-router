# DIY BACnet Server — Open-FDD field-bus sidecar

A turnkey **FastAPI + Swagger** field-bus sidecar for [Open-FDD](https://github.com/bbartling/open-fdd)
where **Python is only the web layer** and every protocol stack is **Rust via
PyO3**. It owns *all* field-bus I/O so the FDD app only ever speaks JSON:

- **rusty-bacnet** — hosts a BACnet server device (**599999**) on UDP `:47808`
  with Open-Meteo weather objects (20-min refresh) + diagnostic points, runs a
  **background poll engine**, **and** provides the full BACnet client toolkit
  (read, write, RPM, Who-Is, Who-Is-router-to-network, discovery, priority-array,
  supervisory override audit).
- **rusty-modbus** — Modbus TCP read API.
- **rusty-haystack** — read-only Haystack client (SCRAM).

Everything is exposed as a clean REST surface with a Swagger UI and optional
Bearer-token auth. Native routes live at the root (`/bacnet/*`, `/modbus/*`,
`/haystack/*`, `/weather`); the same operations are mirrored under **`/api/*`**
so an Open-FDD deployment can reach them with the `/api` prefix it uses
elsewhere.

## Open-FDD sidecar model

Open-FDD contends for UDP `:47808` when it embeds its own BACnet stack. Running
this sidecar lets Open-FDD delegate every network request — BACnet, Modbus,
Haystack — and consume JSON only. The sidecar:

- owns the field bus (poll loop + on-demand client ops) and the hosted server;
- aligns the hosted weather objects to Open-FDD's instance map
  (`outside-air-temperature/humidity/dewpoint` = AV `9101/9102/9103`);
- honors `OPENFDD_FIELDBUS_*` environment variables (with the original
  `RUSTY_GATEWAY_*` names as fallbacks);
- exposes `/api/health` with `git_sha`/service shape and a write-safety
  **dry-run / approval** gate for supervised writes.

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
| POST | `/bacnet/write` | WriteProperty (priority + Null release; `approved:false` ⇒ dry-run) |
| POST | `/bacnet/write-dry-run` | Validate + encode a write without touching the bus |
| POST | `/bacnet/rpm` | ReadPropertyMultiple |
| POST | `/bacnet/whois` | Who-Is range scan |
| POST | `/bacnet/whois-router` | Who-Is router-to-network (routed networks) |
| POST | `/bacnet/discover` | Point discovery (object-list + commandable scan) |
| POST | `/bacnet/priority-array` | Read a priority array (16 slots) |
| POST | `/bacnet/supervisory` | Supervisory override audit |
| GET | `/bacnet/poll/status` | Background poll engine status + last values |
| POST | `/bacnet/poll/once` | Run one poll cycle now (present-value, all points) |
| GET | `/bacnet/points` | Configured field-device catalog |

Every row above is also served under `/api/bacnet/*`. `/api/bacnet/point-discovery`
is the Open-FDD-named alias of `/bacnet/discover`, and `/api/health` mirrors
`/health` with the Open-FDD shape (`service`, `version`, `git_sha`, `poll_running`).

**BACnet server (hosted device 599999)**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/bacnet/server/objects` | Read all hosted points (`present_value`, `commandable`, `api_writable`) |
| GET | `/bacnet/server/commandable` | Read commandable (BACnet-writable) points and their current values |
| POST | `/bacnet/server/update` | Update **server-owned** points (commandable points are rejected) |

> **Read / write split (no data race).** Commandable points (`Commandable=Y`) are
> BACnet-writable — a field or supervisory device may command them, so the REST
> API is **read-only** for them and `/bacnet/server/update` rejects writes to
> them. Server-owned points (`Commandable=N`: weather, fault counts, status) are
> the only ones the API may write. Either way, current values are always visible
> via the read endpoints.

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
(`OPENFDD_FIELDBUS_BIND`, or the legacy `RUSTY_GATEWAY_BIND`) with a derived
directed broadcast. The poll engine is controlled by
`OPENFDD_FIELDBUS_POLL_ENABLED` / `OPENFDD_FIELDBUS_POLL_INTERVAL_SECS`. See
[Environment](docs/environment.md) for the full variable list.

## Smoke test

`scripts/smoke_test.sh` mirrors the Open-FDD nightly bench validation: it drives
the full BACnet client surface through REST (Who-Is, read, RPM, discovery,
priority-array, supervisory audit, write + Null release, dry-run), checks the
poll engine and hosted server, and asserts the live bench override on device
`5007` (`analogOutput:2466` = **55% at priority 8**).

```bash
OPENFDD_FIELDBUS_API_KEY=<key> SMOKE_BASE=http://127.0.0.1:8080 \
  scripts/smoke_test.sh
```

## Security

Optional `OPENFDD_FIELDBUS_API_KEY` (or the legacy `RUSTY_GATEWAY_API_KEY`)
enables Bearer middleware. Send `Authorization: Bearer <key>` on protected
routes; `/`, `/health`, `/api/health`, and the Swagger docs stay reachable
without a token, and Swagger's **Authorize** button uses the same value.

## Tests

```bash
pip install -e ".[dev]" rusty-bacnet rusty-haystack
pytest tests/unit -q
pytest tests/integration -m integration -q   # needs live devices + :47808 free
```
