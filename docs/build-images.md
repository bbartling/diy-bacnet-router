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
| `x86_64` | `bzImage`, `rootfs.ext2`, **`rootfs.iso`** + QEMU `-kernel` and **`-cdrom`** smoke |
| `rpi3_64`, `rpi4_64`, `rpi5_64` | `sdcard.img` (lab commissioning example + FTDI latency udev) |

Live ISO / disposable VMware notes: [x86 live ISO]({{ site.baseurl }}/operations/x86-live-iso/).


## Buildroot pin

Pinned in [`config/buildroot-lock.toml`](https://github.com/bbartling/diy-bacnet-router/blob/master/config/buildroot-lock.toml):

- **2026.05.2** — latest stable bugfix line (Aug 2026)
- Host Rust inside Buildroot: **1.96.1**

Every build publishes SHA256SUMS, legal-info, and `build-manifest.json`.

## Local lab

Reproduce CI failures on an **Ubuntu guest in VMware**, SSH from Windows:

[VMware Buildroot lab]({{ site.baseurl }}/operations/local-buildroot-vm/)

```powershell
.\scripts\vm-ensure.ps1 -Hypervisor vmware -DebugBuild
```

QEMU smoke uses `-snapshot` so checksum verification stays valid after boot.
