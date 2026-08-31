#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

test -f AGENTS.md
test -f config/upstream-lock.toml
test -f docs/hardware/WAVESHARE_USB_RS485_C.md
test -f openapi/openapi.json
grep -q 'ready_to_route' crates/routerd/src/web.rs
grep -q 'router.enabled=true is unavailable' crates/router-core/src/config.rs
! grep -R -n -E 'password\s*=\s*"[^\"]+"' config buildroot-external
! grep -R -n -E '/dev/ttyUSB[0-9]' config buildroot-external

if python3 -c 'import json,sys; json.load(open(sys.argv[1], encoding="utf-8"))' \
  openapi/openapi.json 2>/dev/null; then
  :
else
  python -c 'import json,sys; json.load(open(sys.argv[1], encoding="utf-8"))' \
    openapi/openapi.json
fi
echo "Repository contract validation PASS"
