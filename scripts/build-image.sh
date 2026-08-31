#!/usr/bin/env bash
set -euo pipefail

target="${1:-}"
case "$target" in
  x86_64) base_defconfig=qemu_x86_64_defconfig ;;
  rpi3_64) base_defconfig=raspberrypi3_64_defconfig ;;
  rpi4_64) base_defconfig=raspberrypi4_64_defconfig ;;
  rpi5_64) base_defconfig=raspberrypi5_defconfig ;;
  *) echo "usage: $0 {x86_64|rpi3_64|rpi4_64|rpi5_64}" >&2; exit 2 ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
buildroot_version="${BUILDROOT_VERSION:-2025.02.17}"
work_root="${BUILD_WORK_ROOT:-${RUNNER_TEMP:-/tmp}/dbr-buildroot}"
source_dir="$work_root/buildroot-$buildroot_version"
output_dir="${OUTPUT_DIR:-$work_root/output/$target}"
external="$repo_root/buildroot-external"

mkdir -p "$work_root" "$output_dir"
if [[ ! -d "$source_dir/.git" ]]; then
  git clone --depth 1 --branch "$buildroot_version" \
    https://gitlab.com/buildroot.org/buildroot.git "$source_dir"
fi

make -C "$source_dir" O="$output_dir" BR2_EXTERNAL="$external" "$base_defconfig"
cat > "$output_dir/local.mk" <<EOF
DIY_BACNET_ROUTER_OVERRIDE_SRCDIR = $repo_root
DIY_BACNET_ROUTER_OVERRIDE_SRCDIR_RSYNC_EXCLUSIONS = --exclude .git --exclude .cache --exclude node_modules --exclude output --exclude target
EOF
cat "$external/fragments/common.config" >> "$output_dir/.config"
cat >> "$output_dir/.config" <<EOF
BR2_ROOTFS_OVERLAY="$external/board/common/rootfs-overlay"
EOF
make -C "$source_dir" O="$output_dir" BR2_EXTERNAL="$external" olddefconfig
make -C "$source_dir" O="$output_dir" BR2_EXTERNAL="$external" -j"${JOBS:-$(nproc)}"
make -C "$source_dir" O="$output_dir" BR2_EXTERNAL="$external" legal-info

manifest="$output_dir/images/build-manifest.json"
git_sha="$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || printf unknown)"
buildroot_sha="$(git -C "$source_dir" rev-parse HEAD)"
cat > "$manifest" <<EOF
{
  "schema_version": 1,
  "target": "$target",
  "project_git_sha": "$git_sha",
  "buildroot_version": "$buildroot_version",
  "buildroot_git_sha": "$buildroot_sha",
  "rusty_bacnet": "not-integrated"
}
EOF
(cd "$output_dir/images" && \
  find . -maxdepth 1 -type f ! -name SHA256SUMS -print0 \
    | sort -z \
    | xargs -0 sha256sum > SHA256SUMS)
echo "Images: $output_dir/images"
