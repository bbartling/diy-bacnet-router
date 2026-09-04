#!/usr/bin/env bash
# Download and accept a successful build-os Actions x86_64 artifact.
# Fail closed on missing/skipped/ambiguous evidence.
#
# Usage: vm-accept-artifact.sh <RUN_ID>
# Optional env:
#   GITHUB_REPOSITORY, DBR_ARTIFACT_ROOT, DBR_ACCEPT_WORKTREE
#   DBR_ACCEPT_REQUIRE_SHA  — if set, headSha must match
set -euo pipefail

RUN_ID="${1:-}"
REPO="${GITHUB_REPOSITORY:-bbartling/diy-bacnet-router}"
ART_ROOT="${DBR_ARTIFACT_ROOT:-$HOME/dbr-artifacts}"
WORKTREE_ROOT="${DBR_ACCEPT_WORKTREE:-$HOME/dbr-accept-worktrees}"
TARGET="x86_64"
REQUIRED_FILES=(
  bzImage
  rootfs.ext2
  SHA256SUMS
  build-manifest.json
)

if [[ -z "$RUN_ID" ]]; then
  echo "usage: $0 <github-actions-run-id>" >&2
  exit 2
fi

command -v gh >/dev/null || { echo "gh CLI required" >&2; exit 1; }
command -v python3 >/dev/null || { echo "python3 required" >&2; exit 1; }
command -v sha256sum >/dev/null || { echo "sha256sum required" >&2; exit 1; }

verify_sha256sums() {
  local images_dir="$1"
  local label="$2"
  local sums="$images_dir/SHA256SUMS"
  echo "==> SHA256SUMS $label ($sums)"
  if [[ ! -f "$sums" ]]; then
    echo "missing SHA256SUMS under $images_dir" >&2
    exit 1
  fi
  # Every required payload file (except SHA256SUMS itself) must be listed.
  python3 - "$sums" "${REQUIRED_FILES[@]}" <<'PY'
import pathlib, sys
sums = pathlib.Path(sys.argv[1])
required = [name for name in sys.argv[2:] if name != "SHA256SUMS"]
listed = set()
for line in sums.read_text(encoding="utf-8").splitlines():
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    parts = line.split()
    if len(parts) < 2:
        raise SystemExit(f"malformed SHA256SUMS line: {line!r}")
    name = parts[1].lstrip("./")
    listed.add(pathlib.Path(name).name)
missing = [name for name in required if name not in listed]
if missing:
    raise SystemExit(f"SHA256SUMS missing entries for: {', '.join(missing)}")
print(f"    listed required files: {', '.join(required)}")
PY
  (cd "$images_dir" && sha256sum --check --strict SHA256SUMS)
}

echo "==> Inspecting run $RUN_ID"
conclusion="$(gh run view "$RUN_ID" --repo "$REPO" --json conclusion --jq .conclusion)"
if [[ "$conclusion" != "success" ]]; then
  echo "run $RUN_ID conclusion is '$conclusion', expected success" >&2
  exit 1
fi

head_sha="$(gh run view "$RUN_ID" --repo "$REPO" --json headSha --jq .headSha)"
echo "    headSha=$head_sha"
if [[ -n "${DBR_ACCEPT_REQUIRE_SHA:-}" && "$head_sha" != "$DBR_ACCEPT_REQUIRE_SHA" ]]; then
  echo "headSha $head_sha does not match required $DBR_ACCEPT_REQUIRE_SHA" >&2
  exit 1
fi

jobs_json="$(gh run view "$RUN_ID" --repo "$REPO" --json jobs --jq .jobs)"
if ! python3 - "$jobs_json" <<'PY'
import json, sys
jobs = json.loads(sys.argv[1])
x86 = next((j for j in jobs if j.get("name") == "x86_64 image"), None)
if not x86 or x86.get("conclusion") != "success":
    raise SystemExit("x86_64 image job missing or not successful")
found = False
for step in x86.get("steps") or []:
    name = step.get("name", "")
    if "QEMU" in name:
        found = True
        if step.get("conclusion") == "skipped":
            raise SystemExit(f"step skipped: {name}")
        if step.get("conclusion") != "success":
            raise SystemExit(f"step not success: {name} ({step.get('conclusion')})")
        print(f"    QEMU step OK: {name}")
