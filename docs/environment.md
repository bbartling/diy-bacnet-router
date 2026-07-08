---
title: Environment
nav_order: 10
---

# Environment variables

| Variable | Purpose |
|----------|---------|
| `RUSTY_GATEWAY_API_KEY` | If set, require `Authorization: Bearer <key>` on protected routes. When unset, all routes are open. |
| `RUSTY_GATEWAY_OPENAPI` | `0` / `false` / `no` disables `/docs`, `/redoc`, `/openapi.json`. Enabled by default. |
| `RUSTY_GATEWAY_SWAGGER_SERVERS_URL` | Optional OpenAPI `servers` URL (e.g. when behind a reverse-proxy path). |
| `RUSTY_GATEWAY_CONFIG_DIR` | Override the config directory (defaults to `./config`). |
| `RUSTY_GATEWAY_HTTP_HOST` | HTTP bind host (default `0.0.0.0`). |
| `RUSTY_GATEWAY_HTTP_PORT` | HTTP bind port (default `8080`). |
| `RUSTY_GATEWAY_RELOAD` | `1` / `true` enables uvicorn auto-reload for local dev. |
| `RUSTY_GATEWAY_BIND` | Client NIC IP used as the unicast source; the directed `/24` broadcast is derived from it. |
| `RUSTY_GATEWAY_SERVER_BIND` | Override the BACnet **server** bind interface (default `0.0.0.0` so broadcast Who-Is is received). |
| `RUSTY_GATEWAY_BROADCAST` | Override the BACnet broadcast address for server and client. |
| `HAYSTACK_BASE_URL` | Haystack server base URL (default `http://127.0.0.1:8081`). |
| `HAYSTACK_USER` / `HAYSTACK_PASS` | Haystack SCRAM credentials. |
| `MODBUS_DEFAULT_HOST` | Default Modbus TCP host. |

## Bearer token notes

- Keep `RUSTY_GATEWAY_API_KEY` set for any non-trivial deployment (LAN / edge / shared host).
- The same value is used by:
  - direct HTTP clients sending `Authorization: Bearer <key>`
  - Swagger **Authorize** in `/docs`
  - any service that proxies requests into this gateway
- `/`, `/health`, `/docs`, `/redoc`, and `/openapi.json` are exempt so liveness and docs stay reachable without a token.
