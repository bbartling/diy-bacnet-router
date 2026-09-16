#!/usr/bin/env bash
# Set FTDI (and optionally all usb-serial) latency_timer to 1 ms.
# Requires root for /sys writes. Idempotent.
set -euo pipefail
target="${1:-1}"
shopt -s nullglob
found=0
for path in /sys/bus/usb-serial/devices/*/latency_timer; do
  found=1
  cur="$(cat "$path")"
  if [[ "$cur" != "$target" ]]; then
    echo "$target" >"$path"
    echo "set $path: $cur -> $target"
  else
    echo "ok $path=$cur"
  fi
done
if [[ "$found" -eq 0 ]]; then
  echo "no usb-serial latency_timer sysfs nodes present" >&2
  exit 1
fi
