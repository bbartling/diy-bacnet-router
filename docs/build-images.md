---
title: Build images
layout: default
nav_order: 3
---

# Build appliance images

GitHub Actions workflow
[**build-os**](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml)
runs `scripts/build-image.sh` for:

| Target | Artifact |
| --- | --- |
| `x86_64` | `bzImage`, `rootfs.ext2` + **QEMU smoke** |
| `rpi3_64`, `rpi4_64`, `rpi5_64` | `sdcard.img` |

## Buildroot pin

Pinned in [`config/buildroot-lock.toml`](https://github.com/bbartling/diy-bacnet-router/blob/master/config/buildroot-lock.toml):

- **2026.05.2** — latest stable bugfix line (Aug 2026)
- Host Rust inside Buildroot: **1.96.1**

Every build publishes SHA256SUMS, legal-info, and `build-manifest.json`.

## Local lab

Reproduce CI failures on an **Ubuntu guest in VMware**, SSH from Windows:

[VMware Buildroot lab]({{ site.baseurl }}/operations/local-buildroot-vm/)

```powershell
.\scripts\vm-ensure.ps1 -DebugBuild
```

QEMU smoke uses `-snapshot` so checksum verification stays valid after boot.
