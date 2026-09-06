# M2A — BACnet/IP port qualification (netns)

| Field | Value |
| --- | --- |
| Milestone | **M2A only** (M2B MS/TP remains open) |
| Topology | `dbr-bip-a` 192.0.2.2/24 ↔ veth ↔ `dbr-bip-b` 192.0.2.1/24 |
| Upstream pin | `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` |
| Harness | `scripts/bip-qualify-netns.sh` |
| Opt-in | `diy-bacnet-router --bip-qualify` (ordinary boot does **not** start B/IP) |
| Merge SHA | `6d36e593430f722563aabd87cd40d455b7698c52` |

## Claims

- One real B/IP UDP socket via public `BipTransport::start`
- Observed `bip_rx_packets` ≥ 1 under peer probes
- `ready_to_route=false`; forwarding counters exactly 0
- MS/TP not qualified (`mstp_link=disabled`, token counters remain 0)
- Upstream does not expose malformed/drop counters — documented unavailable (not synthesized)

## Non-claims

- No `BACnetRouter` / NPDU forwarding
- No physical MS/TP (M2B)
- No BBMD/FDR

## CI

Trusted GitHub-hosted Ubuntu job: `.github/workflows/bip-qualify.yml`

## Post-merge Actions

- bip-qualify run ID (PR): `34003661661`
- CI run ID (PR): `34003661593`
- build-os run ID (PR x86): `34003661633`
- Tip master CI: `34007152133`
- Tip master bip-qualify: `34007152084`
- Tip master build-os (full matrix): `34007152106`
- Local VM Buildroot/qemu-ui: **BLOCKED** — SSH `127.0.0.1:2222` refused (Actions covered image + netns)
