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

## What "Rust wheels in a Python app" means

The protocol stacks (`rusty-bacnet`, `rusty-modbus`, `rusty-haystack`) are
written in **Rust**, not Python. They reach Python through **[PyO3](https://pyo3.rs)**,
which compiles the Rust crate into a native **CPython extension module** — a
shared library (`.so` on Linux) that Python can `import` exactly like a normal
module. That compiled artifact is packaged and distributed as a **wheel**
(`.whl`), the standard binary install format for Python.

So when you run `pip install rusty-bacnet`, pip downloads a prebuilt wheel
containing the compiled Rust code and drops it into your environment. Your
Python then does `import rusty_bacnet` and calls into Rust at native speed —
there is no Python BACnet/Modbus/Haystack implementation involved. This app is
therefore "Python only at the web layer": FastAPI handles HTTP/Swagger, and
every byte on the wire is produced and parsed by Rust.

Practical consequences:

- **A wheel is platform- and Python-version-specific.** `rusty-bacnet` and
  `rusty-haystack` publish wheels that work on **CPython 3.12+**; `rusty-modbus`
  currently only publishes a **CPython 3.14** wheel. If pip can't find a wheel
  matching your interpreter/OS, the import fails — that is why `/modbus/*`
  returns a clear "not installed" error on a 3.12 runtime.
- **No Rust toolchain is needed to *run* the app** — the wheel is already
  compiled. You only need Rust/`maturin` if you want to build a wheel from
  source (e.g. for an unpublished branch).
- Verify what's installed with `pip show rusty-bacnet` / `pip list`.

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

### Non-default BACnet UDP port

Default BACnet/IP is UDP `47808` (0xBAC0), but some buildings run the whole
network on another port (e.g. `47809`). Set **`OPENFDD_FIELDBUS_BACNET_PORT`**
(or the legacy `RUSTY_GATEWAY_BACNET_PORT`) and both the hosted server and the
client's Who-Is listener move to that port:

```bash
# .env
OPENFDD_FIELDBUS_BACNET_PORT=47809
```

Per-device *destination* ports are independent and come from
`config/field_devices.toml` (`port = ...` per device), so you can also talk to a
device on a different port than the one you host on.

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

## Docker (long-running deployment)

The image is a slim `python:3.12-slim` that installs the `rusty-bacnet` and
`rusty-haystack` wheels from PyPI — no Rust toolchain or source build required,
so it builds in seconds. `docker compose` runs it with host networking and
`restart: unless-stopped`, so it stays up across crashes and host reboots.

```bash
cd diy-bacnet-server
cp .env.example .env          # then set OPENFDD_FIELDBUS_API_KEY / _BIND / _BACNET_PORT
docker compose up -d --build
docker compose logs -f        # follow
```

The container exposes a Docker `HEALTHCHECK` against `/health`, so
`docker ps` shows `healthy` once it's serving.

> `rusty-modbus` only ships a Python 3.14 wheel today, so `/modbus/*` returns a
> clear "not installed" error on this 3.12 image until that wheel lands on 3.12.

## Services

| Service | What it does |
|---------|--------------|
| **BACnet server** | Hosts device 599999 on `:47808` with weather + diagnostic objects. |
| **BACnet client** | Read / write / RPM / Who-Is / discovery / priority-array / supervisory against field devices. |
| **Weather** | Polls Open-Meteo, caches it, and mirrors it into BACnet objects. |
| **Modbus** | Batched Modbus TCP register reads with decode / scale / offset. |
| **Haystack** | Read-only Haystack client (about / read / nav / hisRead). |

## API (Bearer `OPENFDD_FIELDBUS_API_KEY` when set)

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

`scripts/soak_test.sh` runs the same feature matrix on a **60-second loop for
30 minutes** (configurable via `SOAK_MINUTES` / `SOAK_INTERVAL_SECS`), including
Haystack (about/read/nav/his-read) and Modbus (live read or graceful degradation
when the 3.12 wheel is unavailable). It records container memory each cycle to
catch leaks and prints a per-feature pass/fail table at the end.

```bash
OPENFDD_FIELDBUS_API_KEY=<key> SMOKE_BASE=http://127.0.0.1:8080 \
  scripts/smoke_test.sh

OPENFDD_FIELDBUS_API_KEY=<key> SOAK_MINUTES=30 \
  scripts/soak_test.sh
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
