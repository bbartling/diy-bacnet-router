#!/usr/bin/env bash
# Download and accept a successful build-os Actions artifact on the lab VM.
# Usage: vm-accept-artifact.sh <RUN_ID>
set -euo pipefail

RUN_ID="${1:-}"
REPO="${GITHUB_REPOSITORY:-bbartling/diy-bacnet-router}"
ART_ROOT="${DBR_ARTIFACT_ROOT:-$HOME/dbr-artifacts}"
CLONE_DIR="${DBR_CLONE_DIR:-$HOME/src/diy-bacnet-router}"

if [[ -z "$RUN_ID" ]]; then
  echo "usage: $0 <github-actions-run-id>" >&2
  exit 2
fi

command -v gh >/dev/null || { echo "gh CLI required" >&2; exit 1; }

echo "==> Inspecting run $RUN_ID"
conclusion="$(gh run view "$RUN_ID" --repo "$REPO" --json conclusion --jq .conclusion)"
if [[ "$conclusion" != "success" ]]; then
  echo "run $RUN_ID conclusion is '$conclusion', expected success" >&2
  exit 1
fi

head_sha="$(gh run view "$RUN_ID" --repo "$REPO" --json headSha --jq .headSha)"
echo "    headSha=$head_sha"

# Fail if x86 QEMU smoke was skipped
jobs_json="$(gh run view "$RUN_ID" --repo "$REPO" --json jobs --jq .jobs)"
if ! python3 - "$jobs_json" <<'PY'
import json, sys
jobs = json.loads(sys.argv[1])
x86 = next((j for j in jobs if j.get("name") == "x86_64 image"), None)
if not x86 or x86.get("conclusion") != "success":
    raise SystemExit("x86_64 image job missing or not successful")
for step in x86.get("steps") or []:
    name = step.get("name", "")
    if "QEMU" in name:
        if step.get("conclusion") == "skipped":
            raise SystemExit(f"step skipped: {name}")
        if step.get("conclusion") != "success":
            raise SystemExit(f"step not success: {name} ({step.get('conclusion')})")
        print(f"    QEMU step OK: {name}")
        break
else:
    raise SystemExit("QEMU boot smoke step not found in x86_64 job")
PY
then
  exit 1
fi

dest="$ART_ROOT/$RUN_ID"
mkdir -p "$dest"
echo "==> Downloading artifacts to $dest"
gh run download "$RUN_ID" --repo "$REPO" --dir "$dest"

echo "==> Artifact files"
find "$dest" -maxdepth 4 -type f | sort

while IFS= read -r sums; do
  echo "==> Checking $sums"
  (cd "$(dirname "$sums")" && sha256sum --check --strict "$(basename "$sums")")
done < <(find "$dest" -type f -name 'SHA256SUMS' -print)

images_dir="$(find "$dest" -type f -name bzImage -printf '%h\n' 2>/dev/null | head -1)"
if [[ -z "$images_dir" ]]; then
  echo "could not locate bzImage under $dest" >&2
  exit 1
fi
echo "==> Images at $images_dir"

if [[ ! -d "$CLONE_DIR/.git" ]]; then
  mkdir -p "$(dirname "$CLONE_DIR")"
  git clone "https://github.com/$REPO.git" "$CLONE_DIR"
fi
cd "$CLONE_DIR"
git fetch origin
git checkout --detach "$head_sha"

repo_root="$CLONE_DIR"
if [[ -x "$repo_root/scripts/qemu-smoke.sh" ]]; then
  echo "==> QEMU smoke on downloaded artifact"
  bash "$repo_root/scripts/qemu-smoke.sh" "$images_dir"
else
  echo "warning: qemu-smoke.sh not found; skipping boot test" >&2
fi

echo "==> M0 artifact acceptance complete for run $RUN_ID @ $head_sha"
echo "    Log artifact path: $dest"
echo "    Code checkout: $CLONE_DIR @ $head_sha"
