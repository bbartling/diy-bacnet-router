#!/usr/bin/env bash
# Boot the x86_64 live ISO with QEMU -cdrom (not -kernel).
# Expects images/rootfs.iso from scripts/build-image.sh x86_64.
set -euo pipefail

images="${1:-output/x86_64/images}"
iso="$images/rootfs.iso"
log="${QEMU_CDROM_LOG:-/tmp/diy-bacnet-router-qemu-cdrom.log}"
qemu_pid=""

if [[ ! -s "$iso" ]]; then
  echo "missing live ISO: $iso (build with scripts/build-image.sh x86_64)" >&2
  exit 2
fi

cleanup() {
  if [[ -n "$qemu_pid" ]]; then kill "$qemu_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT

qemu-system-x86_64 \
  -M pc -m 512 -smp 2 \
  -cdrom "$iso" \
  -boot d \
  -nic "user,model=virtio-net-pci,hostfwd=tcp:127.0.0.1:18081-:8080" \
  -nographic -no-reboot >"$log" 2>&1 &
qemu_pid=$!

for _ in $(seq 1 180); do
  if health_json="$(curl --fail --silent http://127.0.0.1:18081/healthz 2>/dev/null)"; then
    if python3 - "$health_json" <<'PY'
import json
import sys

health = json.loads(sys.argv[1])
assert health["status"] == "ok"
assert health["management_plane"] == "operational"
assert health["data_plane"] == "disabled"
assert health["ready_to_route"] is False
PY
    then
      echo "QEMU -cdrom management health PASS (data plane disabled; no default routing)"
      echo "Health JSON: $health_json"
      exit 0
    fi
  fi
  if ! kill -0 "$qemu_pid" 2>/dev/null; then
    echo "QEMU exited before ISO health became ready" >&2
    tail -n 200 "$log" >&2
    exit 1
  fi
  sleep 1
done

echo "Timed out waiting for QEMU -cdrom health" >&2
tail -n 200 "$log" >&2
exit 1
