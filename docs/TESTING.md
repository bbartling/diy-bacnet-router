---
title: Testing and evidence
layout: default
nav_order: 7
permalink: /testing/
---

# Test and evidence strategy

## Labels

- **unit**: no OS device or network
- **integration**: loopback/in-memory/PTY/veth; deterministic and CI-safe
- **qemu**: booted x86 appliance image, no claim about physical serial
- **hardware**: opt-in real adapter and isolated BACnet bench
- **soak**: bounded long-running hardware qualification with artifact output

## Gate ledger

| Gate | Purpose | Initial status |
|---|---|---|
| G0 | Config validation, API contract, bounded metrics | Scaffold implemented |
| G1 | Buildroot x86 image boots in QEMU and `/healthz` responds | Workflow prepared |
| G2 | Raspberry Pi 3/4/5 images build and publish manifests | Workflow prepared |
| G3 | Current rusty-bacnet pin and adapter compile/tests | Open |
| G4 | Passive Waveshare C decode: valid frames/tokens, no TX | Open in new repo |
| G5 | MS/TP master joins isolated ring without disrupting peer | Open |
| G6 | B/IP port local unicast/broadcast behavior | Open |
| G7 | Routed unicast ReadProperty in both directions | Open |
| G8 | Routed Who-Is/I-Am and router network messages | Open |
| G9 | Fault/restart: USB unplug, NIC loss, duplicate MAC/network | Open |
| G10 | One-hour then 24-hour forwarding soak | Open |
| G11 | Claimed baud/board matrix under load | Open |

Vibe13 artifacts are prior evidence and test-vector inputs, not automatic passes
for G4–G11 in a different process and OS image.

## Milestone 0 non-hardware commands

The non-hardware gate is reproducible with the same locked inputs used by CI:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
npm ci --ignore-scripts
npm audit --audit-level=high
npm run check
npm run build
cargo metadata --locked --no-deps --format-version 1
cargo audit
cargo deny check advisories
bash scripts/validate-workflows.sh
```

The Buildroot workflow records the host Rust compiler independently from the
project's `rust-toolchain.toml`: `buildroot-host-rustc-version.txt` and the
same value in `build-manifest.json`. This is evidence of the Buildroot host
toolchain, not a change to the project's Rust toolchain pin. The image
artifact also contains `SHA256SUMS`, the manifest and the Buildroot
`legal-info.tar.xz` archive.

## Hardware job ordering

1. Inventory kernel, PREEMPT settings, CPU governor, USB topology and adapter.
2. Verify physical topology, reference, bias and exactly two terminations.
3. Prove one process owns the tty.
4. Passive capture and valid CRC/token thresholds.
5. Confirm candidate MAC is unused.
6. Join with conservative `Max_Info_Frames=1`.
7. Run one request and confirm existing supervisory health.
8. Run routing tests on an isolated topology.
9. Only then run a soak.

Every artifact records project SHA, full rusty-bacnet SHA, image manifest,
kernel, architecture, adapter IDs/serials, by-id path, baud, FTDI latency,
termination/reference notes, counters, start/end timestamps and exit reason.

## Hardware CI security

The `hardware-mstp.yml` workflow is manual, serialized and restricted to a
protected environment. Never execute fork or arbitrary PR code on a bench
runner. The runner must have no broad SSH or signing credentials.

## Performance budgets

Initial budgets to validate rather than assume:

- no unbounded allocation per packet;
- metrics scrape/snapshot p99 below 25 ms on Raspberry Pi 3 under normal load;
- management CPU below 5% average at one-second updates on Pi 3;
- no token timing regression when 10 dashboard clients connect/disconnect;
- bounded WebSocket connection count and send failure cleanup;
- routing soak reports zero silent truncations and zero duplicate forwarding.
