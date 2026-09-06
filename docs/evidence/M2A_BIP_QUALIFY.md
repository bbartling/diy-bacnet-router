# M2A / G6 — BACnet/IP port qualification (complete matrix)

| Field | Value |
| --- | --- |
| Milestone | **M2A / G6** (M2B MS/TP remains open; full M2 incomplete) |
| Topology | run-unique netns pair, `192.0.2.1/24` ↔ veth ↔ `192.0.2.2/24` |
| Upstream pin | `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` |
| Harness | `scripts/bip-qualify-netns.sh` |
| Independent oracle | `scripts/bip_bvll_oracle.py` (stdlib BVLL; not rusty-bacnet) |
| Opt-in | `diy-bacnet-router --bip-qualify` (ordinary boot does **not** start B/IP) |

## Claims (exact)

Subgates (default `N=4`):

| Subgate | Expected |
| --- | --- |
| U_peer_to_dut | DUT `bip_rx` delta = N; oracle send count = N; BVLL 0x0A |
| B_peer_to_dut | DUT `bip_rx` delta = N; directed `192.0.2.255`; BVLL 0x0B |
| U_dut_to_peer | DUT `bip_tx` contributes; oracle RX ≥ N of 0x0A golden NPDU |
| B_dut_to_peer | DUT `bip_tx` contributes; oracle RX ≥ N of 0x0B golden NPDU |
| L_limited | **`NOT_SUPPORTED`** — `255.255.255.255` not claimed on this topology |

Also: forwarding and MS/TP counters remain 0; metrics sequence advances; durable evidence dir with `result.json` + `SHA256SUMS` (never deleted on success).

## Upstream gaps (honest)

- `BipTransport` exposes no public RX/TX/drop counters — adapter atomics only.
- Malformed/drop counters are **not synthesized**; survival + later-valid RX are asserted in lifecycle where covered.

## Non-claims

- No `BACnetRouter` / NPDU forwarding
- No physical MS/TP (M2B)
- No BBMD/FDR / Clause 9 / BTL

## CI

`.github/workflows/bip-qualify.yml` — offline false-PASS tests + root netns matrix; evidence uploaded `if: always()`.

## Post-merge Actions

- Fill after merge: PR/tip run IDs and merge SHA.
