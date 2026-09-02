# AGENTS.md — DIY BACnet Router engineering contract

These rules apply to the entire repository. Read this file, [README.md](README.md),
[docs/agent/SOFTWARE_SPEC.md](docs/agent/SOFTWARE_SPEC.md),
[docs/agent/FULL_STACK_AUDIT.md](docs/agent/FULL_STACK_AUDIT.md),
[docs/agent/SPEC.md](docs/agent/SPEC.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md),
[docs/TESTING.md](docs/TESTING.md), and [docs/UPSTREAM_LOCK.md](docs/UPSTREAM_LOCK.md)
before changing code.

## Mission

Build a trustworthy, **original** Linux BACnet/IP-to-MS/TP router appliance for
**education and lab use** — functionally in the class of a Contemporary Controls
BASRT-B, but implemented as open Rust + Buildroot with honest evidence gates.

Deliverables:

- reproducible OS images (x86_64, Raspberry Pi 3/4/5);
- Rust data plane (`routerd`) for NPDU forwarding between distinct BACnet networks;
- management plane: Axum REST, OpenAPI, bounded WebSocket metrics, React dashboard;
- SSH-managed Linux networking and TOML application config.

This is **not** a BACnet application-device project (no AI/BI/AV/BV object database
as the router data plane).

## Prototype lineage

Phase 1–2 serial and MS/TP evidence lives in the external prototype:

```text
py-bacnet-stacks-playground/vibe_code_apps_13
```

Reuse: Waveshare C wiring runbooks, passive-decode gates, supervisory metrics
ideas, rusty-bacnet integration lessons.

**Do not import** the Vibe13 mini-device object database into this router.

## Reference hardware

