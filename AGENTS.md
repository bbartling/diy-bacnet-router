# AGENTS.md — DIY BACnet Router engineering contract

These rules apply to the entire repository. Read this file, `README.md`,
`docs/agent/SPEC.md`, `docs/ARCHITECTURE.md`, `docs/TESTING.md`, and
`docs/UPSTREAM_LOCK.md` before changing code.

## Mission

Build a trustworthy, original Linux BACnet/IP-to-MS/TP router appliance. The
deliverable is a reproducible OS image plus a router data plane and a small
management plane. This is not a BACnet application-device project.

## Non-negotiable boundaries

- The router forwards NPDUs between distinct BACnet networks. It must not reuse
  the Vibe13 mini-device AI/BI/AV/BV database as its data plane.
- The dashboard, REST API and WebSocket are management surfaces only. They must
  not block token handling or packet forwarding.
- Do not copy commercial branding, firmware, HTML, images or trade dress.
- Do not claim Clause 9 conformance, BTL certification, segmentation, extended
  frames, BBMD, FDR, routing, or a tested baud unless the named gate has current
  evidence.
- Prefer the pinned upstream rusty-bacnet APIs. Do not copy stack internals into
  this repository to make an API mismatch disappear.
- The default configuration is fail-closed: forwarding disabled, management
  bound to loopback, no default password, no write API.

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

## Agent workflow

1. Inspect the working tree and preserve user changes.
2. Identify one gate from `docs/agent/SPEC.md`.
3. Add a failing test or executable acceptance check.
4. Make the smallest implementation that passes it.
5. Run the required checks.
6. Update the evidence ledger and upstream lock if relevant.
7. Stop at hardware, signing, network mutation or release approval boundaries.

