#!/usr/bin/env bash
# Contract tests for M0 image pipeline scripts (CI + VM lab parity).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "==> qemu-smoke must not mutate rootfs.ext2 (uses -snapshot)"
grep -q '\-snapshot' scripts/qemu-smoke.sh
grep -q 'hostfwd=tcp:127.0.0.1:18080-:8080' scripts/qemu-smoke.sh
grep -q 'noapic' scripts/qemu-smoke.sh

echo "==> qemu-ui persistent preview contract"
test -f scripts/qemu-ui.sh
grep -q '\-snapshot' scripts/qemu-ui.sh
grep -q 'hostfwd=tcp:127.0.0.1:' scripts/qemu-ui.sh
grep -q 'start' scripts/qemu-ui.sh
grep -q 'status' scripts/qemu-ui.sh
grep -q 'stop' scripts/qemu-ui.sh

echo "==> build-os must verify SHA256SUMS after image build"
grep -q 'sha256sum --check --strict SHA256SUMS' .github/workflows/build-os.yml

echo "==> build-os x86 must run QEMU smoke"
grep -q 'QEMU boot smoke' .github/workflows/build-os.yml
grep -q 'scripts/qemu-smoke.sh' .github/workflows/build-os.yml

echo "==> build-os matrix.json drives target ids"
test -f .github/workflows/matrix.json
grep -q 'generic_aarch64' .github/workflows/matrix.json
grep -q 'matrix.json' .github/workflows/build-os.yml

echo "==> minimal qemu + full images artifacts"
grep -q 'dbr-.*-qemu' .github/workflows/build-os.yml
grep -q 'dbr-.*-images' .github/workflows/build-os.yml
grep -q 'ARTIFACT_README.txt' .github/workflows/build-os.yml

echo "==> accept-gh-image-artifact + aarch64 smoke scripts exist"
test -f scripts/accept-gh-image-artifact.sh
test -f scripts/qemu-aarch64-smoke.sh
grep -q 'sha256sum --check --strict' scripts/accept-gh-image-artifact.sh

echo "==> vm-debug-build re-verifies checksums after QEMU"
grep -q 'Post-QEMU verify' scripts/vm-debug-build.sh
grep -q 'sha256sum --check --strict SHA256SUMS' scripts/vm-debug-build.sh

echo "==> vm.env template must not commit secrets"
test -f config/vm.env.example
grep -q 'VM_SSH_PASSWORD' config/vm.env.example
if git ls-files --error-unmatch config/vm.env >/dev/null 2>&1; then
  echo "config/vm.env must stay gitignored" >&2
  exit 1
fi

echo "Image evidence contract tests PASS"
