#!/usr/bin/env bash
set -euo pipefail

version="1.7.12"
asset="actionlint_${version}_linux_amd64.tar.gz"
checksum="8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8"
tool_root="${RUNNER_TEMP:-/tmp}/dbr-actionlint-${version}"
archive="$tool_root/$asset"
binary="$tool_root/actionlint"

if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
  echo "actionlint bootstrap currently supports Linux x86_64 only" >&2
  exit 2
fi

mkdir -p "$tool_root"
if [[ ! -x "$binary" ]]; then
  curl --fail --location --retry 3 --proto '=https' --tlsv1.2 \
    "https://github.com/rhysd/actionlint/releases/download/v${version}/${asset}" \
    --output "$archive"
  printf '%s  %s\n' "$checksum" "$archive" | sha256sum --check --strict
  tar --extract --no-same-owner --file "$archive" --directory "$tool_root" actionlint
  chmod 0755 "$binary"
fi

"$binary" -color
