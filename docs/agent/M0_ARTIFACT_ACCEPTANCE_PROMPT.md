# M0 artifact acceptance prompt

Paste this into a Cursor agent **before** any Buildroot debugging or code edits.
This supersedes [LUNA_MAX_GITHUB_ACTIONS_PROMPT.md](LUNA_MAX_GITHUB_ACTIONS_PROMPT.md).

Also read [SOFTWARE_SPEC.md](SOFTWARE_SPEC.md) for product context (educational
BASRT-class router, Vibe13 prototype lineage, Waveshare C adapter).

---

## IMPORTANT UPDATE

GitHub Actions may already be green. Do **not** assume Buildroot is broken and do
**not** edit anything initially.

First inspect the newest `build-os` workflow runs. If a successful x86_64 run
exists:

1. Record its run ID and exact `headSha`.
2. Verify the x86 build and QEMU smoke steps **actually ran** and were **not**
   skipped.
3. Download the x86 artifact and checksum from that run.
4. Check out the exact `headSha` in detached mode.
5. Boot the downloaded artifact locally under QEMU (VMware Ubuntu lab guest).
6. Verify service startup and `GET /healthz` (expects `status: ok`,
   `data_plane: disabled`, `ready_to_route: false`). Optionally
   `GET /api/status` and WebSocket metrics snapshot (`/api/ws/metrics`).
7. Only then perform a clean local Buildroot build of the same SHA.
8. Compare the locally built artifact manifest/configuration with the Actions
   artifact.

Do not modify Buildroot merely because older runs failed. Diagnose or edit only if:

- the newest relevant run is failing;
- the artifact is missing;
- QEMU smoke was skipped or ineffective;
- the downloaded image fails locally;
- or a clean local rebuild is not reproducible.

If Actions, downloaded-artifact QEMU, and clean local Buildroot all pass, report
**M0 PASS** and stop. Do not manufacture additional cleanup work and do not begin
BACnet routing in this task.

## Inspect Actions (lab VM or Windows with `gh`)

```bash
gh auth status

gh run list \
  --repo bbartling/diy-bacnet-router \
  --workflow build-os.yml \
  --limit 10
```

Newest successful run:

```bash
RUN_ID=<successful-run-id>

gh run view "$RUN_ID" \
  --repo bbartling/diy-bacnet-router \
  --json conclusion,headSha,headBranch,event,jobs,url
```

Confirm jobs `x86_64 image` completed and step **QEMU boot smoke** ran (not
`skipped`).

## Download artifact

```bash
mkdir -p "$HOME/dbr-artifacts/$RUN_ID"

gh run download "$RUN_ID" \
  --repo bbartling/diy-bacnet-router \
  --dir "$HOME/dbr-artifacts/$RUN_ID"

find "$HOME/dbr-artifacts/$RUN_ID" \
  -maxdepth 3 -type f -printf '%p\t%s bytes\n' | sort
```

Verify checksums:

```bash
find "$HOME/dbr-artifacts/$RUN_ID" -type f -name 'SHA256SUMS' -print
# cd to directory containing SHA256SUMS && sha256sum --check --strict SHA256SUMS
```

Or use [scripts/vm-accept-artifact.sh](../../scripts/vm-accept-artifact.sh).

## Clone exact tested code

```bash
mkdir -p "$HOME/src"
cd "$HOME/src"

git clone https://github.com/bbartling/diy-bacnet-router.git
cd diy-bacnet-router

TESTED_SHA="$(
  gh run view "$RUN_ID" \
    --repo bbartling/diy-bacnet-router \
    --json headSha --jq .headSha
)"

git checkout --detach "$TESTED_SHA"
git status --short --branch
```

## QEMU boot (downloaded images)

```bash
# images/ under artifact dir — adjust path
bash scripts/qemu-smoke.sh "$HOME/dbr-artifacts/$RUN_ID/<artifact-folder>/images"
```

From Windows browser, tunnel the QEMU UI preview (loopback-only on the guest):

```bash
ssh -N -L 18080:127.0.0.1:18080 ubuntu2-buildroot
# then http://127.0.0.1:18080
```

Smoke and UI preview both use `hostfwd=tcp:127.0.0.1:18080-:8080`.

## Acceptance order (checklist)

1. Finish VMware `ubuntu2` SSH key auth (`scripts/vm-authorize-key.ps1`).
2. Clone repo; `gh auth login` on VM if needed (or download artifacts on the Windows host).
3. Download Actions artifact for successful run.
4. Verify SHA256SUMS and `build-manifest.json`.
5. Detached checkout at `headSha`.
6. QEMU boot + `/healthz` (`scripts/qemu-smoke.sh` or `scripts/qemu-ui.sh`).
7. Optional: clean `scripts/vm-build-x86.sh` at same SHA; compare manifests.
8. Update [docs/operations/LOCAL_BUILDROOT_VM.md](../operations/LOCAL_BUILDROOT_VM.md)
   session log.

## Windows helpers

```powershell
.\scripts\vm-ensure.ps1 -Hypervisor vmware
.\scripts\vm-authorize-key.ps1          # once
.\scripts\vm-ensure.ps1 -Hypervisor vmware -AcceptRunId <RUN_ID>
```

## After M0 PASS

Proceed with [CURSOR_CONTINUATION_PROMPT.md](CURSOR_CONTINUATION_PROMPT.md) at
M1 (rusty-bacnet adapter). UI work follows
[SOFTWARE_SPEC.md](SOFTWARE_SPEC.md) and
[docs/product/BASRT_EDUCATIONAL_REFERENCE.md](../product/BASRT_EDUCATIONAL_REFERENCE.md).