Primary MS/TP adapter: [Waveshare USB TO RS485 (C)](https://www.waveshare.com/usb-to-rs485-c.htm)
(FT232RNL, isolated RS-485, automatic direction, onboard 120 Ω termination).

Read [docs/hardware/WAVESHARE_USB_RS485_C.md](docs/hardware/WAVESHARE_USB_RS485_C.md)
before bench or trunk work.

## Non-negotiable boundaries

- The router forwards NPDUs between distinct BACnet networks. It must not reuse
  the Vibe13 mini-device AI/BI/AV/BV database as its data plane.
- The dashboard, REST API and WebSocket are **management surfaces only**. They must
  not block token handling or packet forwarding.
- Do not copy commercial branding, firmware, HTML, images or trade dress (including
  BASRT-B web pages). Functional comparison for education is fine in private docs.
- Do not claim Clause 9 conformance, BTL certification, segmentation, extended
  frames, BBMD, FDR, routing, or a tested baud unless the named gate has current
  evidence.
- Prefer the pinned upstream rusty-bacnet APIs. Do not copy stack internals into
  this repository to make an API mismatch disappear.
- The default configuration is fail-closed: forwarding disabled, management
  bound to loopback, no default password, no write API.

## Management UI and metrics

The React dashboard is a **LAN-only operator console** behind the firewall — not
a public internet application. There is **no URL versioning** on management
routes (`/api/status`, not `/api/v1/status`); the appliance ships as one
cohesive firmware + UI unit.

- WebSocket **`/api/ws/metrics`** delivers **aggregate snapshots** (default 1000 ms,
  bounded 250–5000 ms) including MS/TP trunk health (token/PFM, FSM state, CRC
  errors). Never one message per BACnet frame.
- The browser **must always display** the release from root [`VERSION`](VERSION)
  (compiled into `routerd` as `DBR_VERSION`); the sidebar and header show it by default.
- Counter names in the metrics schema are stable API contracts (B/IP and MS/TP
  packet counts, token/PFM counters, RFSM/MNSM state, CRC errors, system stats).
- Browser configuration writes remain disabled until M6 auth/audit gates pass.
  Host IP and routes are configured over **SSH** with normal Linux tools.
- UI layout may follow industrial router **patterns** (grouped config sections,
  status counters); styling must be original DBR branding.

See [docs/agent/SOFTWARE_SPEC.md](docs/agent/SOFTWARE_SPEC.md) and
[docs/product/BASRT_EDUCATIONAL_REFERENCE.md](docs/product/BASRT_EDUCATIONAL_REFERENCE.md).

## Dependency policy

- Never depend on a moving `dev` branch in a committed `Cargo.toml`.
- Audit current upstream `dev`, run its relevant tests, then record a full
  40-character commit in `config/upstream-lock.toml` and `docs/UPSTREAM_LOCK.md`.
- Commit `Cargo.lock`. Use `--locked` in CI and Buildroot.
- Upstream stack changes belong in focused rusty-bacnet PRs with failing tests
  first. Application policy and appliance integration stay here.

## BACnet and serial safety

- Use `/dev/serial/by-id/...`, never persist `ttyUSB0`.
- Allow only 9600, 19200, 38400, 57600, 76800 and 115200 baud. Default 38400.
- Configure 8N1, no flow control. Waveshare automatic direction means no
  simultaneous Linux RS-485 ioctl, RTS or GPIO direction control.
- Exactly one process owns a tty. Never kill an unknown owner automatically.
- Validate that B/IP and MS/TP network numbers are distinct and in 1..=65534.
- Validate MS/TP MAC <= Max_Master <= 127 and Max_Info_Frames in 1..=255.
- Passive decode must pass before any hardware job transmits. Stop on duplicate
  MAC, loss of the existing trunk, token storm or rising CRC/timeouts.
- Never run hardware tests from an untrusted pull request.

## Metrics

- Data-plane counters use atomics or a bounded nonblocking channel.
- Browser updates are aggregate snapshots; no per-packet WebSocket messages.
- The WebSocket interval is bounded to 250..=5000 ms and defaults to 1000 ms.
- All queues, histories, captures and support bundles are bounded.
- Counter names and units are stable API contracts and require tests.

## Required tests before handoff

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
npm --prefix frontend/web ci
npm --prefix frontend/web run check
npm --prefix frontend/web run build
```

Also run `scripts/validate-repository.sh`. Hardware and Buildroot tests must be
reported separately and truthfully; lack of hardware is not a failure and is
never relabeled as a pass.

For repository-wide audits or refactors, follow
[docs/agent/FULL_STACK_AUDIT.md](docs/agent/FULL_STACK_AUDIT.md) and include its
completion report. Key stack facts: **rusty-bacnet (Rust) for BACnet** — no Python
in the data plane; **QEMU `-snapshot` smoke** for x86_64 images; **Buildroot**
for appliance images; **SSH-managed Linux networking** for host IP/routes.

## Agent workflow

1. Inspect the working tree and preserve user changes.
2. For M0 image pipeline work, start with
   [docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md](docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md)
   — verify Actions artifacts before editing Buildroot.
3. Identify one gate from [docs/agent/SPEC.md](docs/agent/SPEC.md).
4. Add a failing test or executable acceptance check.
5. Make the smallest implementation that passes it.
6. Run the required checks.
7. Update the evidence ledger and upstream lock if relevant.
8. Stop at hardware, signing, network mutation or release approval boundaries.

## Buildroot and local lab

- **Buildroot:** pin the **latest stable bugfix** in
  [`config/buildroot-lock.toml`](config/buildroot-lock.toml) (currently **2026.05.2**).
  Bump only after CI **and** lab VM x86 QEMU smoke pass. See
  [docs/UPSTREAM_LOCK.md](docs/UPSTREAM_LOCK.md).
- **Local builds:** Ubuntu guest in **VMware**, SSH from Windows host to
  `127.0.0.1:2222` — **not WSL**. Full topology in
  [docs/operations/LOCAL_BUILDROOT_VM.md](docs/operations/LOCAL_BUILDROOT_VM.md)
  and [docs/agent/SOFTWARE_SPEC.md](docs/agent/SOFTWARE_SPEC.md).
- When `build-os` fails: reproduce with `scripts/vm-debug-build.sh` on the guest,
  fix on a branch, push, confirm green Actions before claiming PASS.

## Cursor skills (project)

- [.cursor/skills/local-buildroot-vm/SKILL.md](.cursor/skills/local-buildroot-vm/SKILL.md) —
  VMware Ubuntu lab, artifact acceptance, Buildroot debug loop.
- [.cursor/skills/basrt-educational-router/SKILL.md](.cursor/skills/basrt-educational-router/SKILL.md) —
  product intent, UI/metrics contract, Vibe13 and Waveshare context.
