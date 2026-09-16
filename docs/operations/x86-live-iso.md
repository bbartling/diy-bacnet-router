---
title: x86 live ISO (QEMU -cdrom + disposable VMware)
layout: default
parent: Build images
nav_order: 4
---

# x86_64 live ISO

`scripts/build-image.sh x86_64` produces:

| Artifact | Use |
| --- | --- |
| `bzImage` + `rootfs.ext2` | Existing CI QEMU `-kernel` smoke |
| `rootfs.iso` | Live ISO — CI `scripts/qemu-cdrom-smoke.sh` (`-cdrom`) |

Safe defaults: management plane only, `ready_to_route=false`, no RS-485 required.
Missing serial yields a useful error only after opt-in `--route-enable`.

## CI

`build-os` runs kernel smoke then:

```bash
bash scripts/qemu-cdrom-smoke.sh "$RUNNER_TEMP/dbr-buildroot/output/x86_64/images"
```

## Disposable VMware boot (not the builder VM)

1. Create a **new** disposable VM (BIOS/UEFI, 1 GiB RAM, no shared OT NIC with the Pi lab).
2. Attach `rootfs.iso` as the CD/DVD and boot from it.
3. Confirm `/healthz` on the management bind (`dbr.bind=0.0.0.0:8080` on the kernel cmdline when needed).
4. Do **not** advertise DNET 2001 on the same LAN as the live Pi router unless the Pi session is stopped.
5. Tear down the VM after smoke; physical USB route is optional C2 and needs Ben USB/bridge approval.

## Lab two-Pi image notes

Pi `rpi3_64` `sdcard.img` ships `/etc/diy-bacnet-router/router.lab-two-pi.example.toml`
(DNET **2001**, route disabled) and FTDI `latency_timer=1` udev rule (issue #66).
Flash media path remains Ben-confirmed only.
