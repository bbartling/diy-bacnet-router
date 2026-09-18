# DIY BACnet Router

<p align="center">
  <a href="https://bbartling.github.io/diy-bacnet-router/"><img src="https://img.shields.io/badge/docs-GitHub%20Pages-blue" alt="Docs"></a>
  <a href="https://github.com/bbartling/diy-bacnet-router/actions/workflows/ci.yml"><img src="https://github.com/bbartling/diy-bacnet-router/actions/workflows/ci.yml/badge.svg?branch=master" alt="CI"></a>
  <a href="https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml"><img src="https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml/badge.svg?branch=master" alt="build-os"></a>
  <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="MIT">
  <img src="https://img.shields.io/badge/Rust-1.93-orange?logo=rust&logoColor=white" alt="Rust 1.93">
  <img src="https://img.shields.io/badge/Buildroot-2026.05.2-blue" alt="Buildroot">
  <img src="https://img.shields.io/badge/forwarding-fail--closed-orange" alt="Forwarding fail-closed">
</p>

<p align="center">
  <a href="https://bbartling.github.io/diy-bacnet-router/"><img src="https://img.shields.io/badge/Docs-online-2563EB?style=for-the-badge" alt="Online docs"></a>
  <a href="docs/hardware/WAVESHARE_USB_RS485_C.md"><img src="https://img.shields.io/badge/Reference%20RS--485-Waveshare%20C-059669?style=for-the-badge" alt="Waveshare C"></a>
  <a href="VERSION"><img src="https://img.shields.io/badge/Release-VERSION%20file-6D28D9?style=for-the-badge" alt="VERSION"></a>
  <a href="config/upstream-lock.toml"><img src="https://img.shields.io/badge/rusty--bacnet-acbf7bae-0B7285?style=for-the-badge" alt="Upstream pin"></a>
</p>

**DIY BACnet Router** is an open-source **Linux appliance** that forwards BACnet
**IP ↔ MS/TP**. It is built as a small custom OS (Buildroot), a Rust data plane
(`routerd`), and an embedded React management UI for lab and education.

