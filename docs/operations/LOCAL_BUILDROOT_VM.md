# Local Buildroot VM notes

VirtualBox **ubuntu2** lab — **no WSL** (corrupt on this machine).

**Repo:** [bbartling/diy-bacnet-router](https://github.com/bbartling/diy-bacnet-router)  
**VM:** `ben@127.0.0.1:2222` (alias `ubuntu2-buildroot`)  
**Credentials:** `config/vm.env` (gitignored)

## Setup checklist

- [x] Copy `config/vm.env.example` → `config/vm.env`
- [x] Set `VM_SSH_PASSWORD` in `config/vm.env`
- [x] `.\scripts\vm-authorize-key.ps1`
- [x] `.\scripts\vm-ensure.ps1 -RunSetup`
- [x] `vm-debug-build.sh` on ubuntu2 — x86_64 `bzImage` produced (2026-09-02)
- [x] PR #36 merged to `master`; feature branch deleted
- [x] Post-merge `ci` green (run `33655367771`)
- [ ] Post-merge `build-os` green on `master` (run `33655367731`, in progress)
- [ ] `.\scripts\vm-ensure.ps1 -AcceptRunId 33646331873` (requires `gh auth` on VM or Windows-side download)

## GH failure → local debug (2026-09-02)

Run `33636224392`: build OK, verify failed — QEMU mutated `rootfs.ext2` before SHA256 check.  
Fix: `qemu-smoke.sh -snapshot` + `scripts/test-image-evidence-contract.sh`.

## Build / acceptance results

| Date | Run ID | SHA | Where | Result | Notes |
|------|--------|-----|-------|--------|-------|
| 2026-09-02 | 33646331873 | acc2fa9 | GitHub Actions | PASS | x86_64 image + QEMU smoke (~1h4m) |
| 2026-09-02 | — | 6cde350 | ubuntu2 | PASS | vm-debug-build.sh; bzImage @ 11:18 local |

## Contract tests (offline)

```bash
bash scripts/test-image-evidence-contract.sh
bash scripts/test-appliance-contract.sh
bash scripts/validate-repository.sh
```

## Commands

```powershell
.\scripts\vm-ensure.ps1 -DebugBuild
.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>
```
