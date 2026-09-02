# Local Buildroot VM notes

VirtualBox **ubuntu2** lab — **no WSL** (corrupt on this machine).

**Repo:** [bbartling/diy-bacnet-router](https://github.com/bbartling/diy-bacnet-router)  
**VM:** `ben@127.0.0.1:2222` (alias `ubuntu2-buildroot`)  
**Credentials:** `config/vm.env` (copy from `config/vm.env.example`, gitignored)

## Setup checklist

- [ ] Copy `config/vm.env.example` → `config/vm.env`
- [ ] Set `VM_SSH_PASSWORD` in `config/vm.env`
- [ ] `.\scripts\vm-authorize-key.ps1`
- [ ] `.\scripts\vm-ensure.ps1 -RunSetup`
- [ ] `.\scripts\vm-ensure.ps1 -DebugBuild` (local Buildroot, mirrors GH)
- [ ] Push fixes; wait for `build-os` green
- [ ] `.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>`

## GH failure → local debug (2026-09-02)

Run `33636224392`: build OK, verify failed — QEMU mutated `rootfs.ext2` before SHA256 check.  
Fix pushed: `qemu-smoke.sh -snapshot`. Prove on VM with `vm-debug-build.sh`.

## Commands

```powershell
.\scripts\vm-ensure.ps1
.\scripts\vm-authorize-key.ps1
.\scripts\vm-ensure.ps1 -RunSetup
.\scripts\vm-ensure.ps1 -DebugBuild
.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>
```

## Build / acceptance results

| Date | Run ID | SHA | Where | Result | Notes |
|------|--------|-----|-------|--------|-------|
| | | | | | |
