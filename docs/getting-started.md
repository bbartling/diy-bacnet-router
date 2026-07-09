---
title: Getting started
nav_order: 2
---

# Getting started

This page matches the **[README](https://github.com/bbartling/diy-bacnet-server/blob/main/README.md)** quick start: set an optional API key in **`.env`**, then run from source or Docker.

---

## Prerequisites

- **Rust 1.96+** for local runs, or **Docker** for container runs.
- Sibling checkouts: `../rusty-bacnet`, `../rusty-haystack` (path dependencies).
- **`openssl`** (or another way to generate a random secret) for `OPENFDD_FIELDBUS_API_KEY`.

---

## Bearer secret (`.env`)

Create **`.env`** at the repo root (file is **gitignored**):

```bash
cp .env.example .env
printf 'OPENFDD_FIELDBUS_API_KEY=%s\n' "$(openssl rand -hex 32)" >> .env
```

When the key is unset, all routes are open.

---

## Run from source

```bash
chmod +x scripts/*.sh
./scripts/run_dev.sh
```

`run_dev.sh` runs the port preflight and starts `cargo run --release` in `rust-api/`.
HTTP binds `0.0.0.0:8080` and the BACnet server binds `0.0.0.0:47808`.

- **`OPENFDD_FIELDBUS_OPENAPI=0`** disables `/docs` and `/openapi.json`.
- **`OPENFDD_FIELDBUS_BIND`** sets the client NIC IP (and derives the directed broadcast).

`scripts/preflight_free_47808.sh` frees UDP `:47808` before the server binds.

---

## Docker

`network_mode: host` puts the container on the **host's** IP stack so BACnet/IP behaves like bare metal. Build from the **parent directory**:

```bash
cd ..
cp diy-bacnet-server/.env.example diy-bacnet-server/.env
docker compose -f diy-bacnet-server/docker-compose.yml up -d --build
```

---

## Verify HTTP

- **`GET /health`** — liveness (no Bearer required).
- **`GET /docs`** — Swagger UI when enabled; **Authorize** uses **`OPENFDD_FIELDBUS_API_KEY`**.

```bash
curl -s http://127.0.0.1:8080/health
```

---

## Point catalog

Hosted BACnet objects: **`config/objects.csv`**. Field devices for the poll engine: **`config/field_devices.toml`**. See [CSV point model](csv-points).

---

## Tests

```bash
cd rust-api && cargo test
OPENFDD_FIELDBUS_API_KEY=... scripts/smoke_test.sh   # live bench
```
