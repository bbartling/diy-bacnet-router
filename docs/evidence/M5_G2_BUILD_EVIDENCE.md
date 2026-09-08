# G2 — Raspberry Pi images build in Actions (flash/boot OPEN)

| Field | Value |
| --- | --- |
| Gate | G2 |
| Claim | **Images build** in GitHub Actions and publish manifests/artifacts |
| Non-claim | Physical Pi flash, boot, soak, or M2B on Buildroot images |

## Recorded Actions run IDs (successful matrix)

| Tip SHA | Workflow run | Targets observed |
| --- | --- | --- |
| `baae2367dd3bf88eca03bb149c5d81d9091a7a8d` | [34071585237](https://github.com/bbartling/diy-bacnet-router/actions/runs/34071585237) | `x86_64`, `rpi3_64`, `rpi4_64`, `rpi5_64` (all success) |

Job IDs at that run (build ≠ flash validation):

| Job | Job ID |
| --- | --- |
| x86_64 image | 101589829331 |
| rpi3_64 image | 101589829357 |
| rpi4_64 image | 101589829315 |
| rpi5_64 image | 101589829323 |

## Tip hygiene after M3–M6 software merges

Later tip merges (#58–#61) may cancel or queue `build-os` when a newer tip supersedes.
Record a **new** successful tip `build-os` run ID here when the next full Pi matrix completes on current `master` — until then G2 evidence cites the last full green matrix above.

## Explicit status

- G2 ledger: **images build in Actions** (documented).
- Physical Raspberry Pi boot / soak: **OPEN**.
- No image binaries committed to git.
