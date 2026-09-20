# Soft-OPEN — named cheap SBC board train

**Status:** Soft-OPEN (not this tip)  
**Date:** 2026-09-20  
**Context:** HA-inspired image matrix tip adds `generic_aarch64` (QEMU virt /
rare UEFI aarch64 only).

## Why Soft-OPEN

Home Assistant OS ships ODROID / Khadas / Pi as **named** U-Boot + DTB boards.
`generic-aarch64` there is explicitly a **VM / UEFI** path ([ADR-0015](https://github.com/home-assistant/architecture/blob/master/adr/0015-home-assistant-os.md)).

diy-bacnet-router follows the same honesty rule: do **not** claim Orange Pi,
Rockchip, Allwinner, or Amlogic flash support via `generic_aarch64`.

## Later tip (when a SKU is chosen)

1. Pick one board ID (e.g. a specific ODROID or Rock 5) with documented UART/USB.
2. Add Buildroot defconfig + U-Boot + DTB + genimage `sdcard.img` (or vendor layout).
3. Extend `.github/workflows/matrix.json` + `build-image.sh`.
4. Lab flash soak evidence under `docs/evidence/`.

Until then: use Pi 3/4/5 images, x86 QEMU, or `generic_aarch64` in QEMU only.
