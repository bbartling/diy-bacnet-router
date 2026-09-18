---
title: Buildroot recipe
parent: Learn
nav_order: 4
permalink: /learn/buildroot-recipe/
---

# Buildroot recipe — what is in the appliance OS?

[Buildroot](https://buildroot.org/) cross-compiles a **tiny Linux userspace and
kernel** from a locked version of the Buildroot tree. This project does **not**
ship a general-purpose distro (no apt, no desktop). The image is optimized for:

1. running one BACnet router daemon with predictable serial timing
2. a small management web UI on the LAN
3. SSH for host networking and config files

## Pins

| Item | Where |
| --- | --- |
| Buildroot version / commit | `config/buildroot-lock.toml` (currently **2026.05.2**) |
| rusty-bacnet git SHA | `config/upstream-lock.toml` (baked into `routerd`) |
| Project Rust toolchain (CI) | `rust-toolchain.toml` (**1.93.0**) |

CI workflow:
[build-os](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml)
→ `scripts/build-image.sh`.

## Layers (mental model)

```text
┌─────────────────────────────────────────┐
│  React static UI  (/usr/share/.../web)  │
├─────────────────────────────────────────┤
│  diy-bacnet-router (routerd)            │  Rust — BACnet + Axum management
├─────────────────────────────────────────┤
│  Dropbear SSH · eudev · CA certs · tzdata│  minimal services
├─────────────────────────────────────────┤
│  Linux kernel (+ USB-serial fragment)   │  FTDI / CH34x for RS-485 dongles
├─────────────────────────────────────────┤
│  rootfs.ext2 / rootfs.iso / sdcard.img  │  board packaging
└─────────────────────────────────────────┘
```

## Userspace packages (common fragment)

From `buildroot-external/fragments/common.config`:

| Package / setting | Why it is there |
| --- | --- |
| `diy-bacnet-router` | The appliance binary + config + web assets + init script |
| **Dropbear** | SSH so you set IP/routes with normal Linux tools |
| **eudev** | Runs udev rules (serial by-id helpers, FTDI latency) |
| CA certificates / tzdata | TLS trust store basics and correct timestamps |
| ext2/4 rootfs | Simple durable root for QEMU and lab |

Device creation is forced to **eudev** (not mdev-only) so rules under
`/etc/udev/rules.d/` actually run.

## Kernel USB-serial fragment

`buildroot-external/fragments/linux-usb-serial.fragment` enables USB serial
support aimed at lab RS-485 dongles (FTDI and WCH CH341/CH343). Unknown Kconfig
symbols are dropped by `olddefconfig` — confirm with `udevadm` on hardware.

## Init and privilege

- SysV script `S80diy-bacnet-router` starts the daemon with
  `start-stop-daemon` as user **`dbr`** (not root).
- Config lives at `/etc/diy-bacnet-router/router.toml`.
- Web UI is static files under `/usr/share/diy-bacnet-router/web`.
- FTDI latency helper: `/usr/sbin/set-ftdi-latency.sh` +
  `/etc/udev/rules.d/99-ftdi-latency.rules`.

## What is intentionally missing

- No BBMD / foreign-device product claims until evidenced
- No browser config writes until auth gates pass
- No full desktop, docker, or package manager
- No claim that QEMU smoke equals a live MS/TP trunk

## Download story (TODO — milestone M8)

Today you download **workflow artifacts** from Actions (short retention).
The product goal is:

1. `build-os` publishes a **GitHub Release** per version tag
2. this docs site links straight to
   `https://github.com/bbartling/diy-bacnet-router/releases/download/vX/...`
3. you flash `sdcard.img` / boot `rootfs.iso` without rebuilding Buildroot locally

Until then, treat local VMware Buildroot loops as **optional debug**, not the
install path. See [Build images]({{ site.baseurl }}/build-images/).
