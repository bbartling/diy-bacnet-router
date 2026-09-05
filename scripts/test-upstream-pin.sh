#!/usr/bin/env bash
# Fail if rusty-bacnet pin locations disagree.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

expected="$(awk -F'"' '/^revision = /{print $2; exit}' config/upstream-lock.toml)"
test "${#expected}" -eq 40

const_rev="$(grep -E 'pub const UPSTREAM_REVISION' crates/rusty-bacnet-adapter/src/lib.rs | grep -oE '[0-9a-f]{40}')"
test "$const_rev" = "$expected"

while IFS= read -r line; do
  rev="$(echo "$line" | grep -oE 'rev = "[0-9a-f]{40}"' | grep -oE '[0-9a-f]{40}')"
  test "$rev" = "$expected"
done < <(grep -E 'bacnet-(types|encoding|transport|network).*rev =' Cargo.toml)

if test -f docs/evidence/M1_CLOSEOUT.md; then
  grep -q "$expected" docs/evidence/M1_CLOSEOUT.md
fi

grep -q "$expected" docs/UPSTREAM_LOCK.md
echo "PASS: upstream pin $expected consistent"
