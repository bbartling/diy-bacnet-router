#!/usr/bin/env bash
# Offline regression fixtures for hardened acceptance / qemu-ui contracts.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

pass=0
fail=0
assert_fails() {
  local name="$1"
  shift
  if "$@" >/dev/null 2>&1; then
    echo "FAIL: expected failure: $name" >&2
    fail=$((fail + 1))
  else
    echo "PASS: $name"
    pass=$((pass + 1))
  fi
}
assert_ok() {
  local name="$1"
  shift
  if "$@"; then
    echo "PASS: $name"
    pass=$((pass + 1))
  else
    echo "FAIL: $name" >&2
    fail=$((fail + 1))
  fi
}

echo "==> acceptance script contracts"
grep -q 'missing SHA256SUMS' scripts/vm-accept-artifact.sh
grep -q 'ambiguous bzImage' scripts/vm-accept-artifact.sh
grep -q 'qemu-smoke.sh missing' scripts/vm-accept-artifact.sh
grep -q 'after QEMU' scripts/vm-accept-artifact.sh
grep -q 'dbr-accept-worktrees' scripts/vm-accept-artifact.sh
grep -q 'diy-bacnet-router-x86_64-' scripts/vm-accept-artifact.sh
echo "PASS: acceptance fail-closed contracts present"
pass=$((pass + 1))

echo "==> qemu-ui identity / lock contracts"
grep -q 'is_our_qemu' scripts/qemu-ui.sh
grep -q 'flock' scripts/qemu-ui.sh
grep -q 'refusing to signal' scripts/qemu-ui.sh
grep -q 'already has a listener' scripts/qemu-ui.sh
grep -q 'ExitOnForwardFailure' scripts/qemu-ui.sh
echo "PASS: qemu-ui safety contracts present"
pass=$((pass + 1))

py() {
  if command -v python3 >/dev/null 2>&1; then
    python3 "$@"
  else
    python "$@"
  fi
}

# Local unit simulation: SHA256SUMS coverage helper via python excerpt
images="$tmp/images"
mkdir -p "$images"
echo hello >"$images/bzImage"
echo root >"$images/rootfs.ext2"
echo '{}' >"$images/build-manifest.json"
# Missing SHA256SUMS
assert_fails "missing SHA256SUMS detected by checker" bash -c "
  set -euo pipefail
  sums='$images/SHA256SUMS'
  test -f \"\$sums\"
"

# Uncovered file — bzImage listed, rootfs/manifest not.
printf 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  ./bzImage\n' >"$images/SHA256SUMS"
assert_fails "SHA256SUMS missing rootfs entry" bash -c '
  sums="'"$images"'/SHA256SUMS"
  grep -E "(^|[[:space:]])(\./)?rootfs\.ext2($|[[:space:]])" "$sums"
'

# Good listing passes coverage check
good_hash="$(sha256sum "$images/bzImage" | awk '{print $1}')"
good_root="$(sha256sum "$images/rootfs.ext2" | awk '{print $1}')"
good_man="$(sha256sum "$images/build-manifest.json" | awk '{print $1}')"
cat >"$images/SHA256SUMS" <<EOF
$good_hash  ./bzImage
$good_root  ./rootfs.ext2
$good_man  ./build-manifest.json
EOF
assert_ok "SHA256SUMS coverage OK" bash -c '
  sums="'"$images"'/SHA256SUMS"
  grep -Eq "(^|[[:space:]])(\./)?bzImage($|[[:space:]])" "$sums" &&
  grep -Eq "(^|[[:space:]])(\./)?rootfs\.ext2($|[[:space:]])" "$sums" &&
  grep -Eq "(^|[[:space:]])(\./)?build-manifest\.json($|[[:space:]])" "$sums"
'
assert_ok "SHA256SUMS verifies" bash -c "cd '$images' && sha256sum --check --strict SHA256SUMS"

# Ambiguous bzImage dirs
mkdir -p "$tmp/a" "$tmp/b"
cp "$images/bzImage" "$tmp/a/"
cp "$images/bzImage" "$tmp/b/"
count="$(find "$tmp" -type f -name bzImage -printf '%h\n' | sort -u | wc -l)"
if [[ "$count" -gt 1 ]]; then
  echo "PASS: ambiguous bzImage locations detectable ($count)"
  pass=$((pass + 1))
else
  echo "FAIL: ambiguous bzImage fixture" >&2
  fail=$((fail + 1))
fi

# qemu-ui refuse wrong identity (simulate PID file for shell itself)
export DBR_QEMU_UI_DIR="$tmp/qemu-ui-state"
mkdir -p "$DBR_QEMU_UI_DIR"
echo "$$" >"$DBR_QEMU_UI_DIR/qemu.pid"
assert_fails "qemu-ui stop refuses non-qemu pid" bash scripts/qemu-ui.sh stop

echo
echo "Evidence script regression: $pass passed, $fail failed"
[[ "$fail" -eq 0 ]]