| | |
| --- | --- |
| **Boards** | x86-64 (QEMU / lab ISO) and Raspberry Pi 3 / 4 / 5 |
| **Serial** | USB RS-485 via `/dev/serial/by-id/...` (reference: Waveshare USB TO RS485 C) |
| **Default** | Forwarding **off**, management on loopback — fail-closed |
| **Docs** | Beginner tutorials on [GitHub Pages](https://bbartling.github.io/diy-bacnet-router/) |

**Status in one breath:** lab **source** routing (G6–G8 / shared-trunk ReadProperty)
is evidenced; flashing a **Buildroot image** and proving the same gates on that
image is still open; the install goal is a versioned **GitHub Release** you can
download from the docs site (not a short-lived Actions artifact).

| Pin | Lock | Value |
| --- | --- | --- |
| Rust (CI) | [`rust-toolchain.toml`](rust-toolchain.toml) | **1.93.0** |
| Buildroot | [`config/buildroot-lock.toml`](config/buildroot-lock.toml) | **2026.05.2** |
| rusty-bacnet | [`config/upstream-lock.toml`](config/upstream-lock.toml) | **`acbf7baefe69d05f2368763dcc659d68e4bc114c`** |
| Cargo.lock | committed | `--locked` in CI and Buildroot |

Badges track **`master`**. Open PRs run the same workflows on their branch.

---

<details>
<summary>Milestones</summary>

## Milestones

Details: [docs/agent/SPEC.md](docs/agent/SPEC.md). Gate ledger: [docs/TESTING.md](docs/TESTING.md).
Hold / pickup: [docs/evidence/CHECKPOINT_2026-09-13_SOURCE_G7_G8_HOLD.md](docs/evidence/CHECKPOINT_2026-09-13_SOURCE_G7_G8_HOLD.md).
Timing / serial follow-ups: [issue #66](https://github.com/bbartling/diy-bacnet-router/issues/66).

### Done (source / scaffold)

- [x] **M0 — Scaffold and OS images** — management API/UI, CI, Buildroot x86+Pi, QEMU smoke
- [x] **M1 — rusty-bacnet adapter closeout** — pin + concrete B/IP/MS/TP compile/config fixtures (transports not started at ordinary boot)
- [x] **M2A — B/IP port qualification (G6)** — netns BVLL oracle + unicast/directed-broadcast matrix PASS
- [x] **M2B — Physical MS/TP port qualification (source)** — isolated two-Pi passive RX + `--mstp-qualify` join @ **38400** (net 2001); `--mstp-passive` fail-closed in tree. Exact-image M2B still OPEN
- [x] **M3 — Isolated routing (source G7/G8)** — dual-B/IP CI + `--route-enable`; evidence under `docs/evidence/SOURCE_G7_G8_*`. Lab persist: `--qualify-secs 0` + [ansible/](ansible/)
- [x] **M3b — Shared trunk routed ReadProperty (source)** — Waveshare lab trunk @38400; bacpypes3 → `2001:2` + `2001:7` **PASS** on tip `acbf7bae` ([PHASE2_FEC_VIA_DIY_20260918T124240Z](docs/evidence/PHASE2_FEC_VIA_DIY_20260918T124240Z/)). Product `ready_to_route` flip still gated

### Open (lab → appliance → install)

- [ ] **M4 — Exact-image G7/G8** — same topology/oracle on a **Buildroot** appliance image (not host-built source binaries)
- [ ] **M5 — Faults and timing** — CI software faults + dual-B/IP link-down survival; **G9** USB/serial stop ownership + hardware faults **OPEN** ([#66](https://github.com/bbartling/diy-bacnet-router/issues/66))
- [ ] **M6 — Production-shaped Pi flash/boot** — Pi **build** evidence in Actions; **flash/boot + on-device prove OPEN**
- [ ] **M7 — Management writes** — auth/session skeleton + env unlock only; `POST /api/config` stays 403 until capability unlocked
- [ ] **M8 — GitHub Releases for images** — `build-os` attaches versioned `rootfs.iso` / `sdcard.img.xz` + checksums; docs site links `…/releases/download/vX/…` so install is “download → flash → boot”
- [ ] **M9 — Optional local Buildroot only** — Releases + QEMU/Pi flash cover normal use; Windows/VMware guest kept for optional Workbench UI evidence / debug builds

**Source vs exact-image:** checked boxes for M2B/M3 are **source** unless marked exact-image. Do not claim product G7–G11 PASS without the Buildroot image gate (**M4**). Do not claim “easy install” until **M8**.

</details>

<details>
<summary>Architecture</summary>

## Architecture

```text
  BACnet/IP (UDP)          routerd (Rust)           MS/TP (USB RS-485)
  host NIC / veth    <-->  NPDU forwarder    <-->  generic adapter
                               |
                    atomics + bounded channel
                               |
              Axum REST + OpenAPI + WebSocket (management only)
                               |
                    React dashboard (static, embedded)
```

The **data plane** owns token timing and forwarding; the **management plane** must never block MS/TP or B/IP. Browser updates are **aggregate snapshots** (~1 Hz), not one WebSocket message per frame.

Host IP/routes: **SSH** + normal Linux tools. App policy: `/etc/diy-bacnet-router/router.toml`.

</details>

<details>
<summary>Develop / run</summary>

## Develop / run

```bash
cp config/router.example.toml config/router.toml
cargo run -p routerd -- --config config/router.toml
```

Open <http://127.0.0.1:8080> (or `DBR_BIND`).

- `GET /healthz` — `ready_to_route` stays false until routing gates pass
- `GET /api/status` · `/api/capabilities` · `/api/metrics/snapshot`
- `GET /api/openapi.json` · **`GET /api/ws/metrics`** · `GET /metrics`

```bash
npm --prefix frontend/web ci
npm --prefix frontend/web run check
npm --prefix frontend/web run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
bash scripts/validate-repository.sh
```

</details>

<details>
<summary>Configuration</summary>

## Configuration

See [`config/router.example.toml`](config/router.example.toml).

| Topic | Policy |
| --- | --- |
| Serial | `/dev/serial/by-id/...` only — never persist `ttyUSB0` |
| Baud | 9600…115200 (default **38400**) |
| Networks | B/IP and MS/TP numbers **distinct** (1–65534) |
| Forwarding | **Off by default** (`router.enabled = false`) |

Reference adapter: [docs/hardware/WAVESHARE_USB_RS485_C.md](docs/hardware/WAVESHARE_USB_RS485_C.md).

Overrides: `DBR_CONFIG`, `DBR_BIND`, `DBR_WEB_ROOT`, `RUST_LOG`.

</details>

<details>
<summary>Build appliance images</summary>

## Build appliance images

Workflow **[build-os](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml)** builds:

- `x86_64` — QEMU boot smoke + SHA256 verify
- `rpi3_64` · `rpi4_64` · `rpi5_64` — `sdcard.img` + manifest

**Today:** images land as **Actions artifacts** (short retention). **Goal (M8):** the same files on **GitHub Releases**, linked from the docs site for a simple download → flash → boot path.

What the image contains: [Buildroot recipe](https://bbartling.github.io/diy-bacnet-router/learn/buildroot-recipe/). Optional local debug: [docs/operations/LOCAL_BUILDROOT_VM.md](docs/operations/LOCAL_BUILDROOT_VM.md).

```powershell
.\scripts\vm-ensure.ps1 -Hypervisor vmware -AcceptRunId <RUN_ID>
# guest: bash scripts/qemu-ui.sh start <images-dir>
# Windows: ssh -N -o ExitOnForwardFailure=yes -L 127.0.0.1:18080:127.0.0.1:18080 ubuntu2-buildroot
# Browser: http://127.0.0.1:18080
```

</details>

<details>
<summary>What we claim (and do not)</summary>

## What we claim (and do not)

| Claim | Status |
| --- | --- |
| Open-source IP↔MS/TP router **intent** + appliance architecture | Yes |
| Reproducible Buildroot images + management UI | **M0** (CI artifacts today; **M8** = Release downloads) |
| Pinned rusty-bacnet + fail-closed adapter crate | **Yes** (ordinary boot does not start forwarding) |
| Versioned Release image download from docs | **No** until **M8** |
| Field-ready routing, BTL, Clause 9 | **No** |
| QEMU/unit tests = live RS-485 trunk | **No** |
| Local Buildroot VM required to use the product | **No** (lab/debug only; **M9**) |

**Agents:** [AGENTS.md](AGENTS.md) · [SOFTWARE_SPEC](docs/agent/SOFTWARE_SPEC.md) · [FULL_STACK_AUDIT](docs/agent/FULL_STACK_AUDIT.md) · [SPEC](docs/agent/SPEC.md)

</details>

<details>
<summary>Support DIY BACnet Router</summary>

If this project saves you time or helps with BAS / BACnet lab work, you can support continued open-source development through PayPal.

<p align="center">
  <a href="https://paypal.me/benbartling20/25"><img src="https://img.shields.io/badge/Donate-$25-0070BA?style=for-the-badge&logo=paypal&logoColor=white" alt="Donate $25 via PayPal"></a>
  <a href="https://paypal.me/benbartling20/50"><img src="https://img.shields.io/badge/Donate-$50-0070BA?style=for-the-badge&logo=paypal&logoColor=white" alt="Donate $50 via PayPal"></a>
  <a href="https://paypal.me/benbartling20/250"><img src="https://img.shields.io/badge/Donate-$250-0070BA?style=for-the-badge&logo=paypal&logoColor=white" alt="Donate $250 via PayPal"></a>
  <a href="https://paypal.me/benbartling20"><img src="https://img.shields.io/badge/Donate-Custom%20Amount-0070BA?style=for-the-badge&logo=paypal&logoColor=white" alt="Choose a custom PayPal donation amount"></a>
</p>

The repository Sponsor button uses [paypal.me/benbartling20](https://paypal.me/benbartling20).

</details>

## License

MIT — see [LICENSE](LICENSE).
