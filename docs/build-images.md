---
title: Build images
layout: default
nav_order: 4
permalink: /build-images/
---

# Build appliance images

The
[**build-os**](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml)
workflow reads [`.github/workflows/matrix.json`](https://github.com/bbartling/diy-bacnet-router/blob/master/.github/workflows/matrix.json)
(HA-inspired) and runs `scripts/build-image.sh` per target.

| Target | Boot class | Primary artifacts | CI smoke |
| --- | --- | --- | --- |
| `x86_64` | QEMU / UEFI-ish | `bzImage`, `rootfs.ext2`, `rootfs.iso` | **Hard** `qemu-smoke.sh` |
| `generic_aarch64` | QEMU virt (UEFI-class portable ARM) | `Image`, `rootfs.ext2` | `qemu-aarch64-smoke.sh` (soft until hardened) |
| `rpi3_64` / `rpi4_64` / `rpi5_64` | U-Boot + Pi firmware | `sdcard.img` | Build + checksums only |

Every `images/` tree also includes `SHA256SUMS`, `build-manifest.json`,
`ARTIFACT_README.txt`, host Rust version stubs, and `legal-info.tar.xz`.

## Artifact names (Actions)

| Artifact | Contents |
| --- | --- |
| `dbr-<target>-<sha>-qemu` | Minimal smoke set + README + checksums (x86 / generic-aarch64) |
| `dbr-<target>-<sha>-images` | Full `images/` directory |
| `diy-bacnet-router-<target>-diagnostics-<run_id>` | Build/QEMU logs on failure |

Accept and boot locally:

```bash
gh run download <run_id> -n "dbr-x86_64-<sha>-qemu" -D /tmp/dbr-x86
bash scripts/accept-gh-image-artifact.sh /tmp/dbr-x86
```

## What you download today vs later

| Today | Goal (milestone **M8**) |
| --- | --- |
| Actions **artifacts** (short retention) | Versioned **GitHub Releases** |
| Manual download from a workflow run | Stable docs links to `…/releases/download/vX/…` |

Install path chooser: [Installation]({{ site.baseurl }}/installation/).

## Soft-OPEN — named cheap SBCs

HA ships ODROID / Khadas as **named** boards, not via `generic-aarch64`.
diy-bacnet-router does the same: Rockchip / Allwinner / Amlogic flash images are
**not** this tip. Track as Soft-OPEN “SBC board train.”

## What is inside the image?

See the beginner [Buildroot recipe]({{ site.baseurl }}/learn/buildroot-recipe/)
(eudev, Dropbear, `routerd`, USB-serial drivers, init as user `dbr`).
Reference RS-485: **Waveshare USB TO RS485 (C)**.

## Buildroot pin

[`config/buildroot-lock.toml`](https://github.com/bbartling/diy-bacnet-router/blob/master/config/buildroot-lock.toml):

- **2026.05.2**
- Host Rust inside Buildroot recorded in each `build-manifest.json`

## Optional local rebuild

[VMware Buildroot lab]({{ site.baseurl }}/operations/local-buildroot-vm/) —
[x86 live ISO]({{ site.baseurl }}/operations/x86-live-iso/).
