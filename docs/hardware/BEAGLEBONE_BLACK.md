---
title: BeagleBone Black (follow-up)
parent: Hardware
nav_order: 2
permalink: /hardware/beaglebone-black/
---

# BeagleBone Black — follow-up target (not started)

M0 ships x86_64 (QEMU) and Raspberry Pi 3/4/5 images only. A **BeagleBone Black**
Buildroot target is intentionally **not-started** in the M0 management closeout.

## Intended later work (separate PR)

1. Confirm pinned Buildroot **2026.05.2** contains `beaglebone_defconfig` and read
   the board README before guessing artifact names.
2. Optional `workflow_dispatch` target `beaglebone` (32-bit ARMv7 hard-float),
   same `routerd` + React assets, Dropbear SSH recovery, manifests/checksums/legal-info.
3. Prove `AtomicU64` and dependencies compile for that triple; handle `libatomic`
   explicitly if required — never silently weaken counters.
4. Keep BBB out of mandatory PR CI until build time and reliability are known.

## Future MS/TP wiring (docs only — not implemented)

| Path | Notes |
| --- | --- |
| A | Existing Waveshare USB-to-RS485-C at a USB endpoint |
| B | Preferred later: expansion-header UART + isolated 3.3 V RS-485 transceiver with kernel RS-485 direction control |
| C | PRU-based MS/TP timing is research only and must be driven by on-wire evidence |

Do not flash physical media from CI. Do not claim hardware boot until demonstrated.
