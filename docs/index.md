---
title: Home
nav_order: 1
description: "Rust-backed BACnet/IP + REST edge gateway; CSV-hosted points, full BACnet client operations, Modbus and Haystack."
permalink: /
---

# DIY BACnet Server (Rust)

{: .fs-6 .fw-300 }
Single-process **BACnet/IP** device and **REST + Swagger** HTTP API for lab and edge.
Python is only the web layer; BACnet, Modbus, and Haystack all run on Rust
(**rusty-bacnet**, **rusty-modbus**, **rusty-haystack**) via PyO3. CSV point table,
full BACnet **client** toolkit (read/write/RPM/Who-Is/discover/priority-array/supervisory).

---

## Quick start (Python 3.12+)

Optional gitignored **`.env`** with `RUSTY_GATEWAY_API_KEY=…` (Bearer for the API
and Swagger **Authorize**).

```bash
cd diy-bacnet-server
python3 -m venv .venv && . .venv/bin/activate
pip install -e . rusty-bacnet rusty-haystack
./scripts/run_dev.sh
# Swagger: http://127.0.0.1:8080/docs
```

### Docker

`--network host` gives BACnet/IP (UDP **47808**) bare-metal semantics.

```bash
cd ..   # parent with rusty-bacnet / rusty-haystack siblings
docker compose -f diy-bacnet-server/docker-compose.yml up -d --build
```

---

## Endpoints

| What | URL | Notes |
|------|-----|--------|
| **Swagger / OpenAPI** | `http://<host>:8080/docs` | On by default; disable with `RUSTY_GATEWAY_OPENAPI=0`. |
| **REST API** | `http://<host>:8080/bacnet/*`, `/weather`, `/modbus/*`, `/haystack/*` | Bearer `RUSTY_GATEWAY_API_KEY` when set. |
| **BACnet/IP** | UDP **47808** | Hosted device 599999; same process as HTTP. |

---

## Documentation

| Section | Description |
|---------|---------------|
| [Getting started](getting-started) | venv, `.env`, Docker, verify HTTP, firewall |
| [CSV point model](csv-points) | Hosted point table, types, commandable points |
| [Client BACnet](client-bacnet) | Client read/write/RPM/Who-Is/discover/priority-array/supervisory |
| [Modbus TCP](modbus-tcp) | `POST /modbus/read` (rusty-modbus) |
| [Environment](environment) | Environment variables |
| [CI & publishing](ci-and-publishing) | Actions, Pages, tests |

See the [README](https://github.com/bbartling/diy-bacnet-server#readme) for the full endpoint table.

---

## Philosophy

**BACnet stays in one process, on Rust.** The hosted server binds `0.0.0.0:47808`
(so it hears broadcast Who-Is) while client operations use Who-Is on `:47808`
and unicast reads on ephemeral ports — no shared-socket contention. Callers use
**HTTP + REST**; BACnet stays on UDP.
