# Local Buildroot VM notes

Persistent log for Buildroot and **M0 artifact acceptance** on VirtualBox
`ubuntu2`, accessed from Windows via SSH. The Cursor agent reads and updates
this file to keep context across sessions.

**Repo:** [bbartling/diy-bacnet-router](https://github.com/bbartling/diy-bacnet-router)  
**Windows workspace:** `C:\Users\ben\Documents\diy-demand-side-management`  
**VM:** VirtualBox `ubuntu2` (8 CPUs, ~24 GB RAM), SSH `ben@127.0.0.1:2222`  
**Prototype bench:** `py-bacnet-stacks-playground/vibe_code_apps_13`  
**Branch:** `luna-max/m0-buildroot-ci-repair`

## Workflow (GH failure → local debug → port fix)

| Phase | Action |
|-------|--------|
| 1 | SSH key: `.\scripts\vm-authorize-key.ps1` |
| 2 | VM setup: `.\scripts\vm-ensure.ps1 -RunSetup` |
| 3 | **Debug build:** `.\scripts\vm-ensure.ps1 -DebugBuild` (or WSL below) |
| 4 | Fix scripts/workflow on Windows, push, confirm `build-os` green |
| 5 | Accept artifact: `.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>` |

### Known GH failure (2026-09-02)

Run `33636224392` built successfully but **Verify image evidence** failed:
`rootfs.ext2: FAILED` in SHA256SUMS — QEMU boot wrote to rootfs before verify.
Fix: `qemu-smoke.sh` now uses `-snapshot`. Local proof: `vm-debug-build.sh`.

### WSL fallback (when VM SSH not ready)

```powershell
wsl -e bash -lc "cd /mnt/c/Users/ben/Documents/diy-demand-side-management && bash scripts/vm-setup.sh && bash scripts/vm-debug-build.sh"
```

## Quick commands (Windows)

```powershell
.\scripts\vm-ensure.ps1
.\scripts\vm-authorize-key.ps1
.\scripts\vm-ensure.ps1 -RunSetup
.\scripts\vm-ensure.ps1 -AcceptRunId 33642454599
.\scripts\vm-ensure.ps1 -RunBuild
```

## Agent docs

- [docs/agent/SOFTWARE_SPEC.md](../agent/SOFTWARE_SPEC.md) — product + UI spec
- [docs/product/BASRT_EDUCATIONAL_REFERENCE.md](../product/BASRT_EDUCATIONAL_REFERENCE.md)
- [.cursor/skills/local-buildroot-vm/SKILL.md](../../.cursor/skills/local-buildroot-vm/SKILL.md)
- [.cursor/skills/basrt-educational-router/SKILL.md](../../.cursor/skills/basrt-educational-router/SKILL.md)

## VM status (2026-09-02)

| Check | Result |
|-------|--------|
| VM starts headless | OK |
| Port 2222 open | OK |
| SSH key auth | **Run `vm-authorize-key.ps1` once** |
| `vm-accept-artifact.sh` | Ready |
| `gh` on VM | Required for artifact download |

## Build / acceptance results

| Date | Run ID | SHA | Target | Where | Result | Notes |
|------|--------|-----|--------|-------|--------|-------|
| | | | | | | |
