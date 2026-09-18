---
title: Build images
layout: default
nav_order: 4
---

# Build appliance images

The
[**build-os**](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml)
workflow runs `scripts/build-image.sh` and produces:

| Target | Artifact |
| --- | --- |
| `x86_64` | `bzImage`, `rootfs.ext2`, **`rootfs.iso`** + QEMU smoke |
| `rpi3_64`, `rpi4_64`, `rpi5_64` | `sdcard.img` + manifest / checksums |

## What you download today vs later

| Today | Goal (milestone **M8**) |
| --- | --- |
| Actions **artifacts** (short retention) | Versioned **GitHub Releases** |
| Manual download from a workflow run | Stable docs links to `…/releases/download/vX/…` |

Until M8 ships, open the latest green `build-os` run on
[Actions](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml)
and download the artifact for your board.

## What is inside the image?

See the beginner [Buildroot recipe]({{ site.baseurl }}/learn/buildroot-recipe/)
(eudev, Dropbear, `routerd`, USB-serial drivers, init as user `dbr`).

## Buildroot pin

[`config/buildroot-lock.toml`](https://github.com/bbartling/diy-bacnet-router/blob/master/config/buildroot-lock.toml):

- **2026.05.2**
- Host Rust inside Buildroot recorded in each `build-manifest.json`

## Optional local rebuild

Reproduce CI on an Ubuntu guest (VMware) only when debugging Buildroot —
not required for normal use once Releases exist:

[VMware Buildroot lab]({{ site.baseurl }}/operations/local-buildroot-vm/)

Live ISO notes: [x86 live ISO]({{ site.baseurl }}/operations/x86-live-iso/).
