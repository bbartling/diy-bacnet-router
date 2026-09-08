# M3–M6 software path (path A) — implemented, gates OPEN

| Field | Value |
| --- | --- |
| Policy | Fast software landing; **no false PASS** on G7–G11 / physical M2B / BTL |
| Tip intent | Opt-in forwarding + M6 write refusal scaffold |
| rusty-bacnet pin | `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` (unchanged) |

## What landed in software

| Area | Surface | Ordinary boot |
| --- | --- | --- |
| M3 | `--route-enable` → `ApplianceRouterSession` (B/IP + MS/TP under `BACnetRouter`) | Still management-only; `router.enabled=true` rejected |
| M3 CI | Loopback A↔B unicast forward tests in `rusty-bacnet-adapter` | N/A |
| M4 CI | Loopback stop + rebind same networks | Hardware USB/NIC faults still open |
| M6 | `POST /api/config` → **403**; `GET /api/audit`; bounded `AuditLog` | Writes remain disabled |
| Caps | `bip_mstp_routing` / `management_writes` stay `BlockedByEvidence` | Honest |

## Explicit non-claims

- G7/G8/G9–G11 **not PASS**
- Physical M2B **not done**
- BBMD/FDR **not enabled** (config reserved; tip stack APIs unpinned)
- `ready_to_route` product claim remains **false** even during `--route-enable`
- No BFR C++ port; no ASHRAE standards text in this repo

## Lab unlock (not CI)

```bash
diy-bacnet-router --config /etc/diy-bacnet-router/router.toml --route-enable --qualify-secs 300
```

Requires real B/IP iface + `/dev/serial/by-id/...`. Prefer isolated lab nets only.
