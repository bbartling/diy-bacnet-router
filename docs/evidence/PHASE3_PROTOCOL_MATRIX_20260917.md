# Phase 3 — BASRT protocol parity matrix (Mint LAN / tip `7e0d13a`)

Living matrix. Update rows as PRs land. Do **not** claim PASS without evidence path.

| Row | Capability | Upstream tip (`7e0d13a`) | Appliance wiring | LAN evidence | Status |
|-----|------------|---------------------------|------------------|--------------|--------|
| P3-1 | Standard MS/TP BIP↔MSTP route | Yes (pin) | `--route-enable` | Phase1 five-point | **PASS** |
| P3-2 | Local router Device (Who-Is/I-Am/RP) | Server objects exist upstream | Not in routerd yet | — | **OPEN** (phase 2b) |
| P3-3 | Extended frames type 32/33 + COBS | Verify codec at tip — not claimed | Not exposed | — | **OPEN** |
| P3-4 | Segmented APDU forward | App-layer; router must not mangle | Needs tests | — | **OPEN** |
| P3-5 | §9.7 ReplyPostponed | Upstream MS/TP master | Needs wire proof | — | **OPEN** |
| P3-6 | BBMD / FDR | Upstream BBMD paths exist | Config flags present (`bbmd_enabled`) | — | **OPEN** |
| P3-7 | Secondary BIP / Ethernet | Partial upstream | UI gray | — | **DEFER** |

## Next Mint PR slices (after FEC trunk)

1. Router Device endpoint (2b) — single socket owner demux.
2. Extended-frame RX/TX policy + golden vectors (upstream-first if missing).
3. Segmented forward oracle both directions.
4. BBMD enablement behind explicit commission + loop guards.

## Pin discipline

Always `config/upstream-lock.toml` == Cargo.toml rev == `UPSTREAM_REVISION`. Watch `jscott3201/rusty-bacnet@dev` daily.
