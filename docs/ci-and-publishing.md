# CI and publishing

## GitHub Actions

| Workflow | What it runs |
|----------|----------------|
| `.github/workflows/ci.yml` | **Rust 1.96** on `ubuntu-latest`: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` (29 unit tests). Clones `rusty-bacnet` and `rusty-haystack` as sibling repos for path dependencies. |

## Local

```bash
cd rust-api
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
OPENFDD_FIELDBUS_CONFIG_DIR=../config cargo test
```

## Live bench (not CI)

```bash
OPENFDD_FIELDBUS_API_KEY=... scripts/smoke_test.sh
OPENFDD_FIELDBUS_API_KEY=... scripts/bench_test.sh
```

Requires free UDP `:47808`, bench BACnet devices, and optional Modbus/Haystack targets.
