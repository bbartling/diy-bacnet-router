---
title: How we test
parent: Learn
nav_order: 4
permalink: /learn/testing/
---

# How we test (honest gates)

Testing is split so a green CI job never pretends to be a live RS-485 trunk.

## Three layers

| Layer | What it proves | Where |
| --- | --- | --- |
| **Software / CI** | Format, clippy, unit/integration tests, dual-B/IP netns, repository contracts | GitHub Actions `ci`, `bip-qualify` |
| **Image / QEMU** | Buildroot rootfs boots, `/healthz` answers, manifests + SHA256 | Actions `build-os` |
| **Lab / source** | Real USB RS-485, token traffic, routed ReadProperty on the bench | Evidence under `docs/evidence/` (Mint dual-boot lab) |

{: .warning }
**Source PASS ≠ exact-image PASS.** Running `routerd` from a Pi with `cargo`
built binaries is not the same gate as flashing a Buildroot `sdcard.img` and
repeating the oracle (**milestone M4**).

## Gate ledger

The living checklist is [TESTING.md on GitHub](https://github.com/bbartling/diy-bacnet-router/blob/master/docs/TESTING.md)
(G1–G11). Summary for readers:

- **G1** — x86 QEMU `/healthz` (image)
- **G6** — BACnet/IP qualify with an independent BVLL oracle (netns)
- **G7 / G8** — routed unicast and router network messages — **source** evidenced; **exact-image** still open
- **G9+** — faults, BBMD, segmentation, certification — open or not claimed

## Notable evidence packs

| Topic | Folder / note |
| --- | --- |
| Isolated two-Pi routing | `docs/evidence/SOURCE_G7_G8_*` |
| Shared trunk ReadProperty | `docs/evidence/PHASE2_FEC_VIA_DIY_*` |
| Dual-B/IP CI | `docs/evidence/M3_DUAL_BIP_NETNS.md` |
| Local-delivery drain / BIP↔loopback MS/TP | `docs/evidence/PR_A_ROUTE_SESSION_DRAIN.md` |
| Image eudev + USB-serial P0 | `docs/evidence/PHASE4B_IMAGE_P0_EUDEV_USB_SERIAL.md` |

## What beginners should remember

1. Fail-closed by default — forwarding off until you opt in on a controlled bench.
2. BIP test clients must not share the router’s UDP bind host/IP.
3. Use `/dev/serial/by-id/...` and check USB latency on FTDI adapters.
4. Read the evidence folder before claiming a product gate PASS.
