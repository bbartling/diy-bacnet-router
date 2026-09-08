# M3 dual-B/IP netns — software PASS, product G7/G8 OPEN

| Field | Value |
| --- | --- |
| Harness | `scripts/route-bip-bip-netns.sh` |
| DUT unlock | `--route-bip-bip` → `DualBipRouterSession` (two `BipTransport`s, no serial) |
| Oracle | `scripts/bip_bvll_oracle.py` (`routed-unicast`, `who-is-router`) |
| CI | `bip-qualify` workflow runs harness after G6; uploads `bip-bip-route-evidence` |
| rusty-bacnet pin | `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` (unchanged) |

## What this proves

- Clause 6 half-router path between **two distinct B/IP networks** in Linux netns.
- Exact-count matrix: A→B and B→A routed unicast; Who-Is-Router observe.
- Durable `result.json` + `SHA256SUMS` under `DBR_ROUTE_EVIDENCE_DIR`.
- `product_g7_g8_bip_mstp` field in result is always **OPEN**.
- Harness uses **distinct UDP ports** (47808 / 47809): upstream `BipTransport` binds
  `INADDR_ANY` with `SO_REUSEADDR`, so two same-port BIP sockets in one netns steal
  ingress (asymmetric false negatives).

## Explicit non-claims

- Not BIP↔MS/TP product routing (that still needs `--route-enable` + lab serial).
- G7/G8 ledger rows stay **Open** until BIP↔MS/TP bench evidence.
- Ordinary boot remains fail-closed; no `router.enabled=true` in TOML.
- Upstream may not expose forwarding counter aggregates — document gap if counts stay 0 on DUT report.
