# Upstream dependency lock

## Current pin (repinned 2026-09-18)

| Field | Value |
| --- | --- |
| Repository | https://github.com/jscott3201/rusty-bacnet |
| Branch audited | `dev` |
| Full SHA | `acbf7baefe69d05f2368763dcc659d68e4bc114c` |
| Status | `phase1-tip-repin` (was `7e0d13a` tip; prior M1 `24e3439`) |
| MSRV | Rust 1.93 |
| Consumed crates | `bacnet-types`, `bacnet-encoding`, `bacnet-transport`, `bacnet-network` **v0.11.0** via `crates/rusty-bacnet-adapter` |

**Agents:** rusty-bacnet (especially MS/TP) changes frequently. Follow the **Daily rusty-bacnet / MS/TP watch** in [AGENTS.md](../AGENTS.md) every session — compare tip to this pin before assuming lab timing is unchanged. Never float the pin without a lock PR.

### Repin gate (run every Cargo.toml `rev` bump)

When advancing `bacnet-*` git `rev` values:

1. Update **all** of: workspace [`Cargo.toml`](../Cargo.toml), [`config/upstream-lock.toml`](../config/upstream-lock.toml), [`crates/rusty-bacnet-adapter/src/lib.rs`](../crates/rusty-bacnet-adapter/src/lib.rs) (`UPSTREAM_REVISION` / `_SHORT`), this doc, then `cargo update -p bacnet-network -p bacnet-transport -p bacnet-encoding -p bacnet-types`.
2. Run:

```bash
bash scripts/test-upstream-pin.sh
cargo test -p rusty-bacnet-adapter --locked
cargo test --workspace --locked
bash scripts/validate-repository.sh
```

CI already runs `cargo test --workspace` and `bash scripts/validate-repository.sh` (which calls `test-upstream-pin.sh`). Hard-coded SHAs in image verify scripts must not be reintroduced — they read the lock file dynamically.

### Audit evidence at this SHA

- Public APIs reused (not forked): `BACnetRouter`, `RouterPort<T>`, `AnyTransport<S>`,
  `BipTransport`, `MstpTransport`, `TokioSerialPort`, `LoopbackTransport`.
- Adapter closeout: concrete B/IP + MS/TP factories compile and validate **without**
  calling `start()` / `TokioSerialPort::open` on ordinary unit/CI paths.
- `cargo test -p bacnet-network --locked` at the pin: **73 passed** (Windows host, 2026-09-04).
- MS/TP codec: standard frames capped at **501 data octets**; extended COBS frames: not claimed PASS at this tip without dedicated evidence (verify codec before phase-3 claims).
- Segmentation remains an application-layer capability, not something this adapter reinterprets.
- Upstream issues **#498–#502** are **open issues** (MS/TP Linux timing / qualification), not merged PRs.
  They do not block fail-closed compile fixtures.

### Explicit non-claims

- Vibe13 hardware results at historical `af4e88680c51eb4da64dac47f0540a35bf184732` **do not transfer** to this SHA.
- Pinning this SHA does **not** enable appliance forwarding. `routerd` stays fail-closed (`data_plane=disabled`, `ready_to_route=false`) until later isolated gates.
- Ordinary `routerd` startup does **not** construct or start B/IP/MS/TP sessions.
- No BBMD, Ethernet, NAT, BeagleBone, extended frames, or mini-device AI/BI object database is pulled into the M1 adapter.

## Historical Vibe13 note

The Vibe13 closeout reportedly used short revision `af4e886` and proved a standard-frame MS/TP mini-device at 38,400 baud. That remains historical evidence only.

## Rust toolchain boundary

Project CI and `rust-toolchain.toml` select Rust **1.93.0**. Upstream's own `rust-toolchain.toml` at the pin may advertise a newer channel for their CI; our MSRV gate is **1.93**. Buildroot host Rust is independent (see `config/buildroot-lock.toml`).

## Patch placement

Changes to MS/TP framing, timers, serial drain behavior, transport health or
generic network routing should be proposed upstream with focused tests. Device
configuration, Linux supervision, management metrics and UI belong here.
