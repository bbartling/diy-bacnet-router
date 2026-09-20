# Checkpoint — 2026-09-18 — Clause 9 baud matrix; hold @38400; Open-FDD test bench

**Date (UTC):** 2026-09-18  
**Status:** Dual-mini + FEC path proven at **38400**. FEC-off baud matrix: **76800 PASS**, **19200/9600 OPEN** (USB/timing).  
**Hold:** Leave the live trunk at **38400**. Re-attach FEC (MAC7, read-only). Use as **Open-FDD OT / MQTT stress bench** — do not chase 9600/19200 until upstream timing guidance lands.

## What landed (this cycle)

| Item | Location |
|---|---|
| rusty-bacnet pin `acbf7bae` + FEC M3b + baud matrix | PR [#70](https://github.com/bbartling/diy-bacnet-router/pull/70) → `master` |
| 38400 dual-mini + FEC evidence | [`CLAUSE9_38400_DUAL_MINI_FEC_20260918T173211Z/`](CLAUSE9_38400_DUAL_MINI_FEC_20260918T173211Z/result.md) |
| FEC-off baud matrix | [`CLAUSE9_BAUD_MATRIX_FEC_OFF_20260918T174441Z/`](CLAUSE9_BAUD_MATRIX_FEC_OFF_20260918T174441Z/result.md) |
| Beginner stack map / lab trunk check | [`docs/learn/stack-map.md`](../learn/stack-map.md), [`lab-trunk-check.md`](../learn/lab-trunk-check.md) |
| G11 partial | [`docs/TESTING.md`](../TESTING.md) |

## Honest baud claim (this hardware)

| Baud | Passive | Routed RP | Claim |
|---|---|---|---|
| **38400** | PASS | PASS (with/without FEC) | **Supported** — lab default |
| **76800** | PASS | PASS (minis only, FEC off) | **Supported** (minis) |
| **19200** | PASS | FAIL AbortPDU no-response | **Not claimed** — USB/reply-window suspect |
| **9600** | FAIL (`tokens:0`, low valid_ratio) | FAIL | **Not claimed** — link/decode |

Upstream discussion: [jscott3201/rusty-bacnet#707](https://github.com/jscott3201/rusty-bacnet/issues/707) (related [#502](https://github.com/jscott3201/rusty-bacnet/issues/502)).

## Locked lab topology (resume / Open-FDD stress)

```text
bacpypes3 oracle  192.168.204.59 (or other non-router IP)  B/IP net 1
        |
bensbench 192.168.204.11  diy-bacnet-router
  --route-enable --qualify-secs 0
  config: config/router.lab-bensbench-dual-mini.toml
  MS/TP net 2001  MAC 1  Waveshare C / FTDI @ 38400
  Max_Master=7  (room for FEC MAC7)
        |
  RS-485 trunk @ 38400
        |
  pi2 192.168.204.60  mstp-mini-device  MAC2  device 123102  CH343
  pi1 192.168.204.59  mstp-mini-device  MAC3  device 123103  FTDI
  FEC (optional)               MAC7  @38400 only — READ ONLY, no writes
```

**Never** plant DNET **2000**. **Never** write the FEC. FTDI `latency_timer=1` on listen/router hosts (Issue #66).

## Pins at hold

| Pin | Value |
|---|---|
| diy-bacnet-router `master` | PR #70 merge (`2b4ea47` or tip after hold docs) |
| rusty-bacnet | `acbf7baefe69d05f2368763dcc659d68e4bc114c` |
| Lab baud | **38400** |

## Why 19200/9600 stay OPEN (agent summary)

- rusty-bacnet MS/TP uses **fixed ms** `T_REPLY_TIMEOUT_MS=255`, `T_USAGE_TIMEOUT_MS=20`; turnaround is baud-scaled.
- On USB-UART, frame serialization + chunking eats the reply budget at slower bauds → routed confirmed services fail while short token frames still pass (**19200** pattern).
- **9600** failed earlier (passive garble / no tokens) on this mixed FTDI+CH343 trunk — treat as link/decode first.
- Same pin **passes** at 38400 and 76800 — not a general “MS/TP broken” claim.

## Resume checklist

1. Confirm minis + router still **38400**; FEC on cable if needed.
2. `latency_timer=1` on FTDI ports.
3. bacpypes on a **different IP** than the router BIP bind.
4. For Open-FDD: fieldbus/MQTT publish into this DNET 2001 trunk; keep writes off FEC.
5. Do not claim 9600/19200 until upstream timing + re-prove on wire.

## Deferred (not blocking this hold)

- ReplyPostponed / Max_Info_Frames burst matrix
- Multi-hour soak lite
- Full PICS self-test beyond baud honesty already recorded
- Exact-image / Buildroot G7–G11

## Superseding full matrix (2026-09-20)

Full allowed-set FEC-off smoke at rusty-bacnet tip `9e5168c5`:
[`CLAUSE9_BAUD_MATRIX_FULL_FEC_OFF_20260920T132700Z/`](CLAUSE9_BAUD_MATRIX_FULL_FEC_OFF_20260920T132700Z/result.md).

| Baud | Claim |
|---|---|
| **38400** | **Default / supported** (± FEC) |
| **57600 / 76800 / 115200** | Lab-supported (FEC off, dual mini) |
| **9600 / 19200** | Still **not claimed** |

Live trunk remains **38400**.

