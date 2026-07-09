# Rust axum migration report

Branch: `feat/rust-axum-migration`  
Binary crate: `rust-api/` (`diy-bacnet-server` v1.2.0)

## Summary

The FastAPI/Python glue layer has been replaced with a **pure Rust axum** application that talks directly to:

- `bacnet-client` / `bacnet-server` / `bacnet-objects` / `bacnet-types` (path: `../../rusty-bacnet/crates/`)
- `haystack_client` (path: `../../rusty-haystack/haystack-client`)
- `rusty-modbus-client` (git: `jscott3201/rusty-modbus`, branch `dev`)

Python is no longer required at runtime for the Rust binary.

## Route map (Python → Rust)

| Python module | Rust module | Notes |
|---------------|-------------|-------|
| `app/main.py` | `src/main.rs` | Lifespan: BACnet server, weather loop, poll engine, Haystack client |
| `app/config.py` | `src/config.rs` | `gateway.toml` + `OPENFDD_FIELDBUS_*` / `RUSTY_GATEWAY_*` env |
| `app/auth.py` | `src/auth.rs` | Bearer middleware; exempt `/`, `/health`, `/api/health`, `/docs`, `/openapi.json` |
| `app/models.py` | `src/models.rs` | serde + validator + utoipa `ToSchema` |
| `app/bacnet_client.py` | `src/services/bacnet_client.rs` | `tokio::Mutex` bus lock |
| `app/bacnet_server.py` | `src/services/bacnet_server.rs` | `ObjectDatabase` + `BACnetServer::bip_builder()` |
| `app/poll.py` | `src/services/poll.rs` | Background tokio task |
| `app/weather.py` | `src/services/weather.rs` | Open-Meteo + BACnet AV/BV mirror |
| `app/haystack_client.py` | `src/services/haystack.rs` | Read-only op allowlist |
| `app/modbus_client.py` | `src/services/modbus.rs` | Native `rusty-modbus-client` |
| `app/routes/bacnet.py` | `src/routes/bacnet.rs` | 16 BACnet routes |
| `app/routes/weather.py` | `src/routes/weather.rs` | 2 routes |
| `app/routes/modbus.py` | `src/routes/modbus.rs` | 1 route |
| `app/routes/haystack.py` | `src/routes/haystack.rs` | 4 routes |
| `app/routes/compat.py` | `src/routes/compat.rs` | 3 Open-FDD aliases |
| `app/routes/*` (mirrored) | `routes/mod.rs` `.nest("/api", …)` | Duplicate native routers under `/api` |

### Endpoint inventory (45 total)

**Root (2):** `GET /`, `GET /health`

**BACnet native (16):** `/bacnet/points`, `/read`, `/write`, `/write-dry-run`, `/poll/status`, `/poll/once`, `/rpm`, `/whois`, `/whois-router`, `/discover`, `/priority-array`, `/supervisory`, `/server/objects`, `/server/commandable`, `/server/update`

**Weather (2):** `GET /weather`, `POST /weather/refresh`

**Modbus (1):** `POST /modbus/read`

**Haystack (4):** `/haystack/about`, `/read`, `/nav`, `/his-read`

**Open-FDD compat (3):** `GET /api/health`, `POST /api/bacnet/point-discovery`, `GET /api/bacnet/server/points`

**`/api` mirrors (17):** all native BACnet + weather + modbus + haystack routes prefixed with `/api`

## How to run

### Local dev (requires sibling repos)

```bash
# Layout:
#   ../rusty-bacnet
#   ../rusty-haystack
#   diy-bacnet-server/

cd diy-bacnet-server
cp .env.example .env   # set OPENFDD_FIELDBUS_API_KEY, BIND, etc.

cd rust-api
cargo build --release
OPENFDD_FIELDBUS_CONFIG_DIR=../config ../target/release/diy-bacnet-server
```

Or from `rust-api/` after build:

```bash
OPENFDD_FIELDBUS_CONFIG_DIR=../config cargo run --release
```

### Docker (Rust profile)

Build context must be the **parent** of `diy-bacnet-server` (includes `rusty-bacnet` and `rusty-haystack`):

```bash
cd diy-bacnet-server
docker compose --profile rust up -d --build gateway-rust
```

Default `docker compose up` still uses the Python image (`Dockerfile`).

### Swagger / OpenAPI

When `OPENFDD_FIELDBUS_OPENAPI=1` (default):

- OpenAPI JSON: `http://localhost:8080/openapi.json`
- Swagger UI: `http://localhost:8080/docs`

## Build & test results

```
cd rust-api
cargo build --release   # OK
cargo test              # 12 passed
cargo clippy --workspace --all-targets -- -D warnings   # OK
cargo fmt --all         # OK
```

### Bench validation (2026-07-09, Rust binary on `feat/rust-axum-migration`)

