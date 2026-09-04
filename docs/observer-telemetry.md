---
title: Observer telemetry contract
parent: Architecture
nav_order: 8
permalink: /architecture/observer-telemetry/
---

# Shared observer telemetry contract (Vibe13 / Pi probe)

This repository owns the **reusable display contract** for a separate read-only
mini-device / probe observer (not a second router). Routing remains fail-closed
here; observer mode must not imply forwarding.

## Required snapshot fields

| Field | Meaning |
| --- | --- |
| `source.kind` / `source.identity` | Probe vs appliance vs lab fixture |
| `project_git_sha` / `upstream_sha` / `host` / `session` | Provenance |
| `sequence` | Monotonic sample id |
| `timestamp_unix_ms` | UTC sample time |
| `monotonic_elapsed_ms` | Session clock |
| `sample_age_ms` | Freshness relative to producer |
| `management_reachable` | HTTP management answers |
| `transport_running` | Serial/B/IP transport task alive |
| `peer_communication_fresh` | Recent on-wire peer evidence |
| Serial identity / presence / reconnects / last_error | Adapter health |
| RX/TX frames, tokens, PFM, CRC, drops | Only **supported** observations |
| FSM gauges + units + reset semantics | Exact labels |
| Probe RP accounting / latency | Probe-owned, not fabricated server counters |
| CPU / RSS / load / temp | With availability flags |
| `unsupported` / `stale` | Explicit absences |

## Compatibility

- Keep API compatibility or version the schema explicitly (`schema_version`).
- Fixture JSON under `docs/fixtures/observer/` must load without synchronized
  merges into the observer app.
- Missing upstream public statistics need a small non-blocking snapshot API —
  not UART log scraping or a second tty owner.

## Appliance metrics note (M0)

Until rusty-bacnet is integrated, appliance WebSocket snapshots set
`bacnet_telemetry_available=false`. Scaffold zeros are **not** observed wire
errors. `event_count` is a resettable gauge (`dbr_mstp_event_count`), not a
lifetime Prometheus counter.
