---
name: local-buildroot-vm
description: >-
  M0 artifact acceptance and optional Buildroot builds on VirtualBox ubuntu2 from
  Windows/Cursor. Use when downloading GitHub Actions x86 artifacts, QEMU boot
  on lab VM, or comparing local builds to build-os.yml — after reading
  M0_ARTIFACT_ACCEPTANCE_PROMPT.md.
---

# Local Buildroot VM workflow

## Roles

| Surface | Purpose |
|---------|---------|
| **Windows + Cursor** | Edit, commit, push, `gh`, Rust/npm CI |
| **VirtualBox ubuntu2** | **Artifact acceptance first**, then optional Buildroot rebuild |
| **GitHub Actions** | CI truth; download artifacts from successful `build-os` runs |

Git on **Windows only**. VM pulls/clones for testing.

## Order (artifact-first)

1. Inspect newest `build-os` run — do not assume failure.
2. `.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>` — download, checksum, QEMU.
3. Only if needed: `.\scripts\vm-ensure.ps1 -RunBuild` — clean local rebuild at same SHA.

See [docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md](../../docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md).

## Windows scripts

```powershell
.\scripts\vm-ensure.ps1
.\scripts\vm-authorize-key.ps1       # once
.\scripts\vm-ensure.ps1 -RunSetup
.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>
.\scripts\vm-ensure.ps1 -RunBuild
```

VM: VirtualBox `ubuntu2`, SSH `ben@127.0.0.1:2222`, alias `ubuntu2-buildroot`.

## Agent behavior

1. Read [docs/operations/LOCAL_BUILDROOT_VM.md](../../docs/operations/LOCAL_BUILDROOT_VM.md).
2. Read [docs/agent/SOFTWARE_SPEC.md](../../docs/agent/SOFTWARE_SPEC.md) for product context.
3. Run app CI on Windows before Buildroot edits.
4. Verify QEMU smoke step **ran** (not skipped) before accepting a run.
5. Log results in LOCAL_BUILDROOT_VM.md acceptance table.

SSH BatchMode requires `vm-authorize-key.ps1` completed once.

## Related docs

- [docs/agent/SOFTWARE_SPEC.md](../../docs/agent/SOFTWARE_SPEC.md)
- [docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md](../../docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md)
- [.cursor/skills/basrt-educational-router/SKILL.md](../basrt-educational-router/SKILL.md)
- [scripts/vm-accept-artifact.sh](../../scripts/vm-accept-artifact.sh)
- `.github/workflows/build-os.yml`
