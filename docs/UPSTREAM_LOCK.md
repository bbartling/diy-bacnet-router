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

Buildroot version and commit are pinned in `config/buildroot-lock.toml`.
Current pin: **2026.05.2** (`72d9d4fa636a371ef9eb99c92a735ce9f6d829d5`), host
Rust **1.96.1** (observed from `package/rust/rust.mk` on 2026-09-02).

The project keeps its declared MSRV and CI toolchain at Rust **1.93** in
`rust-toolchain.toml`; Buildroot supplies an independent host compiler for the
offline `cargo-package` path inside the image build.

Previous M0 pin **2025.02.17** shipped host Rust **1.82.0**, which required
lockfile workarounds for `wasip2` / `wit-bindgen` Edition 2024 syntax. The
2026.05.2 host Rust satisfies the project MSRV and the future rusty-bacnet
MSRV (`1.93`) without a side-channel compiler.

The current `jscott3201/rusty-bacnet` `dev` tip observed on 2026-09-02 is
`65ae4633ea26e24f959991cb4a2ee2d9d982bc98` and advertises MSRV Rust `1.93`.
It remains unaudited and is not a dependency in this M0 build. The historical
Vibe13 evidence pin remains `af4e886`.

Before adding any rusty-bacnet crate to the image, the x86 image job must print
the Buildroot host `rustc --version` and prove it satisfies the exact audited
upstream MSRV. Never bypass an MSRV check with an untracked host-installed
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
