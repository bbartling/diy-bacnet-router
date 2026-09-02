# DIY BACnet Router

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![ci](https://github.com/bbartling/diy-bacnet-router/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/bbartling/diy-bacnet-router/actions/workflows/ci.yml)
[![build-os](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml/badge.svg?branch=master)](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml)

| Pin | Lock file | Current value |
| --- | --- | --- |
| Rust (CI / dev) | [`rust-toolchain.toml`](rust-toolchain.toml) | **1.93.0** |
| Buildroot (appliance images) | [`config/buildroot-lock.toml`](config/buildroot-lock.toml) | **2026.05.2** (`72d9d4fa…`) |
| Buildroot host Rust (image build) | `build-manifest.json` / CI log | **1.96.1** (from Buildroot `package/rust`) |
| rusty-bacnet | [`config/upstream-lock.toml`](config/upstream-lock.toml) | **Not integrated in M0** — observed `dev` tip `65ae4633…` (unaudited; M1 gate) |
| Cargo.lock | committed | `--locked` in CI and Buildroot |

Badges reflect **`master`** branch status. Open PRs run the same workflows on their branch before merge.

**DIY BACnet Router** is an open-source project to build a dedicated Linux
appliance that routes BACnet **IP to MS/TP** — a custom operating system and
application stack for network programming, commissioning, and field use. The end
goal is a reproducible, Home Assistant OS–style appliance image: a minimal
Buildroot-based Linux tuned for **MS/TP timing**, a Rust routing data plane, and
a built-in **React** web UI served from the same binary — no separate container
stack required.

Supported boards today: **x86-64** (lab/QEMU) and **Raspberry Pi 3/4/5**. MS/TP
uses generic **USB RS-485 adapters** identified by stable
`/dev/serial/by-id/...` paths and adapter profiles in configuration (FTDI, CH340,
and similar isolated or bus-powered adapters). The project ships with a documented
reference bench setup; additional adapters are validated through the milestone
gates rather than hard-coded to one vendor.

> **Today:** Milestone **0** — management scaffold and OS image pipeline. The
> device boots, serves the web app, and exposes metrics APIs, but **does not yet
> forward BACnet NPDUs**. Forwarding stays disabled until adapter and routing
> milestones pass with evidence.

## Milestones

Progress toward a working routing prototype (details in [docs/agent/SPEC.md](docs/agent/SPEC.md)):

- [x] **M0 — Scaffold and OS images** — Rust workspace, React UI, read-only REST/OpenAPI/WebSocket, config validation, CI, Buildroot images (x86 + Pi), QEMU smoke, checksums and manifests
- [ ] **M1 — rusty-bacnet adapter** — audit upstream, pin SHA, compile fixture using public B/IP and MS/TP APIs (no forked stack internals)
- [ ] **M2 — Port qualification** — B/IP on Linux networking; MS/TP passive decode and token behavior on USB RS-485 (no forwarding yet)
- [ ] **M3 — Isolated routing** — NPDU forwarding between distinct BACnet networks on a bench; routed Who-Is / ReadProperty both directions
- [ ] **M4 — Faults and timing** — unplug, duplicate MAC, baud mismatch, load, and worst-case MS/TP timing characterization
- [ ] **M5 — Production-shaped images** — unprivileged service, SSH recovery, legal-info/SBOM; Pi hardware validation
- [ ] **M6 — Management writes** — authenticated config, audit trail, optional BBMD/FDR/TLS (each behind its own gate)

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

The **data plane** owns token timing and forwarding; the **management plane**
(HTTP, WebSocket, dashboard) must never block MS/TP or B/IP packet handling.
Counters are atomic; the browser receives **aggregate snapshots** (about 1 Hz),
not one WebSocket message per frame.

Host IP, routes, and DNS are configured over **SSH** with normal Linux tools.
Application policy (network numbers, serial path, baud, adapter profile) lives in
`/etc/diy-bacnet-router/router.toml`.

## Run locally (development)

```bash
cp config/router.example.toml config/router.toml
cargo run -p routerd -- --config config/router.toml
```

Open <http://127.0.0.1:8080>. Key endpoints:

- `GET /healthz` — honest readiness (`ready_to_route` stays false until routing gates pass)
- `GET /api/v1/status` · `GET /api/v1/capabilities` · `GET /api/v1/metrics/snapshot`
- `GET /api/v1/openapi.json` · `GET /api/v1/ws/metrics` · `GET /metrics`

Build the frontend:

```bash
npm --prefix frontend/web ci
npm --prefix frontend/web run check
npm --prefix frontend/web run build
```

## Configuration

Appliance config: `/etc/diy-bacnet-router/router.toml` (see
[`config/router.example.toml`](config/router.example.toml)).

| Topic | Policy |
| --- | --- |
| Serial device | `/dev/serial/by-id/...` only — never persist `ttyUSB0` |
| Baud rates | 9600, 19200, 38400, 57600, 76800, 115200 (default **38400**) |
| Adapter | Profile string + termination model in TOML; validate on generic USB RS-485 hardware |
| BACnet networks | B/IP and MS/TP numbers must be **distinct** (1–65534) |
| Forwarding | **Off by default** until M3+ evidence (`router.enabled = false`) |

Reference hardware notes (one validated bench adapter):
[docs/hardware/WAVESHARE_USB_RS485_C.md](docs/hardware/WAVESHARE_USB_RS485_C.md).

Deployment overrides: `DBR_CONFIG`, `DBR_BIND`, `DBR_WEB_ROOT`, `RUST_LOG`.

## Build appliance images

Workflow **[build-os](https://github.com/bbartling/diy-bacnet-router/actions/workflows/build-os.yml)**
runs `scripts/build-image.sh` for:

- `x86_64` — kernel + rootfs; **QEMU boot smoke** + SHA256 verify
- `rpi3_64` · `rpi4_64` · `rpi5_64` — `sdcard.img` + manifest

Buildroot is pinned in [`config/buildroot-lock.toml`](config/buildroot-lock.toml)
(**2026.05.2** at time of writing). Each build publishes images, checksums,
legal-info, and `build-manifest.json` (including host Rust used inside Buildroot).

Local rebuilds on a lab VM: [docs/operations/LOCAL_BUILDROOT_VM.md](docs/operations/LOCAL_BUILDROOT_VM.md).

## What we claim (and do not)

| Claim | Status |
| --- | --- |
| Open-source IP↔MS/TP router **intent** and appliance architecture | Yes |
| Reproducible Buildroot images and management UI scaffold | M0 (in progress on `master`) |
| Field-ready routing, BTL certification, or Clause 9 conformance | **No** — gated milestones |
| QEMU / unit tests prove RS-485 on a live trunk | **No** — hardware jobs are manual and isolated |

Educational comparison to commercial BACnet routers (layout ideas only, original
UI): [docs/product/BASRT_EDUCATIONAL_REFERENCE.md](docs/product/BASRT_EDUCATIONAL_REFERENCE.md).

**Agent reading (required):**

- [AGENTS.md](AGENTS.md) — engineering contract
- [docs/agent/SOFTWARE_SPEC.md](docs/agent/SOFTWARE_SPEC.md) — product, UI, metrics, prototype lineage
- [docs/agent/FULL_STACK_AUDIT.md](docs/agent/FULL_STACK_AUDIT.md) — full-stack audit/refactor checklist (Rust, React, Buildroot, QEMU, SSH)
- [docs/agent/SPEC.md](docs/agent/SPEC.md) — milestones M0–M6

**Current M0 work:** if GitHub Actions may already be green, start with artifact
acceptance on the VirtualBox lab VM — do not assume Buildroot is broken:

- [docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md](docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md)
- [docs/operations/LOCAL_BUILDROOT_VM.md](docs/operations/LOCAL_BUILDROOT_VM.md)

**UI reference (education only):** [docs/product/BASRT_EDUCATIONAL_REFERENCE.md](docs/product/BASRT_EDUCATIONAL_REFERENCE.md)

Post-M0 routing integration: [docs/agent/CURSOR_CONTINUATION_PROMPT.md](docs/agent/CURSOR_CONTINUATION_PROMPT.md).

## License

MIT — see [LICENSE](LICENSE).