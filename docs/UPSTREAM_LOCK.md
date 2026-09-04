# Upstream dependency lock

## Current M1 pin (audited 2026-09-04)

| Field | Value |
| --- | --- |
| Repository | https://github.com/jscott3201/rusty-bacnet |
| Branch audited | `dev` |
| Full SHA | `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` |
| Status | `m1-audited-pin` |
| MSRV | Rust 1.93 |
| Consumed crates | `bacnet-types`, `bacnet-encoding`, `bacnet-transport`, `bacnet-network` via `crates/rusty-bacnet-adapter` |

### Audit evidence at this SHA

- Public APIs reused (not forked): `BACnetRouter`, `RouterPort<T>`, `AnyTransport<S>`, `LoopbackTransport`.
- `cargo test -p bacnet-network --locked` at the pin: **73 passed** (Windows host, 2026-09-04).
- MS/TP codec: standard frames capped at **501 data octets**; extended COBS frames are not in this pin.
- Segmentation remains an application-layer capability, not something this adapter reinterprets.
- Upstream issues **#498–#502** are **open issues** (MS/TP Linux timing / qualification), not merged PRs. They do not block a fail-closed compile + loopback fixture.

### Explicit non-claims

- Vibe13 hardware results at historical `af4e88680c51eb4da64dac47f0540a35bf184732` **do not transfer** to this SHA.
- Pinning this SHA does **not** enable appliance forwarding. `routerd` stays fail-closed (`data_plane=disabled`, `ready_to_route=false`) until later isolated gates.
- No BBMD, Ethernet, NAT, BeagleBone, extended frames, or mini-device AI/BI object database is pulled into the M1 adapter.

## Historical Vibe13 note

The Vibe13 closeout reportedly used short revision `af4e886` and proved a standard-frame MS/TP mini-device at 38,400 baud. That remains historical evidence only.

## Rust toolchain boundary

Project CI and `rust-toolchain.toml` select Rust **1.93.0**. Upstream's own `rust-toolchain.toml` at the pin may advertise a newer channel for their CI; our MSRV gate is **1.93**. Buildroot host Rust is independent (see `config/buildroot-lock.toml`).

## Patch placement

Changes to MS/TP framing, timers, serial drain behavior, transport health or
generic network routing should be proposed upstream with focused tests. Device
configuration, Linux supervision, management metrics and UI belong here.
