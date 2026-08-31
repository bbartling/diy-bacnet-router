# Linux network and serial operations

## Separation of concerns

The operating system configures Ethernet addressing, prefix, gateway, DNS,
hostname and time synchronization. `router.toml` tells the BACnet process which
configured interface/address and BACnet network number to use. Do not create a
second hidden network-management system out of environment variables.

For development Ubuntu hosts, continue using Netplan or NetworkManager as the
host already expects. For the Buildroot image, keep the selected init/network
mechanism in the board overlay and version its defaults. The application must
not invoke `ip addr`, rewrite `/etc/network/interfaces`, or change a default
route during normal startup.

## Supported deployment overrides

- `DBR_CONFIG`: application TOML path
- `DBR_BIND`: temporary management listen override
- `DBR_WEB_ROOT`: compiled React asset path
- `RUST_LOG`: structured logging filter

Baud, MAC, Max_Master, network numbers and the serial by-id path belong in the
validated TOML so one effective configuration can be inspected and archived.

## Serial permissions

The service runs without root. Grant only its service account access to the
specific serial device using the image's `dialout` group or a narrowly scoped
udev/mdev rule. Do not run the whole router as root to avoid a permission error.

Before opening the port:

```bash
serial=/dev/serial/by-id/usb-FTDI_REPLACE_ME
test -e "$serial"
readlink -f "$serial"
fuser -v "$serial" || true
```

One process owns one tty. A busy port is a startup error with owner diagnostics,
not permission to kill the existing process.

## Timing baseline

Record `uname -a`, high-resolution timer/PREEMPT configuration, CPU governor,
USB topology and adapter latency with every qualification. Standard Ubuntu or
Raspberry Pi kernels may be sufficient when measured under load; PREEMPT_RT is
an experiment to compare worst-case behavior, not a feature flag that proves
MS/TP timing.

