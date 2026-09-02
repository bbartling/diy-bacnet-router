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
- [ ] `.\scripts\vm-ensure.ps1 -DebugBuild` — **running via nohup on VM**
- [ ] Push fixes; wait for `build-os` green (run `33646208419` @ `6cde350`)
- [ ] Merge PR #36; delete branch
- [ ] `.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>`

## GH failure → local debug (2026-09-02)

Run `33636224392`: build OK, verify failed — QEMU mutated `rootfs.ext2` before SHA256 check.  
Fix: `qemu-smoke.sh -snapshot` + `scripts/test-image-evidence-contract.sh`.

## VM build log

```bash
ssh ubuntu2-buildroot 'tail -f ~/dbr-buildroot/vm-debug-nohup.log'
```

## Build / acceptance results

| Date | Run ID | SHA | Where | Result | Notes |
|------|--------|-----|-------|--------|-------|
| 2026-09-02 | — | 6cde350 | ubuntu2 | in_progress | vm-debug-build.sh nohup |

## Commands

```powershell
.\scripts\vm-ensure.ps1 -DebugBuild
.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>
```
