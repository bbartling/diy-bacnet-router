#!/usr/bin/env bash
set -euo pipefail

images="${1:-output/x86_64/images}"
log="${QEMU_LOG:-/tmp/diy-bacnet-router-qemu.log}"
qemu_pid=""

cleanup() {
  if [[ -n "$qemu_pid" ]]; then kill "$qemu_pid" 2>/dev/null || true; fi
}
trap cleanup EXIT

qemu-system-x86_64 \
  -M pc -m 512 -smp 2 \
  -snapshot \
  -kernel "$images/bzImage" \
  -drive "file=$images/rootfs.ext2,if=virtio,format=raw" \
  -append "root=/dev/vda console=ttyS0 dbr.bind=0.0.0.0:8080" \
  -nic "user,model=virtio-net-pci,hostfwd=tcp::18080-:8080" \
  -nographic -no-reboot >"$log" 2>&1 &
qemu_pid=$!

for _ in $(seq 1 90); do
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
        echo "QEMU management health PASS (data plane disabled; service unprivileged)"
        echo "Health JSON: $health_json"
        echo "QEMU log evidence: $(grep -E 'Starting diy-bacnet-router: OK' "$log" | tail -n 1)"
        exit 0
      fi
    fi
  fi
  if ! kill -0 "$qemu_pid" 2>/dev/null; then
    echo "QEMU exited before health became ready" >&2
    tail -n 200 "$log" >&2
    exit 1
  fi
  sleep 1
done

echo "Timed out waiting for QEMU health and unprivileged service evidence" >&2
tail -n 200 "$log" >&2
exit 1
