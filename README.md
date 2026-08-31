# DIY BACnet Router

An original, Linux-based BACnet/IP-to-MS/TP router appliance project for
Raspberry Pi 3/4/5 and x86-64. The repository combines a Rust data plane, a
small Rust management API, a React dashboard, reproducible Buildroot images,
and evidence-gated hardware testing.

> Current status: **Milestone 0 scaffold, not a working BACnet router.** The
> management API, bounded metrics stream, configuration validation, frontend,
> CI and image-build framework live here. BACnet NPDU forwarding stays disabled
> until the rusty-bacnet adapter and isolated routing gates pass.

## Why this repository exists

`vibe_code_apps_13` proved the critical Phase 1 serial path and Phase 2
standard-frame MS/TP device behavior. This repository is the clean appliance
boundary. It deliberately does not import the Phase 2 mini-device object
database into the router.

## Architecture

```text
BACnet/IP UDP                 Rust router core                  USB RS-485
 Ethernet NIC   <---->   NPDU forwarding + policy   <---->   MS/TP master
                                |
                         bounded atomics/events
                                |
                    REST + OpenAPI + WebSocket
                                |
                         React dashboard
```

The management plane must never sit in the packet-forwarding hot path. Packet
counters are atomic, snapshots are bounded, and the browser receives aggregate
statistics rather than one event per BACnet frame.

## Run the scaffold

```bash
cp config/router.example.toml config/router.toml
cargo run -p routerd -- --config config/router.toml
```

Then open <http://127.0.0.1:8080>. Useful endpoints:

- `GET /healthz`
- `GET /api/v1/status`
- `GET /api/v1/capabilities`
- `GET /api/v1/metrics/snapshot`
- `GET /api/v1/openapi.json`
- `GET /api/v1/ws/metrics`
- `GET /metrics`

Build the React application:

```bash
cd frontend/web
npm ci
npm run check
npm run build
```

## Configuration

The normal appliance configuration is `/etc/diy-bacnet-router/router.toml`.
Use a stable `/dev/serial/by-id/...` path. The supported rates are 9600,
19200, 38400, 57600, 76800 and 115200 baud; 38400 is the default.

The initial reference adapter is the isolated
[Waveshare USB TO RS485 (C)](https://docs.waveshare.com/USB_TO_RS485_C), using
its FT232RNL and hardware-automatic direction control. It has an onboard 120 Ω
resistor, so it counts as a terminated endpoint in the topology. Read
[docs/hardware/WAVESHARE_USB_RS485_C.md](docs/hardware/WAVESHARE_USB_RS485_C.md)
before attaching it to an active trunk.

Linux interface addresses, routes, DNS and hostnames remain ordinary Linux
network configuration. They are not hidden inside BACnet-specific environment
variables. Environment variables are reserved for deployment overrides such as
`DBR_CONFIG`, `DBR_BIND`, `DBR_WEB_ROOT` and `RUST_LOG`.

## Build appliance images

GitHub Actions calls `scripts/build-image.sh` with one of:

- `x86_64`
- `rpi3_64`
- `rpi4_64`
- `rpi5_64`

The image pipeline is based on a Buildroot `br2-external` tree. Every build
publishes the images, checksums, legal information and a version manifest.
The x86-64 image is also eligible for a QEMU boot smoke test.

## Evidence boundaries

- Compilation, unit tests and QEMU do not prove RS-485 behavior.
- Hardware jobs are manual and use an isolated self-hosted runner.
- Extended MS/TP frames, segmentation and production conformance are not
  claimed.
- No BTL certification or formal PICS claim is made.
- The router remains fail-closed until both BACnet ports are healthy and their
  network numbers are valid and distinct.

Read [AGENTS.md](AGENTS.md) and [docs/agent/SPEC.md](docs/agent/SPEC.md) before
asking an AI coding agent to continue implementation.

For a focused first run whose only job is to make the non-hardware scaffold and
OS image matrix honestly green, use
[docs/agent/LUNA_MAX_GITHUB_ACTIONS_PROMPT.md](docs/agent/LUNA_MAX_GITHUB_ACTIONS_PROMPT.md).

For the exact checks completed when this scaffold was generated, and the
network-dependent checks that remain for the first GitHub Actions run, read
[docs/BOOTSTRAP_STATUS.md](docs/BOOTSTRAP_STATUS.md).