if not found:
    raise SystemExit("QEMU boot smoke step not found in x86_64 job")
PY
then
  exit 1
fi

dest="$ART_ROOT/$RUN_ID"
mkdir -p "$dest"
artifact_name="$(gh api "repos/$REPO/actions/runs/$RUN_ID/artifacts" \
  --jq ".artifacts[] | select(.name | test(\"diy-bacnet-router-x86_64-\")) | .name" \
  | head -n 1)"
if [[ -z "$artifact_name" ]]; then
  echo "could not find diy-bacnet-router-x86_64-* artifact on run $RUN_ID" >&2
  exit 1
fi
echo "==> Downloading only $artifact_name"
gh run download "$RUN_ID" --repo "$REPO" --name "$artifact_name" --dir "$dest"

echo "==> Artifact files"
find "$dest" -maxdepth 4 -type f | sort

# Exactly one x86 images directory containing bzImage + required files.
mapfile -t bz_dirs < <(find "$dest" -type f -name bzImage -printf '%h\n' | sort -u)
if [[ "${#bz_dirs[@]}" -eq 0 ]]; then
  echo "could not locate bzImage under $dest" >&2
  exit 1
fi
if [[ "${#bz_dirs[@]}" -ne 1 ]]; then
  echo "ambiguous bzImage locations under $dest:" >&2
  printf '  %s\n' "${bz_dirs[@]}" >&2
  exit 1
fi
images_dir="${bz_dirs[0]}"
echo "==> Images at $images_dir"

for required in "${REQUIRED_FILES[@]}"; do
  if [[ ! -e "$images_dir/$required" ]]; then
    echo "missing required file: $images_dir/$required" >&2
    exit 1
  fi
done

verify_sha256sums "$images_dir" "before QEMU"

python3 - "$images_dir/build-manifest.json" "$head_sha" "$TARGET" <<'PY'
import json, pathlib, sys
manifest = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
head_sha = sys.argv[2]
target = sys.argv[3]
proj = manifest.get("project_git_sha") or manifest.get("git_sha")
man_target = manifest.get("target")
if proj != head_sha:
    raise SystemExit(f"manifest project_git_sha {proj!r} != run headSha {head_sha!r}")
if man_target != target:
    raise SystemExit(f"manifest target {man_target!r} != required {target!r}")
print(f"    manifest OK: target={man_target} sha={proj}")
print(f"    buildroot={manifest.get('buildroot_version')} project_rust={manifest.get('project_rust_toolchain')}")
PY

# Isolated acceptance worktree — never detach an active development checkout.
accept_repo="$WORKTREE_ROOT/$RUN_ID"
mkdir -p "$WORKTREE_ROOT"
if [[ -d "$accept_repo/.git" ]] || [[ -f "$accept_repo/.git" ]]; then
  if [[ -n "$(git -C "$accept_repo" status --porcelain 2>/dev/null || true)" ]]; then
    echo "refusing dirty acceptance worktree: $accept_repo" >&2
    exit 1
  fi
  git -C "$accept_repo" fetch origin
  git -C "$accept_repo" checkout --detach "$head_sha"
else
  git clone --no-checkout "https://github.com/$REPO.git" "$accept_repo"
  git -C "$accept_repo" fetch origin
  git -C "$accept_repo" checkout --detach "$head_sha"
fi

smoke="$accept_repo/scripts/qemu-smoke.sh"
if [[ ! -x "$smoke" && -f "$smoke" ]]; then
  chmod +x "$smoke"
fi
if [[ ! -f "$smoke" ]]; then
  echo "qemu-smoke.sh missing in acceptance checkout; cannot accept" >&2
  exit 1
fi

echo "==> QEMU smoke on downloaded artifact (mandatory)"
bash "$smoke" "$images_dir"

verify_sha256sums "$images_dir" "after QEMU (-snapshot must leave rootfs unchanged)"

echo "==> M0 artifact acceptance complete for run $RUN_ID @ $head_sha"
echo "    Artifact path: $dest"
echo "    Images dir:    $images_dir"
echo "    Worktree:      $accept_repo @ $head_sha"
