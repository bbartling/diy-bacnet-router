#!/usr/bin/env bash
# Verify a downloaded GH Actions image artifact and optionally run QEMU smoke.
#
# Usage:
#   bash scripts/accept-gh-image-artifact.sh /path/to/unzipped-or-images-dir
#   bash scripts/accept-gh-image-artifact.sh /path/to/dir --smoke
#   SKIP_SMOKE=1 bash scripts/accept-gh-image-artifact.sh /path/to/dir --smoke
#
# Finds images/ nested one level if needed. Prints REQUIRED vs optional files.
set -euo pipefail

root="${1:-}"
do_smoke=0
if [[ "${2:-}" == "--smoke" ]] || [[ "${RUN_SMOKE:-0}" == "1" ]]; then
  do_smoke=1
fi
if [[ -z "$root" || ! -d "$root" ]]; then
  echo "usage: $0 <artifact-dir> [--smoke]" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
images="$root"
if [[ ! -f "$images/SHA256SUMS" && -f "$images/images/SHA256SUMS" ]]; then
  images="$images/images"
fi
if [[ ! -f "$images/SHA256SUMS" ]]; then
  # GH zip sometimes nests the artifact name as a single directory
  nested="$(find "$root" -maxdepth 2 -type f -name SHA256SUMS 2>/dev/null | head -1 || true)"
  if [[ -n "$nested" ]]; then
    images="$(dirname "$nested")"
  fi
fi
if [[ ! -f "$images/SHA256SUMS" ]]; then
  echo "SHA256SUMS not found under $root" >&2
  exit 1
fi

echo "==> images dir: $images"
if [[ -f "$images/ARTIFACT_README.txt" ]]; then
  echo "==> ARTIFACT_README.txt"
  sed -n '1,80p' "$images/ARTIFACT_README.txt"
fi

echo "==> sha256sum --check"
(cd "$images" && sha256sum --check --strict SHA256SUMS)

target="unknown"
if [[ -f "$images/build-manifest.json" ]]; then
  target="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["target"])' "$images/build-manifest.json")"
  echo "==> build-manifest target=$target"
fi

echo "==> required vs optional (by target)"
case "$target" in
  x86_64)
    for f in bzImage rootfs.ext2 SHA256SUMS build-manifest.json ARTIFACT_README.txt; do
      test -s "$images/$f" && echo "  REQUIRED ok: $f" || echo "  REQUIRED MISSING: $f" >&2
    done
    for f in rootfs.iso legal-info.tar.xz buildroot-host-rustc-version.txt; do
      [[ -e "$images/$f" ]] && echo "  optional: $f" || echo "  optional absent: $f"
    done
    ;;
  generic_aarch64)
    kernel=""
    for candidate in Image Image.gz; do
      [[ -s "$images/$candidate" ]] && kernel=$candidate && break
    done
    [[ -n "$kernel" ]] && echo "  REQUIRED ok: $kernel" || echo "  REQUIRED MISSING: Image" >&2
    for f in rootfs.ext2 SHA256SUMS build-manifest.json ARTIFACT_README.txt; do
      test -s "$images/$f" && echo "  REQUIRED ok: $f" || echo "  REQUIRED MISSING: $f" >&2
    done
    ;;
  rpi3_64|rpi4_64|rpi5_64)
    for f in sdcard.img SHA256SUMS build-manifest.json ARTIFACT_README.txt; do
      test -s "$images/$f" && echo "  REQUIRED ok: $f" || echo "  REQUIRED MISSING: $f" >&2
    done
    ;;
  *)
    echo "  (unknown target — listing files)"
    ls -la "$images"
    ;;
esac

if [[ "$do_smoke" -eq 1 && "${SKIP_SMOKE:-0}" != "1" ]]; then
  case "$target" in
    x86_64)
      echo "==> qemu-smoke.sh"
      bash "$repo_root/scripts/qemu-smoke.sh" "$images"
      ;;
    generic_aarch64)
      echo "==> qemu-aarch64-smoke.sh"
      bash "$repo_root/scripts/qemu-aarch64-smoke.sh" "$images"
      ;;
    *)
      echo "no QEMU smoke for target=$target (flash/physical only)" >&2
      ;;
  esac
fi

echo "PASS: accept-gh-image-artifact ($images)"
