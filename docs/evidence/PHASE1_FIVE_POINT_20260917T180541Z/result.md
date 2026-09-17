# Phase 1 — Workbench-proxy five-point SLO (2026-09-17)

## Lab inventory (capture before FEC)

| Host | Role | Notes |
|------|------|-------|
| workerpi1 `192.168.204.59` | DIY router | FTDI by-id `…BH002I9S…`; **latency_timer=1**; MIF=2; Max_Master=2; DNET **2001** MAC1 |
| workerpi2 `192.168.204.60` | mini | CH343 by-id `…5A98075745…`; MAC2; instance **123102**; Max_Master=2 |
| bensbench `192.168.204.11` | bacpypes3 oracle | UDP 47808 free for soak |

**Binary under test:** aarch64 musl build of `fix/repin-rusty-bacnet-7e0d13a` @ `b610f76` with rusty-bacnet **`7e0d13a`** (was live `b5a5708` / pin `24e3439`). Deployed with `--route-enable --qualify-secs 0`. `/api/metrics/rp-spans` → HTTP 200.

**Config note:** TOML `[router] enabled=false` is overridden by systemd `--route-enable`. Healthz still reports product `ready_to_route=false` (intentional claim wording).

## Pre-repin single-AI probe (old binary)

`PHASE1_WORKBENCH_PROXY_20260917T175443Z/single_ai1_burst5_seq30.json`

- seq30 AI:1: **30/30**, p50 **36.0**, p95 **38.4**, max **48.2** ms

## Five-point concurrent soak (tip binary) — PASS

Script: `scripts/phase1_five_point_soak.py`  
Artifact: `five_point_10m.json`

| Metric | Value |
|--------|-------|
| Duration | 600 s |
| Interval | 1.0 s (5 concurrent RPs / round) |
| Rounds | 517 |
| ok / fail | **2585 / 0** |
| overall p50 / p95 / p99 / max | **99.7 / 162.0 / 165.8 / 178.7** ms |

**SLO (phase 1):** p95 ≤ 250 ms, max ≤ 2000 ms, zero fails → **PASS**.

Objects polled (mini object-list): `device:123102.object-name`, `AI:1`, `AV:2`, `BI:1`, `BV:2` present-value.

## FX Workbench

Operator UI capture still welcome for human screenshots; **independent bacpypes3 five-point oracle is PASS** on tip pin. Do not treat Niagara timeout inflation as a fix.

## rusty-bacnet

Repin PR: https://github.com/bbartling/diy-bacnet-router/pull/70 — `24e3439` → **`7e0d13a`** (v0.11 tip of `dev`).
