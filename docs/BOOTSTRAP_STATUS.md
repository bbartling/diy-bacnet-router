> **Historical snapshot** from scaffold generation. Live CI status and
> [M0_ARTIFACT_ACCEPTANCE_PROMPT.md](agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md)
> are authoritative for current M0 work.

# Bootstrap status

This file is the evidence boundary for the generated Milestone 0 scaffold. It
prevents a successful syntax check from being reported as a working BACnet
router or a bootable appliance image.

## Checks completed on the generation host

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo metadata --no-deps --locked --format-version 1` | PASS |
| TypeScript source parse/transpile check | PASS |
| OpenAPI and npm lock JSON parse | PASS |
| GitHub workflow YAML parse | PASS |
| `bash -n` for every shell script | PASS |
| PowerShell AST parse | PASS |
| `Install-DiyBacnetRouter.ps1 -WhatIf` | PASS; no target mutation |
| `scripts/validate-repository.sh` | PASS |

## Not completed on the generation host

The sandbox had no package-registry access and did not have the required Cargo
or npm packages cached. It also had no QEMU or BACnet hardware. Consequently,
these are **OPEN**, not failed and not passed:

- full Cargo compile, Clippy and Rust unit/integration tests;
- `npm ci`, TypeScript typecheck and Vite production build;
- Buildroot compilation, legal-info and x86 QEMU boot;
- Raspberry Pi image builds and physical boot;
- rusty-bacnet source audit or dependency integration;
- MS/TP, BACnet/IP and routed hardware acceptance.

The first networked GitHub Actions run is the authoritative bootstrap compile.
Correct the repository if it fails; do not weaken or skip the gate.

## Seed lockfiles

`Cargo.lock` and `frontend/web/package-lock.json` were adapted from local clean
repository lockfiles so `--locked` metadata and static checks could run without
network access. Their root manifests match this scaffold, but they may retain
unused historical package records. A networked agent must regenerate both from
the committed manifests, review the resulting dependency graph and commit the
clean lockfiles before calling Milestone 0 green.

## Allowed claim

> The repository contains a statically validated management-plane and OS-build
> scaffold. BACnet routing, complete dependency builds and image boot are not
> yet verified.
