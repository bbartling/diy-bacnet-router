# BFR design reference (architecture only)

Public-domain C++ BACnet/IP firewall-router (**BFR** ~0.2) is useful as a
**mental model** for multi-adapter routing, hop-count handling, filter pipelines,
and optional BBMD. This repository does **not** port BFR C++ and must **not**
vendor or paste ASHRAE standards text.

## Hard rules

- Do **not** copy BFR source into this tree.
- Do **not** ingest or redistribute `ASHRAE-bacnet-spec.txt` or any standards PDF/text.
- Prefer pinned upstream **rusty-bacnet** public APIs; application policy stays here.
- Ordinary DBR boot remains fail-closed; BBMD/FDR stay reserved until named gates pass.

## Module map (conceptual)

| BFR idea (external) | DIY BACnet Router / rusty-bacnet analogue |
| --- | --- |
| Multi-adapter `BACnetRouter` | `bacnet_network::router::BACnetRouter` + `RouterPort` via `rusty-bacnet-adapter` (`ApplianceRouterSession`, `DualBipRouterSession`) |
| Hop-count decrement on forward | Upstream router forward path (observe in loopback / dual-B/IP netns tests) |
| Network-message vs APDU split | Upstream NPDU decode; oracle Who-Is-Router observe in CI |
| Filter / firewall pipeline | **Future lab** management/policy layer — not in data plane today |
| BBMD / foreign device | Config keys reserved (`bbmd_enabled`); stack APIs unpinned for product claim — **M6 optional later** |
| Management / UI | Original Axum + React management plane (not BFR HTML) |

## What DBR already proves in software

- Dual B/IP Clause 6 forward in Linux netns (`--route-bip-bip`) without serial.
- Opt-in BIP↔MS/TP session (`--route-enable`) for lab — product G7/G8 still OPEN.
- Fail-closed management writes; auth/session skeleton only.

## What remains out of scope here

- Porting BFR packet filters as C++.
- Claiming BTL / Clause 9 / BBMD from BFR’s age or docs.
- Using BFR as a substitute for Waveshare MS/TP bench evidence.

Operator-held BFR trees (if any) stay **outside** this git repository.
