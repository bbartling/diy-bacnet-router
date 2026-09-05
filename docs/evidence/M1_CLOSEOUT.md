# M1 closeout evidence

| Field | Value |
| --- | --- |
| Project branch | `feat/m1b-concrete-port-fixtures` |
| Recorded at | 2026-09-05 (local Windows verification) |
| Full rusty-bacnet SHA | `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` |
| Dirty/clean | See final merge SHA on `master` |
| Local VM Buildroot | **BLOCKED** — SSH `127.0.0.1:2222` connection refused (guest powered off) |

## APIs compiled (no UDP/tty opened in unit tests)

- `BipTransport::new` via `build_bip_transport` (no `start`)
- `MstpTransport::new` via `build_mstp_transport` with injected `SerialPort` (no `TokioSerialPort::open`)
- `AnyTransport<TokioSerialPort>` / `ApplianceRouterPort` type aliases
- Heterogeneous B/IP + MS/TP `RouterPort` construction with test `SerialPort`
- Existing loopback `BACnetRouter` start/stop
- Contract: ordinary `routerd` sources do not call transport factories / `TokioSerialPort::open`

## Commands (local)

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
npm --prefix frontend/web ci && npm --prefix frontend/web run check && npm --prefix frontend/web run build
bash scripts/validate-repository.sh
bash scripts/validate-workflows.sh
```

Exit codes: all **0** on Windows host 2026-09-05. Post-merge CI/`build-os` run IDs filled after merge.

## Explicit non-claims

- No BACnet UDP socket opened by ordinary `routerd` startup
- No tty opened in unit/CI adapter tests
- No NPDU forwarding; `ready_to_route=false`
- No physical MS/TP proof at this pin
- No extended frames, BBMD/FDR, or Clause 9 claim

## Post-merge Actions (fill on merge)

- CI run ID:
- build-os run ID:
- Merge SHA:
