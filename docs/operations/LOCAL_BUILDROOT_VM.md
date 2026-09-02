# Local Buildroot VM notes

Persistent log for Buildroot and **M0 artifact acceptance** on VirtualBox
`ubuntu2`, accessed from Windows via SSH. The Cursor agent reads and updates
this file to keep context across sessions.

**Repo:** [bbartling/diy-bacnet-router](https://github.com/bbartling/diy-bacnet-router)  
**Windows workspace:** `C:\Users\ben\Documents\diy-demand-side-management`  
**VM:** VirtualBox `ubuntu2` (8 CPUs, ~24 GB RAM), SSH `ben@127.0.0.1:2222`  
**Prototype bench:** `py-bacnet-stacks-playground/vibe_code_apps_13`  
**Branch:** `luna-max/m0-buildroot-ci-repair`

## Workflow (artifact-first)

| Phase | Action |
|-------|--------|
| 1 | SSH key auth: `.\scripts\vm-authorize-key.ps1` |
| 2 | VM setup: `.\scripts\vm-ensure.ps1 -RunSetup` |
| 3 | **Accept Actions artifact:** `.\scripts\vm-ensure.ps1 -AcceptRunId <RUN_ID>` |
| 4 | Optional reproducibility: `.\scripts\vm-ensure.ps1 -RunBuild` |

Do **not** debug Buildroot until a successful Actions x86 artifact boots under
QEMU locally. See [docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md](../agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md).

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
