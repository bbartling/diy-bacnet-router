---
title: Linux and MS/TP timing
parent: Learn
nav_order: 2
permalink: /learn/linux-mstp-timing/
---

# Linux, USB serial, and MS/TP timing

MS/TP assumes a quiet, predictable serial path. On Linux that means: the right
driver, a stable device path, and low USB-UART latency.

## Stable device names

USB serial nodes renumber (`ttyUSB0` → `ttyUSB1`) after replug or reboot.
Persist only:

```text
/dev/serial/by-id/usb-...
```

Record vendor/product IDs and the by-id link in lab notes. See
[Waveshare USB TO RS485 C]({{ site.baseurl }}/hardware/waveshare-rs485-c/).

## Why USB latency matters

Many FTDI adapters default to a **16 ms** `latency_timer`. For BACnet confirmed
services over a router hop, that can turn into multi-second tool timeouts.
This project ships:

- a small helper (`scripts/set-ftdi-latency.sh`) that sets `latency_timer` to **1 ms**
- udev rules installed into the Buildroot image for common FTDI / WCH adapters

On a live lab host you can inspect:

```bash
ls -l /dev/serial/by-id/
cat /sys/bus/usb-serial/devices/*/latency_timer
```

## Baud and topology

- Allowed bauds in config: 9600…115200; lab default **38400**.
- Exactly **one** process should own the tty.
- Endpoint termination (often 120 Ω) must match the physical trunk; do not add a
  third terminated adapter mid-span without a plan.
- Passive decode / observe before transmitting on an unknown trunk.

## BIP oracle host vs router host

When testing routed traffic, the BACnet/IP client (oracle, Workbench, bacpypes)
must bind on a **different host or IP** than the router’s BACnet/IP socket.
Same-host or macvlan-on-parent demux mistakes look like “Timeout” or
“unknown route” even when the MS/TP side is fine.

## Further reading

- [Buildroot recipe]({{ site.baseurl }}/learn/buildroot-recipe/) — eudev + USB-serial in the image
- [How we test]({{ site.baseurl }}/learn/testing/)
- Issue [#66](https://github.com/bbartling/diy-bacnet-router/issues/66) — timing / serial ownership follow-ups
