---
title: Waveshare RS-485 (C)
parent: Hardware
nav_order: 1
permalink: /hardware/waveshare-rs485-c/
---

# Waveshare USB TO RS485 (C) reference profile

The first supported lab adapter is the
[Waveshare USB TO RS485 (C)](https://docs.waveshare.com/USB_TO_RS485_C), SKU
34620. Waveshare documents an FT232RNL USB interface, an isolated RS-485 field
side, A+/B-/GND terminals, hardware-automatic direction control, Linux support,
and an onboard 120 Ω balancing resistor.

## Consequences for this project

- Use normal 8N1 serial mode. Do **not** enable Linux `TIOCSRS485`, RTS toggling,
  GPIO DE/RE, or another direction-control scheme at the same time.
- Use the FTDI serial number through `/dev/serial/by-id/...`; `/dev/ttyUSB0`
  may change after reboot or replug.
- A+ connects to A/+, B- connects to B/-. Connect field-side GND to the
  controller's isolated reference/common when the device manuals and measured
  potentials permit it. This is not the host USB ground.
- The onboard resistor counts as one 120 Ω endpoint termination. The operator's
  unpowered measurement of roughly 130 Ω across A+/B- is consistent with a
  nominal onboard termination plus component/meter tolerance.
- Do not assume the termination provides fail-safe bias. Termination and bias
  are different electrical functions and must be verified independently.
- Do not open, drill, or modify the sealed adapter as part of this project.

## Approved initial topologies

### Isolated two-adapter bench

```text
Linux USB                         Linux USB
Waveshare C #1                   Waveshare C #2
  A+  =============================  A+
  B-  =============================  B-
  GND =============================  GND
 [120 Ω]                         [120 Ω]
```

With power removed from both field sides, the complete bus should measure near
60–65 Ω across A+/B- because the two onboard endpoint resistors are in parallel.
This topology is useful for raw serial and isolated MS/TP tests.

### Existing live MS/TP trunk

The C adapter must be treated as a **terminated endpoint**, not a transparent
mid-span probe. If the existing two physical ends are already terminated,
adding this adapter in the middle creates a third termination and changes the
bus load. For a live BAS/controller trunk, either:

1. temporarily make the C adapter one physical end and ensure only the opposite
   end retains termination; or
2. use an adapter whose termination can be verifiably disabled for a short,
   topology-compliant tap.

Do not transmit merely because raw bytes are visible. The passive gate must
decode valid complete frames and tokens first.

## Ubuntu inventory commands

```bash
ls -l /dev/serial/by-id/
serial=/dev/serial/by-id/usb-FTDI_REPLACE_ME
udevadm info --query=property --name="$(readlink -f "$serial")" \
  | grep -E '^(ID_VENDOR_ID|ID_MODEL_ID|ID_SERIAL|ID_USB_DRIVER)='
readlink -f "$serial"
```

Record the actual USB IDs, serial, kernel driver, by-id link, kernel version,
baud, topology, powered-off resistance and reference wiring in every hardware
artifact. Do not hard-code a VID/PID based only on marketing material; capture
what Linux reports for each physical unit.

If the driver exposes an FTDI latency setting, record it:

```bash
serial=/dev/serial/by-id/usb-FTDI_REPLACE_ME
device="$(basename "$(readlink -f "$serial")")"
cat "/sys/bus/usb-serial/devices/${device}/latency_timer" 2>/dev/null || true
```

Changing sysfs or `setserial` settings requires an explicit operator action and
must be reported as part of the test environment. A low latency value is not a
substitute for correct frame-stream handling in the stack.

## Preflight checklist

- [ ] USB identity and by-id path recorded
- [ ] No other process owns the tty (`fuser -v <by-id-path>`)
- [ ] Adapter field side unpowered while measuring resistance
- [ ] Exactly two endpoint terminations in the complete topology
- [ ] Bias/reference arrangement verified
- [ ] A+/B- polarity confirmed from device manuals
- [ ] Baud, MAC, Max_Master and Max_Info_Frames agreed
- [ ] Passive valid-frame/token gate passes before TX
- [ ] Existing supervisory workstation remains healthy
