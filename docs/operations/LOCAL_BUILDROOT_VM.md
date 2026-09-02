# Local Buildroot VM notes

Persistent log for Buildroot experiments on the operator's Ubuntu VM, accessed from
Windows via SSH. The Cursor agent reads and updates this file to keep context across
sessions.

**Repo:** [bbartling/diy-bacnet-router](https://github.com/bbartling/diy-bacnet-router)  
**Windows workspace:** `C:\Users\ben\Documents\diy-demand-side-management`  
**VM:** VirtualBox `ubuntu2` (8 CPUs, ~24 GB RAM), headless, SSH `ben@127.0.0.1:2222`  
**SSH alias:** `ubuntu2-buildroot` (in `~/.ssh/config`)  
**Branch:** `luna-max/m0-buildroot-ci-repair`

## Quick commands (Windows)

```powershell
# Start VM + check SSH
.\scripts\vm-ensure.ps1

# ONE-TIME: authorize Windows SSH key (prompts for VM password)
.\scripts\vm-authorize-key.ps1

# After key auth: install deps + clone repo on VM
.\scripts\vm-ensure.ps1 -RunSetup

# Long-running: x86_64 Buildroot build on VM
.\scripts\vm-ensure.ps1 -RunBuild
```

## Workflow decision

| Where | Git commit/push | Buildroot | Rust/npm CI |
|-------|-----------------|-----------|-------------|
| Windows + Cursor | **Yes — primary** | No | **Yes** |
| Ubuntu VM | Pull only | **Yes — local iteration** | Optional |
| GitHub Actions | N/A | **Yes — CI truth** | Yes (`ci.yml`) |

## Windows toolchain verified (2026-09-02)

| Tool | Version |
|------|---------|
| rustc / cargo | 1.93.0 |
| node / npm | v24.14.0 / 10.9.2 |
| git | 2.47.1 |
| gh | 2.88.1 (logged in as bbartling) |
| VirtualBox | 7.2.16 |

Checks run: `cargo test --workspace --locked` pass; `npm ci && npm run check` pass after `npm ci`.

## VM status (2026-09-02)

| Check | Result |
|-------|--------|
| VM starts headless | OK |
| Port 2222 open | OK |
| SSH key auth (BatchMode) | **Blocked — run `vm-authorize-key.ps1` once** |
| VBox guestcontrol | Not ready / guest additions unavailable |
| VM setup script | Ready (`scripts/vm-setup.sh`) |
| VM build script | Ready (`scripts/vm-build-x86.sh`) |

## Session log

### 2026-09-02 — bootstrap + tooling

- Started `ubuntu2` headless; NAT forward `127.0.0.1:2222 → guest:22` confirmed.
- SSH reachable but Windows `id_rsa.pub` not in VM `authorized_keys` yet.
- Added scripts: `vm-ensure.ps1`, `vm-authorize-key.ps1`, `vm-setup.sh`, `vm-build-x86.sh`.
- Appended `ubuntu2-buildroot` to Windows `~/.ssh/config`.
- Added agent skill: `.cursor/skills/local-buildroot-vm/SKILL.md`.
- GitHub `ci` green on PR; `build-os` re-run in progress (run `33636224392`).

**Next operator step:** run `.\scripts\vm-authorize-key.ps1` in PowerShell, enter VM password once, then tell the agent to run `.\scripts\vm-ensure.ps1 -RunSetup`.

## Build results

| Date | SHA | Target | Where | Result | Notes |
|------|-----|--------|-------|--------|-------|
| | | | | | |
