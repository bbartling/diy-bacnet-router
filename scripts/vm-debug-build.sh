#!/usr/bin/env bash
# Local Buildroot debug build — mirrors build-os.yml when GitHub Actions fails.
# Run on VirtualBox ubuntu2 (preferred) or WSL Ubuntu. Logs everything for
# porting fixes back to .github/workflows/build-os.yml.
#
# Usage:
#   bash scripts/vm-debug-build.sh              # full x86_64 build + verify + qemu
#   bash scripts/vm-debug-build.sh --verify-only # checksum/qemu on existing output
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export BUILDROOT_VERSION="${BUILDROOT_VERSION:-2025.02.17}"
export BUILD_WORK_ROOT="${BUILD_WORK_ROOT:-$HOME/dbr-buildroot}"
export JOBS="${JOBS:-$(nproc)}"
export OUTPUT_DIR="$BUILD_WORK_ROOT/output/x86_64"
log="${BUILD_LOG:-$BUILD_WORK_ROOT/x86_64-debug-$(date +%Y%m%d-%H%M%S).log}"
images="$OUTPUT_DIR/images"
verify_only=false

if [[ "${1:-}" == "--verify-only" ]]; then
  verify_only=true
fi

verify_artifacts() {
  local root="$OUTPUT_DIR"
  echo "==> Verifying image evidence (same checks as build-os.yml)"
  test -d "$root/legal-info"
  test -s "$images/legal-info.tar.xz"
  test -s "$images/build-manifest.json"
  test -s "$images/buildroot-host-rustc-version.txt"
  test -s "$images/buildroot-host-cargo-version.txt"
  test -s "$images/SHA256SUMS"
  python3 - "$images/build-manifest.json" "x86_64" <<'PY'
import json, sys
with open(sys.argv[1], encoding="utf-8") as handle:
    manifest = json.load(handle)
assert manifest["target"] == sys.argv[2]
assert manifest["project_rust_toolchain"] == "1.93.0"
assert manifest["rusty_bacnet"] == "not-integrated"
assert manifest["buildroot_host_rustc_version"]
assert manifest["buildroot_host_cargo_version"]
PY
  (cd "$images" && sha256sum --check --strict SHA256SUMS)
  echo "==> SHA256SUMS verify PASS"
}

if [[ "$verify_only" == true ]]; then
  verify_artifacts
  if command -v qemu-system-x86_64 >/dev/null; then
    bash "$repo_root/scripts/qemu-smoke.sh" "$images"
  fi
  exit 0
fi

mkdir -p "$BUILD_WORK_ROOT"
cd "$repo_root"

{
  echo "==> vm-debug-build started $(date --iso-8601=seconds)"
  echo "    repo=$repo_root"
  echo "    sha=$(git rev-parse HEAD 2>/dev/null || echo unknown)"
  echo "    jobs=$JOBS"
  echo "    log=$log"

  echo "==> Host deps check"
  for cmd in git python3 npm nproc; do command -v "$cmd"; done

  echo "==> Building frontend"
  npm --prefix frontend/web ci
  npm --prefix frontend/web run build

  echo "==> Buildroot x86_64"
  bash scripts/build-image.sh x86_64

  echo "==> Post-build verify (before QEMU — must match CI pristine checksums)"
  verify_artifacts

  if command -v qemu-system-x86_64 >/dev/null; then
    echo "==> QEMU smoke (-snapshot must not mutate rootfs.ext2)"
    bash scripts/qemu-smoke.sh "$images"
    echo "==> Post-QEMU verify (rootfs.ext2 checksum must still match)"
    (cd "$images" && sha256sum --check --strict SHA256SUMS)
    echo "==> Post-QEMU SHA256SUMS verify PASS"
  else
    echo "warning: qemu-system-x86_64 not installed; skipping smoke" >&2
  fi

  echo "==> vm-debug-build PASS $(date --iso-8601=seconds)"
  ls -lh "$images"
} 2>&1 | tee "$log"

echo "Log saved: $log"
echo "Port any script/workflow fixes to .github/workflows/build-os.yml and commit from Windows."
