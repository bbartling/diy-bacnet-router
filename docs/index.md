---
title: Home
nav_order: 1
description: "Pure Rust axum BACnet/IP + REST edge gateway; CSV-hosted points, full BACnet client, Modbus and Haystack."
permalink: /
---

# DIY BACnet Server (Rust)

{: .fs-6 .fw-300 }
Single-process **BACnet/IP** device and **REST + Swagger** HTTP API for lab and edge.
**100% Rust** — axum HTTP, **rusty-bacnet**, **rusty-modbus**, **rusty-haystack**.
CSV point table, full BACnet **client** toolkit (read/write/RPM/Who-Is/discover/priority-array/supervisory).

---

## Quick start

Optional gitignored **`.env`** with `OPENFDD_FIELDBUS_API_KEY=…` (Bearer for the API
and Swagger **Authorize**).

```bash
cd diy-bacnet-server
cp .env.example .env
chmod +x scripts/*.sh
./scripts/run_dev.sh
# Swagger: http://127.0.0.1:8080/docs
```

### Docker

```bash
cd ..   # parent with rusty-bacnet / rusty-haystack siblings
docker compose -f diy-bacnet-server/docker-compose.yml up -d --build
```

---

## Endpoints

| What | URL | Notes |
|------|-----|--------|
| **Swagger / OpenAPI** | `http://<host>:8080/docs` | On by default. |
| **REST API** | `http://<host>:8080/bacnet/*`, `/weather`, `/modbus/*`, `/haystack/*` | Bearer when API key set. |
| **BACnet/IP** | UDP **47808** | Hosted device 599999; same process as HTTP. |

---

## Documentation

| Section | Description |
|---------|---------------|
| [Getting started](getting-started) | `.env`, Docker, verify HTTP |
| [CSV point model](csv-points) | Hosted point table |
| [Client BACnet](client-bacnet) | Client operations |
| [Modbus TCP](modbus-tcp) | `POST /modbus/read` |
| [Environment](environment) | Environment variables |
| [CI & publishing](ci-and-publishing) | Actions, tests |

See the [README](https://github.com/bbartling/diy-bacnet-server#readme) for the full endpoint table.
