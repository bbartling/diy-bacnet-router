#!/usr/bin/env bash
# Fail if rusty-bacnet pin locations disagree.
# Run on every CI via scripts/validate-repository.sh and after any Cargo.toml
# git rev bump for bacnet-* crates.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

expected="$(awk -F'"' '/^revision = /{print $2; exit}' config/upstream-lock.toml)"
short="$(awk -F'"' '/^revision_short = /{print $2; exit}' config/upstream-lock.toml)"
test "${#expected}" -eq 40
test "${#short}" -ge 7
test "${expected:0:${#short}}" = "$short"

const_rev="$(grep -E 'pub const UPSTREAM_REVISION:' crates/rusty-bacnet-adapter/src/lib.rs | grep -oE '[0-9a-f]{40}')"
const_short="$(grep -E 'pub const UPSTREAM_REVISION_SHORT:' crates/rusty-bacnet-adapter/src/lib.rs | grep -oE '[0-9a-f]+' | head -1)"
test "$const_rev" = "$expected"
test "$const_short" = "$short"

while IFS= read -r line; do
  rev="$(echo "$line" | grep -oE 'rev = "[0-9a-f]{40}"' | grep -oE '[0-9a-f]{40}')"
  test "$rev" = "$expected"
done < <(grep -E 'bacnet-(types|encoding|transport|network).*rev =' Cargo.toml)

# Cargo.lock must resolve the same git rev (four bacnet-* crates).
lock_hits="$(grep -c "rev=${expected}" Cargo.lock || true)"
test "$lock_hits" -ge 4

grep -q "$expected" docs/UPSTREAM_LOCK.md
grep -q "$expected" config/upstream-lock.toml

echo "PASS: upstream pin $expected ($short) consistent across lock, Cargo.toml, Cargo.lock, adapter consts, UPSTREAM_LOCK.md"
