# ISSUE66 fix — FTDI latency_timer (2026-09-16T190000Z)

## Causality (A2)

| Experiment | seq30 / seq60 | p50 | p95 | max | Verdict |
| --- | ---: | ---: | ---: | ---: | --- |
| Baseline MIF=1, FTDI latency=16 | 30/30 | 48 ms | **3038 ms** | 3042 ms | A0 reproduce |
| MIF=2 only, latency=16 | 30/30 | 48 ms | **3042 ms** | 3042 ms | **no change** |
| MIF=2 + FTDI latency=**1** | 60/60 | 36 ms | **38 ms** | 50 ms | **PASS** |

~3.0 s outliers matched bacpypes default APDU retry after dropped/late MS/TP
frames under FTDI's default 16 ms USB latency coalescing — not Max_Info_Frames=1
serialization alone, and not a rusty-bacnet `try_send` drop under this load.

## Fix (A3)

1. Set `/sys/bus/usb-serial/devices/*/latency_timer` to **1** on workerpi1.
2. Persist via udev `ansible/files/99-ftdi-latency.rules` + `scripts/set-ftdi-latency.sh`.
3. Lab TOML `max_info_frames = 2` retained for concurrent-poll headroom (orthogonal).

## Instrumentation (A1)

- `crates/rusty-bacnet-adapter/src/rp_span.rs` — bounded RP span store + tests.
- `GET /api/metrics/rp-spans` on management API (OpenAPI updated).

## A4 source soak

Controlled bacpypes3 poll `2001:2` AI:1 every 0.5 s for **30 min** (Workbench
operator capture remains OPEN). Storm threshold: >2 misses / 60 s window.

Artifacts: `controlled-rp-ftdi1.json`, `a4-30m.log`, `a4-30m-summary.json`.

## Gate

| Gate | Status |
| --- | --- |
| A1 spans + tests | **PASS** |
| A2 causality | **PASS** (FTDI latency) |
| A3 fix applied on lab Pi | **PASS** (udev installed) |
| A4 30m controlled soak | **PASS** — 3364/3364 ok, slow=0, storm=0, p95=39.5 ms, max=120.8 ms |
| Workbench 5-point | **OPEN** (Windows operator) |
