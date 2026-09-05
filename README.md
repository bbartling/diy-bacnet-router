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
  <a href="config/upstream-lock.toml"><img src="https://img.shields.io/badge/rusty--bacnet-24e3439-0B7285?style=for-the-badge" alt="Upstream pin"></a>
</p>

**DIY BACnet Router** is an open-source Linux appliance that routes BACnet **IP to MS/TP** — Buildroot OS images, a Rust data plane (`routerd`), and an embedded React management UI for lab and education use (BASRT-class intent, original implementation).

Boards today: **x86-64** (lab/QEMU) and **Raspberry Pi 3/4/5**. MS/TP uses USB RS-485 adapters via `/dev/serial/by-id/...` (reference: Waveshare USB TO RS485 C).

> **Today:** Milestone **0** complete (images + management). rusty-bacnet is **pinned** and a fail-closed adapter crate exists (`24e3439…`). **NPDU forwarding stays disabled** until M2/M3 evidence. Concrete B/IP + MS/TP port fixtures (M1 closeout) and port qualification (M2) are next.

| Pin | Lock | Value |
| --- | --- | --- |
| Rust (CI) | [`rust-toolchain.toml`](rust-toolchain.toml) | **1.93.0** |
| Buildroot | [`config/buildroot-lock.toml`](config/buildroot-lock.toml) | **2026.05.2** |
| rusty-bacnet | [`config/upstream-lock.toml`](config/upstream-lock.toml) | **`24e3439694b7d286e57e0a80cf7f1df4bd39d8ad`** |
| Cargo.lock | committed | `--locked` in CI and Buildroot |

Badges track **`master`**. Open PRs run the same workflows on their branch.

---

<details>
<summary>Milestones</summary>

## Milestones

Details: [docs/agent/SPEC.md](docs/agent/SPEC.md).

- [x] **M0 — Scaffold and OS images** — management API/UI, CI, Buildroot x86+Pi, QEMU smoke
- [ ] **M1 — rusty-bacnet adapter closeout** — pin present; loopback fixture present; **concrete B/IP + MS/TP compile/config fixtures** still required before marking M1 done
- [ ] **M2 — Port qualification** — M2A B/IP on Linux netns; M2B physical MS/TP (separate)
- [ ] **M3 — Isolated routing** — NPDU forwarding between distinct networks
- [ ] **M4 — Faults and timing**
- [ ] **M5 — Production-shaped images / Pi hardware validation**
- [ ] **M6 — Management writes** — auth, audit, optional BBMD/FDR/TLS

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

Artifacts: images, checksums, legal-info, `build-manifest.json`.

Local lab (VMware Ubuntu guest, not WSL): [docs/operations/LOCAL_BUILDROOT_VM.md](docs/operations/LOCAL_BUILDROOT_VM.md).

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
| Reproducible Buildroot images + management UI | **M0** |
| Pinned rusty-bacnet + fail-closed adapter crate | **Yes** (loopback fixture; transports not started at ordinary boot) |
| Field-ready routing, BTL, Clause 9 | **No** |
| QEMU/unit tests = live RS-485 trunk | **No** |

Educational UI patterns only: [docs/product/BASRT_EDUCATIONAL_REFERENCE.md](docs/product/BASRT_EDUCATIONAL_REFERENCE.md).

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
