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
lock_file="$repo_root/config/buildroot-lock.toml"
if [[ ! -f "$lock_file" ]]; then
  echo "missing Buildroot lock: $lock_file" >&2
  exit 2
fi
buildroot_version="${BUILDROOT_VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' "$lock_file")}"
buildroot_expected_sha="$(sed -n 's/^commit = "\(.*\)"/\1/p' "$lock_file")"
if [[ -z "$buildroot_version" || -z "$buildroot_expected_sha" ]]; then
  echo "invalid Buildroot lock: $lock_file" >&2
  exit 2
fi
work_root="${BUILD_WORK_ROOT:-${RUNNER_TEMP:-/tmp}/dbr-buildroot}"
source_dir="$work_root/buildroot-$buildroot_version"
output_dir="${OUTPUT_DIR:-$work_root/output/$target}"
external="$repo_root/buildroot-external"
git_sha="$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || printf unknown)"
source_archive_name="diy-bacnet-router-${git_sha}.tar.gz"
source_archive="$work_root/$source_archive_name"
buildroot_dl_dir="$source_dir/dl"

# Keep the Buildroot input immutable.  The tag is human-readable, while the
# expected commit in config/buildroot-lock.toml makes a retagged reference fail closed.

mkdir -p "$work_root" "$output_dir"
if [[ -d "$source_dir" && ! -d "$source_dir/.git" ]]; then
  # Actions cache may restore only dl/ under source_dir; remove the partial tree
  # so git clone can populate a full Buildroot checkout.
  rm -rf "$source_dir"
fi
if [[ ! -d "$source_dir/.git" ]]; then
  git clone --depth 1 --branch "$buildroot_version" \
    https://gitlab.com/buildroot.org/buildroot.git "$source_dir"
fi
buildroot_sha="$(git -C "$source_dir" rev-parse HEAD)"
if [[ "$buildroot_sha" != "$buildroot_expected_sha" ]]; then
  echo "Buildroot $buildroot_version resolved to unexpected commit $buildroot_sha" >&2
  echo "expected $buildroot_expected_sha" >&2
  exit 1
fi
buildroot_tag="$(git -C "$source_dir" describe --tags --exact-match "$buildroot_sha" 2>/dev/null || true)"
if [[ "$buildroot_tag" != "$buildroot_version" ]]; then
  echo "Buildroot commit $buildroot_sha is not tagged $buildroot_version" >&2
  exit 1
fi

# Buildroot's cargo-package infrastructure vendors dependencies while it
# downloads a normal source archive.  An OVERRIDE_SRCDIR bypasses that
# download post-process, leaving the later --offline build without crates.
# Package the checked-out application as a local archive so the official
# cargo-package download/vendor path remains active and reproducible.
tar -C "$(dirname "$repo_root")" \
  --sort=name \
  --mtime='UTC 1970-01-01' \
  --numeric-owner --owner=0 --group=0 \
  --pax-option=delete=atime,delete=ctime,delete=mtime \
  --exclude=.git \
  --exclude=.cache \
  --exclude=node_modules \
  --exclude=output \
  --exclude=target \
  -cf - "$(basename "$repo_root")" | gzip -6 -n > "$source_archive"
rm -f "$buildroot_dl_dir/$source_archive_name"

buildroot_make=(
  make -C "$source_dir"
  O="$output_dir"
  BR2_EXTERNAL="$external"
  DIY_BACNET_ROUTER_SITE_METHOD=file
  DIY_BACNET_ROUTER_SITE="$work_root"
  DIY_BACNET_ROUTER_SOURCE="$source_archive_name"
)

"${buildroot_make[@]}" "$base_defconfig"
cat "$external/fragments/common.config" >> "$output_dir/.config"
cat >> "$output_dir/.config" <<EOF
BR2_ROOTFS_OVERLAY="$external/board/common/rootfs-overlay"
EOF
"${buildroot_make[@]}" olddefconfig
"${buildroot_make[@]}" -j"${JOBS:-$(nproc)}"

case "$target" in
  x86_64) expected_images=(bzImage rootfs.ext2) ;;
  rpi3_64|rpi4_64|rpi5_64) expected_images=(sdcard.img) ;;
esac
for image in "${expected_images[@]}"; do
  if [[ ! -s "$output_dir/images/$image" ]]; then
    echo "Buildroot did not produce required image: $output_dir/images/$image" >&2
    exit 1
  fi
done

host_rustc="$output_dir/host/bin/rustc"
host_cargo="$output_dir/host/bin/cargo"
if [[ ! -x "$host_rustc" || ! -x "$host_cargo" ]]; then
  echo "Buildroot host rustc/cargo were not produced under $output_dir/host/bin" >&2
  exit 1
fi
host_rustc_version="$($host_rustc --version)"
host_cargo_version="$($host_cargo --version)"
printf '%s\n' "$host_rustc_version" > "$output_dir/images/buildroot-host-rustc-version.txt"
printf '%s\n' "$host_cargo_version" > "$output_dir/images/buildroot-host-cargo-version.txt"
echo "Buildroot host rustc: $host_rustc_version"
echo "Buildroot host cargo: $host_cargo_version"
if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo '### Buildroot host Rust toolchain'
    echo "- \`$host_rustc --version\`: $host_rustc_version"
    echo "- \`$host_cargo --version\`: $host_cargo_version"
  } >> "$GITHUB_STEP_SUMMARY"
fi

# legal-info is intentionally after the successful image and toolchain checks.
"${buildroot_make[@]}" legal-info

legal_info_archive="$output_dir/images/legal-info.tar.xz"
tar -C "$output_dir" -cJf "$legal_info_archive" legal-info
test -s "$legal_info_archive"

manifest="$output_dir/images/build-manifest.json"
buildroot_sha="$(git -C "$source_dir" rev-parse HEAD)"
project_rust_toolchain="$(awk -F'"' '/^[[:space:]]*channel[[:space:]]*=/{print $2; exit}' "$repo_root/rust-toolchain.toml")"
rusty_bacnet_rev="$(awk -F'"' '/^revision = /{print $2; exit}' "$repo_root/config/upstream-lock.toml")"
test -n "$rusty_bacnet_rev"
test "${#rusty_bacnet_rev}" -eq 40
cat > "$manifest" <<EOF
{
  "schema_version": 1,
  "target": "$target",
  "project_git_sha": "$git_sha",
  "project_rust_toolchain": "$project_rust_toolchain",
  "buildroot_host_rustc_version": "$host_rustc_version",
  "buildroot_host_rustc_version_file": "buildroot-host-rustc-version.txt",
  "buildroot_host_cargo_version": "$host_cargo_version",
  "buildroot_host_cargo_version_file": "buildroot-host-cargo-version.txt",
  "buildroot_version": "$buildroot_version",
  "buildroot_git_sha": "$buildroot_sha",
  "rusty_bacnet": "$rusty_bacnet_rev"
}
EOF
(cd "$output_dir/images" && \
  find . -maxdepth 1 -type f ! -name SHA256SUMS -print0 \
    | sort -z \
    | xargs -0 sha256sum > SHA256SUMS)
test -s "$manifest"
test -s "$output_dir/images/SHA256SUMS"
echo "Images: $output_dir/images"
