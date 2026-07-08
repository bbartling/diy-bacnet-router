---
title: Getting started
nav_order: 2
---

# Getting started

This page matches the **[README](https://github.com/bbartling/diy-bacnet-server/blob/main/README.md)** quick start: create a venv, set an optional API key in **`.env`**, then run from source or Docker.

---

## Prerequisites

- **Python 3.12+** for local runs, or **Docker** for container runs.
- **Git** (only for first clone).
- **`openssl`** (or another way to generate a random secret) for `RUSTY_GATEWAY_API_KEY`.

---

## Clone and venv (Python)

If you are **already** inside a clone, skip `git clone` / `cd`.

```bash
git clone https://github.com/bbartling/diy-bacnet-server.git
cd diy-bacnet-server
python3 -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install --upgrade pip
pip install -e . rusty-bacnet rusty-haystack
```

---

## Bearer secret (`.env`)

Create **one line** in **`.env`** at the repo root (file is **gitignored**):

```bash
printf 'RUSTY_GATEWAY_API_KEY=%s\n' "$(openssl rand -hex 32)" > .env
```

Load it into your shell for local runs:

```bash
set -a && . ./.env && set +a
```

Skip this section only for **unsecured loopback** experiments. When the key is
unset, all routes are open.

---

## Run from source

```bash
chmod +x scripts/*.sh
./scripts/run_dev.sh
```

`run_dev.sh` runs the port preflight and starts `python -m app.main`. HTTP binds
`0.0.0.0:8080` and the BACnet server binds `0.0.0.0:47808`.

- **`RUSTY_GATEWAY_OPENAPI=0`** disables `/docs`, `/redoc`, `/openapi.json`.
- **`RUSTY_GATEWAY_BIND`** sets the client NIC IP (and derives the directed broadcast).

`scripts/preflight_free_47808.sh` frees UDP `:47808` before the server binds.

---

## Docker

`--network host` puts the container on the **host's** IP stack so BACnet/IP behaves like bare metal. Build from the **parent directory** (siblings `rusty-bacnet`, `rusty-haystack` provide the wheels):

```bash
cd ..
cp diy-bacnet-server/.env.example diy-bacnet-server/.env
docker compose -f diy-bacnet-server/docker-compose.yml up -d --build
```

---

## Verify HTTP

- **`GET /health`** — liveness (no Bearer required).
- **`GET /docs`** — Swagger UI when enabled; **Authorize** uses the same value as **`RUSTY_GATEWAY_API_KEY`**.

From another machine on the LAN, use **`http://<server-LAN-IP>:8080/docs`**, not `127.0.0.1`. Ensure the host firewall allows **TCP 8080** (and **UDP 47808** for BACnet).

```bash
curl -s http://127.0.0.1:8080/health
# {"ok":true,"service":"diy-bacnet-server"}
```

---

## Point catalog

The hosted BACnet objects come from **`config/objects.csv`**; the client's field
devices come from **`config/field_devices.toml`**. See [CSV point model](csv-points).

---

## Host-level note

Only one BACnet service should bind UDP `:47808` on a host at a time. The
preflight script stops whatever is holding the port before the server starts.
