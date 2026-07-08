---
title: Environment
nav_order: 10
---

# Environment variables

Every setting accepts an `OPENFDD_FIELDBUS_*` name (preferred for Open-FDD
deployments) and falls back to the original `RUSTY_GATEWAY_*` name. When both are
set, `OPENFDD_FIELDBUS_*` wins.

| Variable (preferred) | Legacy fallback | Purpose |
|----------------------|-----------------|---------|
| `OPENFDD_FIELDBUS_API_KEY` | `RUSTY_GATEWAY_API_KEY` | If set, require `Authorization: Bearer <key>` on protected routes. When unset, all routes are open. |
| `OPENFDD_FIELDBUS_OPENAPI` | `RUSTY_GATEWAY_OPENAPI` | `0` / `false` / `no` disables `/docs`, `/redoc`, `/openapi.json`. Enabled by default. |
| — | `RUSTY_GATEWAY_SWAGGER_SERVERS_URL` | Optional OpenAPI `servers` URL (e.g. when behind a reverse-proxy path). |
| `OPENFDD_FIELDBUS_CONFIG_DIR` | `RUSTY_GATEWAY_CONFIG_DIR` | Override the config directory (defaults to `./config`). |
| `OPENFDD_FIELDBUS_HTTP_HOST` | `RUSTY_GATEWAY_HTTP_HOST` | HTTP bind host (default `0.0.0.0`). |
| `OPENFDD_FIELDBUS_HTTP_PORT` | `RUSTY_GATEWAY_HTTP_PORT` | HTTP bind port (default `8080`). |
| `OPENFDD_FIELDBUS_RELOAD` | `RUSTY_GATEWAY_RELOAD` | `1` / `true` enables uvicorn auto-reload for local dev. |
| `OPENFDD_FIELDBUS_BIND` | `RUSTY_GATEWAY_BIND` | Client NIC IP used as the unicast source; the directed `/24` broadcast is derived from it. |
| `OPENFDD_FIELDBUS_SERVER_BIND` | `RUSTY_GATEWAY_SERVER_BIND` | Override the BACnet **server** bind interface (default `0.0.0.0` so broadcast Who-Is is received). |
| `OPENFDD_FIELDBUS_BROADCAST` | `RUSTY_GATEWAY_BROADCAST` | Override the BACnet broadcast address for server and client. |
| `OPENFDD_FIELDBUS_BACNET_PORT` | `RUSTY_GATEWAY_BACNET_PORT` | BACnet/IP UDP port for the hosted server + client Who-Is listener (default `47808`; e.g. `47809`). Per-device destination ports come from `field_devices.toml`. |
| `OPENFDD_FIELDBUS_POLL_ENABLED` | `RUSTY_GATEWAY_POLL_ENABLED` | `false` disables the background poll engine (default enabled). |
| `OPENFDD_FIELDBUS_POLL_INTERVAL_SECS` | `RUSTY_GATEWAY_POLL_INTERVAL_SECS` | Poll cycle interval in seconds (default `60`). |
| `OPENFDD_FIELDBUS_GIT_SHA` | `GIT_SHA` | Build SHA reported by `/health` and `/api/health`. |
| `HAYSTACK_BASE_URL` | — | Haystack server base URL (default `http://127.0.0.1:8081`). |
| `HAYSTACK_USER` / `HAYSTACK_PASS` | — | Haystack SCRAM credentials. |
| `MODBUS_DEFAULT_HOST` | — | Default Modbus TCP host. |

## Bearer token notes

- Keep the API key set for any non-trivial deployment (LAN / edge / shared host).
- The same value is used by:
  - direct HTTP clients sending `Authorization: Bearer <key>`
  - Swagger **Authorize** in `/docs`
  - any service that proxies requests into this sidecar
- `/`, `/health`, `/api/health`, `/docs`, `/redoc`, and `/openapi.json` are exempt so liveness and docs stay reachable without a token.
