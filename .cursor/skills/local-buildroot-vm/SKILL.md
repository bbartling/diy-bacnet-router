---
name: local-buildroot-vm
description: >-
  M0 artifact acceptance and local Buildroot debug on VirtualBox ubuntu2 (or WSL
  fallback) when GitHub Actions build-os fails. Use vm-debug-build.sh to reproduce
  CI, fix scripts, port back to build-os.yml.
---

# Local Buildroot VM workflow

## Roles

| Surface | Purpose |
|---------|---------|
| **Windows + Cursor** | Edit, commit, push, `gh`, Rust/npm CI |
| **VirtualBox ubuntu2** | Local Buildroot debug + artifact acceptance |
| **WSL Ubuntu** | Fallback Linux lab when VM SSH key not ready |
| **GitHub Actions** | CI truth after fixes ported from local |

Git on **Windows only**.

## When GitHub Actions fails

1. Fetch logs: `gh run view <RUN_ID> --repo bbartling/diy-bacnet-router --log-failed`
2. Common failure (fixed): QEMU smoke mutating `rootfs.ext2` before SHA256 verify —
   `qemu-smoke.sh` must use `-snapshot`.
3. Reproduce locally: `bash scripts/vm-debug-build.sh` (full build + CI-identical verify + QEMU + post-QEMU checksum).
4. Fix scripts/workflow on Windows branch, push, confirm `build-os` green.
5. Log in [docs/operations/LOCAL_BUILDROOT_VM.md](../../docs/operations/LOCAL_BUILDROOT_VM.md).

## Windows scripts

```powershell
.\scripts\vm-ensure.ps1
.\scripts\vm-authorize-key.ps1       # once — required for VM
.\scripts\vm-ensure.ps1 -RunSetup
.\scripts\vm-ensure.ps1 -DebugBuild   # reproduce GH build-os on VM
.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>
.\scripts\vm-ensure.ps1 -RunBuild
```

WSL fallback (from repo root):

```powershell
wsl -e bash -lc "cd /mnt/c/Users/ben/Documents/diy-demand-side-management && bash scripts/vm-setup.sh && bash scripts/vm-debug-build.sh"
```

## vm-debug-build.sh

Same verification sequence as `.github/workflows/build-os.yml`:

1. `build-image.sh x86_64`
2. Verify SHA256SUMS (pristine)
3. `qemu-smoke.sh` with `-snapshot`
4. Verify SHA256SUMS again (must still pass)

Log: `$HOME/dbr-buildroot/x86_64-debug-*.log`

## Agent behavior

1. Read [docs/operations/LOCAL_BUILDROOT_VM.md](../../docs/operations/LOCAL_BUILDROOT_VM.md).
2. Diagnose GH failure before editing Buildroot configs.
3. Prefer script/workflow fixes that local debug proves.
4. Port proven fixes to `build-os.yml` in same PR.

## Related docs

- [docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md](../../docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md)
- [scripts/vm-debug-build.sh](../../scripts/vm-debug-build.sh)
- `.github/workflows/build-os.yml`
