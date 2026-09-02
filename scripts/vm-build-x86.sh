#!/usr/bin/env bash
# Build x86_64 appliance image on the VM (mirrors build-os.yml core steps).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export BUILDROOT_VERSION="${BUILDROOT_VERSION:-2025.02.17}"
export BUILD_WORK_ROOT="${BUILD_WORK_ROOT:-$HOME/dbr-buildroot}"
export JOBS="${JOBS:-$(nproc)}"
export OUTPUT_DIR="$BUILD_WORK_ROOT/output/x86_64"
log="${BUILD_LOG:-$BUILD_WORK_ROOT/x86_64-build.log}"

mkdir -p "$BUILD_WORK_ROOT"
cd "$repo_root"

echo "==> Building frontend"
npm --prefix frontend/web ci
npm --prefix frontend/web run build

echo "==> Buildroot x86_64 (jobs=$JOBS, log=$log)"
set -o pipefail
bash scripts/build-image.sh x86_64 2>&1 | tee "$log"

images="$OUTPUT_DIR/images"
echo "==> Images in $images"
ls -lh "$images"

if [[ -x "$(command -v qemu-system-x86_64)" ]]; then
  echo "==> QEMU smoke"
  bash scripts/qemu-smoke.sh "$images"
fi

echo "==> Done. Manifest:"
cat "$images/build-manifest.json"
