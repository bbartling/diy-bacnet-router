---
name: local-buildroot-vm
description: >-
  Local Buildroot debug on VirtualBox ubuntu2 only (no WSL). Requires config/vm.env
  with VM_SSH_PASSWORD. Use vm-debug-build.sh when build-os fails; port fixes to
  GitHub Actions. See docs/operations/LOCAL_BUILDROOT_VM.md.
---

# VirtualBox Buildroot lab (ubuntu2)

**Do not use WSL** on this machine — it is corrupt. All Linux builds run on
VirtualBox `ubuntu2` via SSH from Windows.

## Credentials (required)

```powershell
copy config\vm.env.example config\vm.env
# Edit config\vm.env — set VM_SSH_PASSWORD
.\scripts\vm-authorize-key.ps1
```

Files:

- `config/vm.env.example` — template (committed)
- `config/vm.env` — secrets (gitignored)
- `scripts/vm-load-env.ps1` — loads env for other scripts
- `scripts/vm-ssh-install-key.py` — paramiko key install

## Workflow when GitHub Actions fails

1. `gh run view <RUN_ID> --log-failed`
2. `.\scripts\vm-ensure.ps1 -DebugBuild` on VM
3. Fix scripts/workflow on Windows branch
4. Push; confirm `build-os` green
5. `.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>`

## Scripts

| Script | Purpose |
|--------|---------|
| `vm-ensure.ps1` | Start VM, probe SSH |
| `vm-authorize-key.ps1` | One-time key via `config/vm.env` |
| `vm-setup.sh` | apt, node, rust, clone repo |
| `vm-debug-build.sh` | Full x86 build + CI verify + QEMU + post-QEMU checksum |
| `vm-build-x86.sh` | Build only |
| `vm-accept-artifact.sh` | Download GH artifact + QEMU |

## Known fix (rootfs checksum)

QEMU must use `-snapshot` so `rootfs.ext2` is not modified before SHA256 verify
in `build-os.yml`.

## Track progress

Update [docs/operations/LOCAL_BUILDROOT_VM.md](../../docs/operations/LOCAL_BUILDROOT_VM.md)
checklist and results table after each run.
