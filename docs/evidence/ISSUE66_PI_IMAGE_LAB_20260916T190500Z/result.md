# Phase B — rpi3_64 lab commissioning

## Delivered in tree

- `config/router.lab-two-pi.example.toml` — DNET **2001**, MAC1, Max_Master=2, MIF=2, `router.enabled=false`
- Package install → `/etc/diy-bacnet-router/router.lab-two-pi.example.toml`
- FTDI `latency_timer=1` udev rule + `set-ftdi-latency.sh` on appliance image
- Buildroot `rpi3_64` defconfig path validated (`olddefconfig`)

## Flash / 30m image oracle

| Step | Status |
| --- | --- |
| Cross-build `sdcard.img` | OPEN — run `scripts/build-image.sh rpi3_64` on Ubuntu Buildroot VM / CI |
| Ben confirms `/dev/sdX` | **OPEN** (required before any flash) |
| First boot USB off trunk | OPEN |
| `--mstp-passive` → route-enable → A4 oracle | OPEN |

Do not flash until Ben confirms media device node.
