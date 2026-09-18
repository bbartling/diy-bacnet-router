# Phase-2 FEC via DIY router — 20260918T124240Z

## Topology

| Role | Host | Notes |
|------|------|-------|
| DIY router | bensbench `192.168.204.11` | Waveshare C FTDI; MAC1; DNET **2001**; `--route-enable --qualify-secs 0` |
| Mini | workerpi2 `.60` | MAC2 / device **123102** |
| FEC | trunk | MAC7 / device **5007** — **read-only** |
| BIP client | workerpi1 `.59` | bacpypes3 0.0.106 `--route-aware --network 1` |

Baud 38400, Max_Master=7. Never DNET 2000. No FEC writes.

## Preflight

- Passive (10s): sources_seen **[2, 7]**, valid_ratio ≈0.998 — see `passive.json`
- MAC1 qualify (15s): next_station=**2**, tokens_since_pfm=26 — see `qualify-mac1.json`

## Controlled BIP oracle (from pi1)

| Probe | Result |
|-------|--------|
| Who-Is-Router-To-Network(2001) | **PASS** — I-Am-Router from bensbench |
| `2001:2` device:123102 object-name | **PASS** → Rust MS/TP Mini Device (~53 ms) |
| `2001:2` AI:1 present-value | **PASS** → 3.0 (~69 ms) |
| `2001:7` device:5007 object-name | **PASS** → BENS BENCHTEST BOX (~85 ms) |
| `2001:7` AI:1173 present-value | **PASS** (~65 ms) |
| Explicit `@192.168.204.11` variants | **PASS** |

Raw: `controlled-rp-from-pi1.json`

## False negatives (do not re-open as route-dead)

| Client placement | Symptom | Cause |
|------------------|---------|-------|
| Same host as router BIP bind (`.11`) | Timeout / unknown-route | Client cannot share router UDP 47808 / bind |
| Docker macvlan `.12` on parent `eno1` | Timeout | Host↔macvlan isolation on same parent |

**Product path (Workbench / remote BIP) matches the working pi1 client topology.**

## Gate

| Gate | Status |
|------|--------|
| FEC+mini hear (passive) | **PASS** |
| MAC1 join | **PASS** |
| Routed RP mini + FEC | **PASS** |
| Product `ready_to_route=true` | still **false** by claim wording until formal G7/G8 closeout |
| Exact-image G7/G8 | **OPEN** |

## Tip re-prove (`acbf7bae`)

After `chore(deps): repin rusty-bacnet to acbf7bae`, rebuilt `routerd` reports `rusty_bacnet_rev=acbf7bae`. Pi1 bacpypes3 re-probe:

| Probe | Result |
|-------|--------|
| `2001:2` device:123102 object-name | **PASS** |
| `2001:7` device:5007 object-name | **PASS** → BENS BENCHTEST BOX |
| `2001:7` AI:1173 present-value | **PASS** |

Raw: `controlled-rp-tip-acbf7bae.json`
