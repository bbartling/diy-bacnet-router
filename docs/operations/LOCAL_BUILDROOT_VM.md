---
title: VMware Buildroot lab
parent: Operations
nav_order: 1
permalink: /operations/local-buildroot-vm/
---

# Local Buildroot lab — VMware Ubuntu guest

Linux image builds and QEMU smoke tests run on an **Ubuntu guest VM** reached by
**SSH from the Windows host**. **Do not use WSL** on this machine when it is
corrupt or unavailable — the lab VM is the canonical Buildroot environment.

**Repo:** [bbartling/diy-bacnet-router](https://github.com/bbartling/diy-bacnet-router)

## Topology

```text
  Windows host (Cursor, git, gh CLI)
        |
        |  SSH  ben@127.0.0.1:2222  (NAT port forward)
        v
  VMware Workstation / Player
        |
        v
  Ubuntu 24.04 LTS guest  (~8 vCPU, ~24 GB RAM recommended)
        |
        +-- ~/src/diy-bacnet-router     (git clone)
        +-- ~/dbr-buildroot/              (Buildroot work root, artifacts)
        +-- scripts/vm-debug-build.sh   (full x86 parity with CI)
```

| Item | Value |
| --- | --- |
| Hypervisor | **VMware** (guest name e.g. `ubuntu2`) |
| Guest OS | Ubuntu 24.04 LTS amd64 |
| SSH target | `ben@127.0.0.1:2222` (alias `ubuntu2-buildroot` in `~/.ssh/config`) |
| Credentials | `config/vm.env` on Windows host (**gitignored**) |
| Buildroot pin | [`config/buildroot-lock.toml`](../config/buildroot-lock.toml) — **2026.05.2** |

### VMware networking

1. Guest uses **NAT**.
2. **Port forward** host `2222` → guest `22` (VMware VM Settings → Network Adapter
   → NAT → Advanced → Port Forwarding).
3. From PowerShell on the host: `ssh -p 2222 ben@127.0.0.1`.

Optional: install OpenSSH server on the guest (`sudo apt install openssh-server`).

## One-time host setup (Windows)

```powershell
copy config\vm.env.example config\vm.env
# Edit config\vm.env — set VM_SSH_PASSWORD to the Ubuntu user password
.\scripts\vm-authorize-key.ps1
# Start the Ubuntu guest in VMware first, then:
.\scripts\vm-ensure.ps1 -Hypervisor vmware -RunSetup
```

`vm-setup.sh` on the guest installs apt build deps, Node 24, Rust 1.93, QEMU tools,
and clones the repo to `~/src/diy-bacnet-router`.

`vm-ensure.ps1` defaults to `-Hypervisor auto`: if SSH on port 2222 is already
open it skips hypervisor start; otherwise it uses VirtualBox only when
`VBoxManage` is installed. On VMware-only hosts use `-Hypervisor vmware` or
`-SkipVmStart` (never touches VirtualBox).

## Daily agent workflow

### When GitHub `build-os` fails

1. `gh run view <RUN_ID> --log-failed` on the host.
2. On the guest (via SSH):

   ```bash
   cd ~/src/diy-bacnet-router && git pull --ff-only
   bash scripts/vm-debug-build.sh
   ```

   Or from Windows: `.\scripts\vm-ensure.ps1 -Hypervisor vmware -DebugBuild`

3. Fix `scripts/build-image.sh`, `build-os.yml`, or Buildroot external tree on
   the host branch; commit and push.
4. Wait for green `build-os` on GitHub.
5. Artifact acceptance:
   `.\scripts\vm-ensure.ps1 -Hypervisor vmware -AcceptRunId <RUN_ID>`
   (requires `gh auth` on the guest; otherwise download on the Windows host with
   `gh run download` and copy images into the guest).

### Parity with CI

`vm-debug-build.sh` runs the same steps as Actions: frontend build (via archive),
`build-image.sh x86_64`, SHA256SUMS verify, `qemu-smoke.sh -snapshot`, post-QEMU
checksum verify.

### Persistent QEMU UI preview (lab)

After artifact acceptance (or a local image build), on the guest:

```bash
bash scripts/qemu-ui.sh start /path/to/images   # directory containing bzImage + rootfs.ext2
bash scripts/qemu-ui.sh status
# Windows host tunnel, then browser:
#   ssh -N -L 18080:127.0.0.1:18080 ubuntu2-buildroot
#   http://127.0.0.1:18080
bash scripts/qemu-ui.sh stop
```

`qemu-ui.sh` uses `-snapshot` and `hostfwd=tcp:127.0.0.1:18080-:8080`. It never
mutates `rootfs.ext2`. Keep `qemu-smoke.sh` for short-lived CI/acceptance smoke.

## Build / acceptance log

| Date | Run ID | SHA | Where | Result | Notes |
| --- | --- | --- | --- | --- | --- |
| 2026-09-03 | 33671378385 | 3c03e30 | VMware ubuntu2 QEMU | PASS | host download + scp; SHA256SUMS OK; smoke `/healthz` ok, data_plane disabled, uid=100; lab needed `noapic` |
| 2026-09-02 | 33671378385 | 3c03e30 | GitHub Actions | PASS | tip master; x86 + rpi3/4/5 |
| 2026-09-02 | 33646331873 | acc2fa9 | GitHub Actions | PASS | x86_64 + QEMU (~1h4m) |
| 2026-09-02 | — | 6cde350 | VMware ubuntu2 | PASS | local `vm-debug-build.sh`; `bzImage` |

## Offline contract tests (host or guest)

```bash
bash scripts/test-image-evidence-contract.sh
bash scripts/test-appliance-contract.sh
bash scripts/validate-repository.sh
```

## Useful SSH commands

```bash
# Tail a long Buildroot job
tail -f ~/dbr-buildroot/vm-debug-nohup.log

# Confirm Buildroot pin inside clone
grep -E '^version|^commit' ~/src/diy-bacnet-router/config/buildroot-lock.toml

# Quick SSH probe from host
ssh -o BatchMode=yes -p 2222 ben@127.0.0.1 'uname -a && rustc --version'
```
