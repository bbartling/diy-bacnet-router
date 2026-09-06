#!/usr/bin/env bash
# Offline golden tests for the independent BVLL oracle (no sockets required).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
python3 "$root/scripts/bip_bvll_oracle.py" self-test
# Encode path must stay aligned with Rust GOLDEN_NPDU.
out="$(python3 "$root/scripts/bip_bvll_oracle.py" encode-golden)"
echo "$out" | head -n1 | grep -qx '010010'
echo "PASS: bip BVLL oracle golden contract"
