---
title: Quick start
layout: default
nav_order: 2
---

# Quick start (development)

Run the management scaffold on your workstation:

```bash
cp config/router.example.toml config/router.toml
cargo run -p routerd -- --config config/router.toml
```

Open [http://127.0.0.1:8080](http://127.0.0.1:8080).

## API (no URL versioning)

| Endpoint | Purpose |
| --- | --- |
| `GET /healthz` | Readiness (`ready_to_route` false until routing gates pass) |
| `GET /api/status` | Name, **VERSION**, git SHA, runtime |
| `GET /api/metrics/snapshot` | Aggregate counters (REST fallback) |
| `GET /api/ws/metrics` | **WebSocket** — live MS/TP trunk health (~1 Hz) |
| `GET /metrics` | Prometheus text |

Build the React dashboard:

```bash
npm --prefix frontend/web ci
npm --prefix frontend/web run check
npm --prefix frontend/web run build
```

{: .warning }
The dashboard is a **LAN-only operator console**. Do not expose it directly on
the public internet.
