---
name: local-buildroot-vm
description: >-
  Local Buildroot debug on VMware Ubuntu guest over SSH from Windows (no WSL).
  Requires config/vm.env with VM_SSH_PASSWORD. Use vm-debug-build.sh when build-os
  fails; port fixes to GitHub Actions. See docs/operations/LOCAL_BUILDROOT_VM.md.
---

# VMware Buildroot lab (Ubuntu guest)

**Do not use WSL** on this machine when it is corrupt. All Linux Buildroot builds
run on an **Ubuntu 24.04 guest in VMware**, accessed by **SSH from the Windows
host** (`ben@127.0.0.1:2222`).

## Topology

```text
Windows host  --SSH:2222-->  VMware  -->  Ubuntu guest  -->  build-image.sh / QEMU
```

Read [docs/operations/LOCAL_BUILDROOT_VM.md](../../docs/operations/LOCAL_BUILDROOT_VM.md)
for port forwarding, credentials, and directory layout.

## Buildroot pin

Always read [`config/buildroot-lock.toml`](../../config/buildroot-lock.toml).
Current stable pin: **2026.05.2** (`72d9d4fa…`, host Rust 1.96.1). Do not bump
without green x86 QEMU on CI and the lab VM.

## Credentials (required)

```powershell
copy config\vm.env.example config\vm.env
# Edit config\vm.env — set VM_SSH_PASSWORD
.\scripts\vm-authorize-key.ps1
```

## Workflow when GitHub Actions fails

1. `gh run view <RUN_ID> --log-failed`
2. Start Ubuntu guest in VMware; confirm SSH: `ssh -p 2222 ben@127.0.0.1`
3. `.\scripts\vm-ensure.ps1 -Hypervisor vmware -DebugBuild` or on guest: `bash scripts/vm-debug-build.sh`
4. Fix scripts/workflow on Windows branch; push
5. Confirm `build-os` green
6. `.\scripts\vm-ensure.ps1 -Hypervisor vmware -AcceptRunId <RUN_ID>`

## Scripts

| Script | Purpose |
|--------|---------|
| `vm-ensure.ps1` | Probe SSH (`-Hypervisor vmware|auto|virtualbox|none`, `-SkipVmStart`); run setup/debug/accept on guest |
| `vm-authorize-key.ps1` | One-time key via `config/vm.env` |
| `vm-setup.sh` | apt, node, rust, clone repo (on guest) |
| `vm-debug-build.sh` | Full x86 build + CI verify + QEMU + post-QEMU checksum |
| `vm-accept-artifact.sh` | Download GH artifact + QEMU (needs `gh` on guest or host) |
| `qemu-ui.sh` | Persistent QEMU UI preview (`start`/`status`/`stop`, `-snapshot`, loopback :18080) |

## Known fix (rootfs checksum)

QEMU must use `-snapshot` so `rootfs.ext2` is not modified before SHA256 verify.

Update [docs/operations/LOCAL_BUILDROOT_VM.md](../../docs/operations/LOCAL_BUILDROOT_VM.md)
after each lab run.
