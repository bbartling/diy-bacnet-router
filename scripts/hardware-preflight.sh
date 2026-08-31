#!/usr/bin/env bash
set -euo pipefail

serial="${SERIAL_BY_ID:?SERIAL_BY_ID must be set}"
baud="${MSTP_BAUD:-38400}"
case "$baud" in 9600|19200|38400|57600|76800|115200) ;; *) echo "unsupported baud" >&2; exit 2;; esac
case "$serial" in /dev/serial/by-id/*) ;; *) echo "use /dev/serial/by-id" >&2; exit 2;; esac
[[ -e "$serial" ]] || { echo "serial device does not exist: $serial" >&2; exit 1; }

mkdir -p captures
resolved="$(readlink -f "$serial")"
tty_name="$(basename "$resolved")"
if fuser "$serial" >/tmp/dbr-fuser.txt 2>&1; then
  echo "serial port is already owned" >&2
  cat /tmp/dbr-fuser.txt >&2
  exit 1
fi

latency="unknown"
if [[ -r "/sys/bus/usb-serial/devices/$tty_name/latency_timer" ]]; then
  latency="$(cat "/sys/bus/usb-serial/devices/$tty_name/latency_timer")"
fi

python3 - "$serial" "$resolved" "$baud" "$latency" > captures/hardware-inventory.json <<'PY'
import json, os, platform, subprocess, sys, time
by_id, resolved, baud, latency = sys.argv[1:]
props = subprocess.run(
    ["udevadm", "info", "--query=property", f"--name={resolved}"],
    check=False, text=True, capture_output=True,
).stdout.splitlines()
allowed = ("ID_VENDOR_ID=", "ID_MODEL_ID=", "ID_SERIAL=", "ID_USB_DRIVER=")
print(json.dumps({
    "schema_version": 1,
    "hardware_evidence": False,
    "operation": "inventory_no_tx",
    "timestamp_unix": int(time.time()),
    "project_git_sha": subprocess.run(["git", "rev-parse", "HEAD"], text=True, capture_output=True).stdout.strip(),
    "kernel": platform.release(),
    "architecture": platform.machine(),
    "serial_by_id": by_id,
    "serial_resolved": resolved,
    "baud_requested": int(baud),
    "latency_timer": latency,
    "udev": {line.split("=", 1)[0]: line.split("=", 1)[1] for line in props if line.startswith(allowed)},
    "adapter_profile": "waveshare-usb-to-rs485-c",
    "termination": "onboard-present",
}, indent=2))
PY

cat captures/hardware-inventory.json

