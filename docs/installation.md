---
title: Installation
layout: default
nav_order: 3
permalink: /installation/
---

# Installation

Inspired by [Home Assistant OS board layout](https://www.home-assistant.io/installation/)
and [home-assistant/operating-system](https://github.com/home-assistant/operating-system)
(ADR-0015): **named boards** for real silicon, **generic UEFI/QEMU** for VMs — not
one “any ARM SBC” flash image. **No Windows** path.

Ordinary appliance boot is **fail-closed** (management only; no BACnet forwarding).

## Choose a path

| Path | Target id | What you download | How to run |
| --- | --- | --- | --- |
| QEMU / lab VM (x86) | `x86_64` | `bzImage` + `rootfs.ext2` (+ checksums) | [`accept-gh-image-artifact.sh`]({{ site.baseurl }}/operations/local-buildroot-vm/) → `qemu-smoke.sh` |
| Live ISO (soft gate) | `x86_64` | `rootfs.iso` | `qemu-cdrom-smoke.sh` |
| Raspberry Pi 3 / 4 / 5 | `rpi3_64` / `rpi4_64` / `rpi5_64` | `sdcard.img` | Flash with Etcher / `dd` (physical soak still OPEN) |
| Generic AArch64 VM / UEFI | `generic_aarch64` | `Image` + `rootfs.ext2` | QEMU aarch64 virt — **not** Orange Pi / Rockchip |
| Native Linux (dev / lab) | — | cargo build / source `routerd` | Workstation or Pi OS — not the Buildroot appliance |

## Honest non-claims

- **`generic_aarch64` ≠ cheap IoT SBC.** Rockchip / Allwinner / Amlogic need a
  **named** U-Boot + DTB board (Soft-OPEN later), same as HA’s ODROID/Khadas model.
- No Windows / Hyper-V install path.
- Live MS/TP product stress stays **38400** baud (Waveshare **C** reference).
  See [AGENTS.md](https://github.com/bbartling/diy-bacnet-router/blob/master/AGENTS.md)
  lab baud hold.

## Getting images today

1. Open a green [**build-os**](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml) run.
2. Prefer the **`-qemu`** artifact for x86 / generic-aarch64 smoke (small zip).
3. Or download the full **`-images`** artifact and follow `ARTIFACT_README.txt` inside.

Milestone **M8** will publish versioned GitHub Releases; until then Actions artifacts
are the delivery channel.

## Related

- [Build images]({{ site.baseurl }}/build-images/) — artifact matrix + Buildroot pin
- [Buildroot recipe]({{ site.baseurl }}/learn/buildroot-recipe/) — what is inside the OS
- [Local Buildroot / QEMU lab]({{ site.baseurl }}/operations/local-buildroot-vm/)