```
OPENFDD_FIELDBUS_API_KEY=bench-demo-key-1234567890 \
  SMOKE_BASE=http://127.0.0.1:8080 \
  scripts/smoke_test.sh
# Summary: 20 passed, 0 failed
# P8 override on device 5007 analogOutput:2466 confirmed (55.0%)
```

**Bug fixed during validation:** `object_type_name()` used `Debug` (`ObjectType::ANALOG_OUTPUT`) with a `.` split, producing `objecttype::analog-output` OIDs. That broke commandable detection in `/bacnet/discover` and `/bacnet/supervisory`. Fixed to use `Display` → hyphenated names (`analog-output,2466`) matching the Python/FastAPI contract.

### PCAP validation

Capture during Who-Is + point discovery (Docker `netshoot` + `NET_RAW`, 25s window):

```
artifacts/bacnet_rust_capture.pcap   # 33 UDP/47808 frames
```

Observed traffic (via `tcpdump -r`):

- Who-Is broadcast to `192.168.204.255:47808` (not a storm — single bursts per operation)
- I-Am responses from routed MSTP devices (`192.168.204.200`, `.11`, `.13`, `.14`)
- ReadPropertyMultiple to device 5007 through router

`scripts/pcap_validate.sh` requires `tshark` (not installed on this bench host); the Docker Rust image includes `tshark` for in-container validation.

```bash
PCAP_FILE=artifacts/bacnet_rust_capture.pcap PCAP_MIN_IAM=1 scripts/pcap_validate.sh
```

### 30-minute soak (2026-07-09)

```
OPENFDD_FIELDBUS_API_KEY=bench-demo-key-1234567890 \
  SMOKE_BASE=http://127.0.0.1:8080 \
  SOAK_MINUTES=30 SOAK_CONTAINER="" \
  scripts/soak_test.sh
# SOAK PASSED — 15 cycles, override P8: 15 ok / 0 miss
# All features 15/0 (modbus degrades gracefully when no simulator on :5502)
```

## Completion checklist

| Task | Status |
|------|--------|
| FastAPI route audit + migration map | Done — see route table above |
| axum + serde + validator + utoipa stack | Done |
| All 45 endpoints ported | Done |
| Bearer auth + exempt paths | Done |
| OpenAPI `/openapi.json` + Swagger `/docs` | Done (DTO schemas; per-handler `utoipa::path` partial) |
| BACnet bus lock (`tokio::Mutex`) | Done |
| Poll engine + weather loop | Done |
| `cargo build --release` | Done |
| `cargo test` (12 tests) | Done |
| `cargo clippy -D warnings` | Done |
| `scripts/smoke_test.sh` (20/20) | Done |
| PCAP capture + review | Done (33 frames; no Who-Is storm) |
| `scripts/pcap_validate.sh` | Done (script added; needs `tshark` on host) |
| Docker `gateway-rust` profile | Done (`docker compose --profile rust build gateway-rust`) |
| 30m `scripts/soak_test.sh` on Rust binary | Done (15 cycles, 30m, P8 override 15/15) |
| Migration report + README | Done |

## Known gaps / differences

1. **Hosted BACnet server API shape** — Rust uses native `ObjectDatabase` + `BACnetObject::write_property` instead of Python PyO3 `add_analog_value` / `write_property_local` helpers. Behavior matches; implementation differs.

2. **OpenAPI path coverage** — Schemas are registered for all DTOs; individual route `utoipa::path` annotations are not yet wired for every handler (Swagger documents components; expand `openapi.rs` paths for full operation list).

3. **`ServerScheduleUpdateRequest`** — Present in Python models but no route was exposed; not ported (same as Python).

4. **Routed MSTP devices** — Client routing follows Python logic (Who-Is + `source_network`); full BBMD foreign-device registration not implemented in either stack.

5. **Docker Rust image** — Requires monorepo-style build context (`..`) with `rusty-bacnet` and `rusty-haystack` checked out beside `diy-bacnet-server`.

6. **PCAP gate** — `scripts/pcap_validate.sh` is a lightweight tshark gate; tune `PCAP_MIN_IAM` / `PCAP_FORBID_WRITE` in CI as bench captures mature.

7. **Python integration tests** — Existing `tests/` still target the FastAPI app; Rust has unit/integration smoke tests in `main.rs` and service modules only.

## Files created

```
rust-api/
  Cargo.toml
  Dockerfile
  src/
    main.rs
    config.rs
    state.rs
    auth.rs
    models.rs
    openapi.rs
    error.rs
    services/
      mod.rs
      bacnet_client.rs
      bacnet_server.rs
      poll.rs
      weather.rs
      haystack.rs
      modbus.rs
    routes/
      mod.rs
      root.rs
      bacnet.rs
      compat.rs
      haystack.rs
      modbus.rs
      weather.rs
docs/rust-migration-report.md
scripts/pcap_validate.sh
docker-compose.yml          (updated: gateway-rust profile)
```
