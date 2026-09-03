#!/usr/bin/env bash
# Offline contract tests for the Linux embedded appliance (Buildroot + routerd).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "==> build-image.sh defines all appliance targets"
for target in x86_64 rpi3_64 rpi4_64 rpi5_64; do
  grep -q "${target})" scripts/build-image.sh
done

echo "==> x86_64 expects kernel + rootfs artifacts"
grep -q 'expected_images=(bzImage rootfs.ext2)' scripts/build-image.sh

echo "==> Pi targets expect sdcard.img"
grep -q 'expected_images=(sdcard.img)' scripts/build-image.sh

echo "==> Buildroot external package and init script present"
test -f buildroot-external/external.desc
test -f buildroot-external/package/diy-bacnet-router/diy-bacnet-router.mk
test -f buildroot-external/package/diy-bacnet-router/S80diy-bacnet-router
grep -q 'start-stop-daemon' buildroot-external/package/diy-bacnet-router/S80diy-bacnet-router
grep -q '\-c dbr:dbr' buildroot-external/package/diy-bacnet-router/S80diy-bacnet-router

echo "==> example router.toml is fail-closed"
grep -q '127.0.0.1:8080' config/router.example.toml
grep -q 'enabled = false' config/router.example.toml
grep -q '/dev/serial/by-id/' config/router.example.toml
! grep -qE '/dev/ttyUSB[0-9]' config/router.example.toml

echo "==> qemu-smoke checks unprivileged service and health contract"
grep -q '\-snapshot' scripts/qemu-smoke.sh
grep -q 'ready_to_route' scripts/qemu-smoke.sh
grep -q 'uid=' scripts/qemu-smoke.sh

echo "==> qemu-ui is persistent preview only (snapshot + loopback)"
test -f scripts/qemu-ui.sh
grep -q '\-snapshot' scripts/qemu-ui.sh
grep -q '127.0.0.1' scripts/qemu-ui.sh

echo "==> daemon supports --check-config without binding"
grep -q -- '--check-config' crates/routerd/src/main.rs

echo "==> Buildroot lock is pinned with commit"
grep -q '^version = ' config/buildroot-lock.toml
grep -q '^commit = ' config/buildroot-lock.toml

echo "==> upstream lock documents rusty-bacnet pin"
grep -q 'rusty-bacnet' config/upstream-lock.toml
grep -q 'rusty-bacnet' docs/UPSTREAM_LOCK.md

echo "==> OpenAPI lists core management routes"
for route in /healthz /api/status /api/metrics/snapshot /api/ws/metrics; do
  grep -q "\"$route\"" openapi/openapi.json
done

echo "Appliance contract tests PASS"
