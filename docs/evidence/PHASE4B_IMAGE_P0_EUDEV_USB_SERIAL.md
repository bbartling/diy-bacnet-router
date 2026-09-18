# Phase 4b — image P0: eudev + USB-serial (Windows / Buildroot)

## Scope

Windows+VMware image path only. Mint owns live trunk / FEC READ-ONLY evidence.
Do **not** relabel Mint source G7/G8 or FEC RP as appliance-image PASS.

## Pin

| Item | Value |
|------|-------|
| rusty-bacnet | `acbf7baefe69d05f2368763dcc659d68e4bc114c` |
| build-os manifest assert | same SHA (was stale `7e0d13a…`) |

## What landed in tree

| P0 piece | Change |
|----------|--------|
| eudev | `BR2_PACKAGE_EUDEV` + `BR2_ROOTFS_DEVICE_CREATION_DYNAMIC_EUDEV` in `common.config`; `build-image.sh` strips competing device-creation choices before `olddefconfig` |
| Kernel USB-serial | `buildroot-external/fragments/linux-usb-serial.fragment` (FTDI_SIO, CH341, CH343) wired via absolute `BR2_LINUX_KERNEL_CONFIG_FRAGMENT_FILES` |
| udev rules | FTDI `0403:6001` latency_timer=1; WCH `1a86:55d3` / `1a86:7523` RUN `set-ftdi-latency.sh` |
| Helper | `set-ftdi-latency.sh` exits 0 when no sysfs nodes (udev-safe) |

## Still OPEN

- Exact-image flash/QEMU prove of by-id bind for a physical CH343 (needs appliance boot + adapter)
- NIC driver matrix, `/data` persistence, SSH key provisioning, CD-ROM smoke hard-fail
- Product G7/G8 on **image** binary (Mint source PASS is not image PASS)
- VMware guest SSH (`127.0.0.1:2222`) was refused during this Windows session — guest sync deferred

## Operator check (after flash)

```bash
ls -l /dev/serial/by-id/
udevadm info --query=property --name="$(readlink -f /dev/serial/by-id/usb-*-if00)" | grep -E 'ID_VENDOR_ID|ID_MODEL_ID|ID_USB_DRIVER'
cat /sys/bus/usb-serial/devices/*/latency_timer
```
