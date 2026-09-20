# Tip repin rusty-bacnet → 9e5168c5

| Field | Value |
| --- | --- |
| Previous pin | `acbf7baefe69d05f2368763dcc659d68e4bc114c` |
| New pin | `9e5168c5ac10bf06f3422f66fc2b0b9983a6ac5c` |
| Includes | #715 `MstpTransport::diagnostics()` (`7aab278`) through tip `9e5168c5` |
| Product baud | **hold 38400** (9600/19200 not claimed) |
| Stamp UTC | 20260920T130422Z |

## Software gates

- `bash scripts/test-upstream-pin.sh` → PASS
- `bash scripts/validate-repository.sh` → PASS
- `cargo test -p rusty-bacnet-adapter --lib` → 41 passed (includes host_diagnostics report)
- `cargo test --workspace --exclude diy-bacnet-router-ui` → PASS

## Adapter tip consumption

MS/TP qualify reports now mirror `host_diagnostics` from rusty-bacnet #715. Gap note updated: host counts ≠ wire CRC aggregates.

## Hardware / 38400 FEC

- Local FTDI inventory only (`hardware-preflight.sh`, `hardware_evidence: false`)
- Pi lab hostnames not resolvable from this workstation; **38400 FEC via-DIY RP re-prove deferred** — do not claim trunk PASS at this tip until lab oracle runs

## Explicit non-claims

- No 9600/19200 product claim
- No extended-frame PASS
- No `ready_to_route` product flip
