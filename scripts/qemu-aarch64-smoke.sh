#!/usr/bin/env bash
# QEMU aarch64 virt smoke for generic_aarch64 appliance images.
# Same health contract as scripts/qemu-smoke.sh (x86).
set -euo pipefail

images="${1:-output/generic_aarch64/images}"
log="${QEMU_LOG:-/tmp/diy-bacnet-router-qemu-aarch64.log}"
qemu_pid=""

cleanup() {
  if [[ -n "$qemu_pid" ]]; then kill "$qemu_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT

kernel=""
for candidate in Image Image.gz vmlinux; do
  if [[ -s "$images/$candidate" ]]; then
    kernel="$images/$candidate"
    break
  fi
done
if [[ -z "$kernel" ]]; then
  echo "missing aarch64 kernel Image under $images" >&2
  exit 2
fi
test -s "$images/rootfs.ext2"

qemu-system-aarch64 \
  -M virt -cpu cortex-a53 -m 512 -smp 2 \
  -snapshot \
  -kernel "$kernel" \
  -drive "file=$images/rootfs.ext2,if=virtio,format=raw" \
  -append "root=/dev/vda console=ttyAMA0 dbr.bind=0.0.0.0:8080" \
  -netdev "user,id=net0,hostfwd=tcp:127.0.0.1:18080-:8080" \
  -device virtio-net-device,netdev=net0 \
  -nographic -no-reboot >"$log" 2>&1 &
qemu_pid=$!

for _ in $(seq 1 120); do
  if health_json="$(curl --fail --silent http://127.0.0.1:18080/healthz)"; then
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
      if grep -Eq 'Starting diy-bacnet-router: OK \(uid=[1-9][0-9]*\)' "$log"; then
        echo "QEMU aarch64 management health PASS (data plane disabled; service unprivileged)"
        echo "Health JSON: $health_json"
        echo "QEMU log evidence: $(grep -E 'Starting diy-bacnet-router: OK' "$log" | tail -n 1)"
        exit 0
      fi
    fi
  fi
  if ! kill -0 "$qemu_pid" 2>/dev/null; then
    echo "QEMU aarch64 exited early; log: $log" >&2
    tail -n 80 "$log" >&2 || true
    exit 1
  fi
  sleep 2
done

echo "QEMU aarch64 smoke timed out; log: $log" >&2
tail -n 80 "$log" >&2 || true
exit 1
