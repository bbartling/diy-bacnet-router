# Upstream dependency lock

## Current state

The Vibe13 closeout reportedly used `jscott3201/rusty-bacnet` dev at short
revision `af4e886` and proved a standard-frame MS/TP mini-device at 38,400 baud.
That is historical evidence, not a valid production dependency lock.

Before adding rusty-bacnet dependencies here, an agent must:

1. fetch current `jscott3201/rusty-bacnet` `dev`;
2. record its full 40-character SHA and date;
3. verify merged MS/TP CRC/token work and inspect later changes;
4. run formatting, `bacnet-transport` serial tests, `bacnet-network` tests and
   applicable integration tests at that SHA;
5. inspect actual public APIs for heterogeneous ports and routing;
6. pin every consumed crate to that exact SHA;
7. commit `Cargo.lock` and a compile fixture;
8. update `config/upstream-lock.toml` and this document.

The upstream project describes `bacnet-transport` as containing B/IP and MS/TP
and `bacnet-network` as containing network-layer routing. That is a promising
base, not proof that the appliance's exact B/IP↔MS/TP lifecycle exists.

## Rust toolchain boundary

At scaffold time, current upstream documentation advertised Rust 1.93 as its
minimum supported toolchain. The ordinary project CI and `rust-toolchain.toml`
therefore select Rust 1.93.0. Buildroot does **not** automatically honor that
rustup file: its `cargo-package` infrastructure uses the host Rust toolchain
selected by the Buildroot configuration.

## M0 Buildroot compatibility lock

Buildroot `2025.02.17` supplies host Rust/Cargo `1.82.0`. The project keeps its
own declared MSRV and CI toolchain at Rust `1.93`; this is not a project-MSRV
downgrade. The network-dependent `getrandom` target graph selected
`wasip2 1.0.4+wasi-0.2.12` and `wit-bindgen 0.57.1` in the seeded lockfile, but
`wit-bindgen 0.57.1` uses Edition 2024 syntax that Cargo `1.82.0` cannot parse.
The reviewed M0 `Cargo.lock` therefore pins the compatible transitive pair
`wasip2 1.0.1+wasi-0.2.4` and `wit-bindgen 0.46.0`. This keeps Buildroot's
official offline cargo-vendor path usable without changing application source,
silently installing another compiler, or integrating rusty-bacnet.

The current `jscott3201/rusty-bacnet` `dev` tip observed on 2026-09-02 is
`65ae4633ea26e24f959991cb4a2ee2d9d982bc98` and advertises MSRV Rust `1.93`.
It remains unaudited and is not a dependency in this M0 build. The historical
Vibe13 evidence pin remains `af4e886`.

Before adding any rusty-bacnet crate to the image, the x86 image job must print
the Buildroot host `rustc --version` and prove it satisfies the exact audited
upstream MSRV. If Buildroot 2025.02.x cannot provide that version, solve it as
a reproducible Buildroot toolchain/package change or move to a reviewed newer
Buildroot LTS. Never bypass an MSRV check with an untracked host-installed
compiler.

## Patch placement

Changes to MS/TP framing, timers, serial drain behavior, transport health or
generic network routing should be proposed upstream with focused tests. Device
configuration, Linux supervision, management metrics and UI belong here.

Open candidates from the Vibe13 work that must be re-audited rather than blindly
ported:

- physical TX completion / timer anchoring after serial drain;
- Rust `MstpConfig` fail-fast validation;
- transport health/disconnect notification;
- one shared MS/TP endpoint usable by the required application roles.
