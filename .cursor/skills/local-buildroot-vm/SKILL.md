---
name: local-buildroot-vm
description: >-
  Run Buildroot image builds on the operator's Ubuntu VM from Windows/Cursor,
  keep notes in docs/operations/LOCAL_BUILDROOT_VM.md, and use GitHub Actions as
  the CI verifier. Use when experimenting with Buildroot, SSH into ubuntu2,
  comparing local builds to build-os.yml, or deciding where git push should run.
---

# Local Buildroot VM workflow

## Roles (do not blur these)

| Surface | Purpose |
|---------|---------|
| **Windows + Cursor** | Edit repo, run Rust/npm CI, `git commit`, `git push`, `gh`, open PRs |
| **Ubuntu VM (SSH)** | Heavy Buildroot builds only — mirrors `.github/workflows/build-os.yml` |
| **GitHub Actions** | Source of truth for green M0; upload artifacts and diagnostics |

Git lives on **Windows**. The VM is a build lab, not a second source-control home.

## VM access (VirtualBox headless)

The operator uses VirtualBox VM `ubuntu2` with SSH forwarded to localhost port 2222.
Windows scripts live under `scripts/`:

```powershell
.\scripts\vm-ensure.ps1              # start VM + probe SSH
.\scripts\vm-authorize-key.ps1       # one-time key install (password prompt)
.\scripts\vm-ensure.ps1 -RunSetup    # apt/node/rust + git clone on VM
.\scripts\vm-ensure.ps1 -RunBuild    # x86_64 Buildroot build on VM
```

Manual start if needed:

```powershell
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" startvm "ubuntu2" --type headless
Start-Sleep -Seconds 15
ssh ubuntu2-buildroot
```

## Agent behavior on Windows

1. Read `docs/operations/LOCAL_BUILDROOT_VM.md` for session notes before Buildroot work.
2. Prefer application CI on Windows: `cargo test`, `npm run check`, `scripts/validate-repository.sh`.
3. For Buildroot, push commits from Windows then either:
   - run `ssh ben@127.0.0.1 -p 2222 '<command>'` for one-off VM commands, or
   - ask the operator to start the VM if port 2222 is closed.
4. After a local VM build, trigger or watch `build-os` on GitHub and compare logs/manifests.
5. Append dated notes to `docs/operations/LOCAL_BUILDROOT_VM.md` (build times, failures, SHAs).

Non-interactive SSH requires key-based auth. If BatchMode fails, tell the operator to start the VM or fix keys — do not guess passwords.

## VM one-time setup

On the VM (after `ssh ben@127.0.0.1 -p 2222`):

```bash
sudo apt-get update
sudo apt-get install --yes --no-install-recommends \
  bc build-essential cpio file git libncurses-dev python3 rsync unzip wget xz-utils \
  curl ca-certificates

# Node 24 for frontend build (matches build-os.yml)
curl -fsSL https://deb.nodesource.com/setup_24.x | sudo -E bash -
sudo apt-get install --yes nodejs

# Rust 1.93.0 — use rustup; project pin is in rust-toolchain.toml
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.93.0
source "$HOME/.cargo/env"

git clone https://github.com/bbartling/diy-bacnet-router.git ~/src/diy-bacnet-router
cd ~/src/diy-bacnet-router
git fetch origin
git checkout luna-max/m0-buildroot-ci-repair   # or current feature branch
```

Refresh the VM clone after pushing from Windows:

```bash
cd ~/src/diy-bacnet-router && git fetch origin && git checkout <branch> && git pull
```

## Build on VM (matches CI)

```bash
cd ~/src/diy-bacnet-router
npm --prefix frontend/web ci && npm --prefix frontend/web run build
export BUILDROOT_VERSION=2025.02.17
export JOBS="$(nproc)"
export BUILD_WORK_ROOT="$HOME/dbr-buildroot"
bash scripts/build-image.sh x86_64
```

Optional QEMU smoke (x86_64 only):

```bash
sudo apt-get install --yes e2fsprogs qemu-system-x86 curl
bash scripts/qemu-smoke.sh "$HOME/dbr-buildroot/output/x86_64/images"
```

Pi targets (`rpi3_64`, `rpi4_64`, `rpi5_64`) build on the VM the same way; they are slow — use `workflow_dispatch` on GitHub for full matrix verification.

## When to use VM vs GitHub only

| Situation | Prefer |
|-----------|--------|
| Fast config/package tweak iteration | VM `x86_64` build |
| Full matrix + artifact retention | GitHub `build-os` workflow |
| Rust/npm/unit tests | Windows (or VM, but Windows is faster for agent) |
| First-time "does Buildroot work at all?" | VM, then push and confirm Actions |

## Evidence and notes

- Update `docs/operations/LOCAL_BUILDROOT_VM.md` with each experiment: date, branch, SHA, target, pass/fail, log path.
- Do not claim hardware or routing evidence from a VM build — same boundaries as `AGENTS.md`.
- Compare `build-manifest.json` and `buildroot-host-rustc-version.txt` between VM output and Actions artifacts.

## Related repo docs

- `AGENTS.md` — engineering contract
- `docs/agent/LUNA_MAX_GITHUB_ACTIONS_PROMPT.md` — M0 Actions focus
- `.github/workflows/build-os.yml` — CI reference for deps and steps
- `scripts/build-image.sh` — local/CI entry point
