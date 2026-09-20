# Clause 9 baud matrix (FEC off, full allowed set) — 20260920T132700Z

FEC physically removed. Dual mini MAC2+MAC3 + bensbench router MAC1.
FTDI `latency_timer=1`. Oracle: bacpypes3 on bosspi `192.168.204.12`.
Router appliance: diy-bacnet-router tip pin **rusty-bacnet `9e5168c5`** (host binary via privileged Docker + `iproute2`).
Minis: Vibe13 `mstp-mini-device` (historical `af4e886` — fixtures only).

| Baud | Passive ok | valid_ratio | tokens | sources | Routed RP (4 reads) | Product claim |
|------|------------|-------------|---------|---------|---------------------|---------------|
| **38400** | PASS | ~0.999 | 2119 | [2,3] | **4/4 PASS** | **Default / supported** |
| **9600** | FAIL | ~0.568 | 0 | [2,3] | **0/4 FAIL** (timeout) | **Not claimed** |
| **19200** | PASS | ~0.999 | 1354 | [2,3] | **0/4 FAIL** (timeout) | **Not claimed** |
| **57600** | PASS | ~0.999 | 2816 | [2,3] | **4/4 PASS** | Lab-supported (FEC off, minis) |
| **76800** | PASS | ~0.999 | 3192 | [2,3] | **4/4 PASS** | Lab-supported (FEC off, minis) |
| **115200** | PASS | ~0.999 | 3484 | [2,3] | **4/4 PASS** | Lab-supported (FEC off, minis) |

## Product baud policy (locked)

- **Default live trunk: 38400.** FEC (when re-attached) stays **38400 only**, read-only.
- Allowed config set remains `{9600, 19200, 38400, 57600, 76800, 115200}` for fail-closed validation.
- **Claimed for this Waveshare C + FTDI/CH343 dual-mini lab:** 38400 (default), plus 57600 / 76800 / 115200 **without FEC**.
- **Not claimed:** 9600, 19200 (USB/timing — same pattern as prior Clause9 + rusty-bacnet #707).

## Explicit non-claims

- No FEC-on evidence at non-38400.
- No `ready_to_route` product flip.
- No BTL / full Clause 9 self-test.
- No claim that native UART (non-USB) would fail the same way at 9600/19200.

## Restore

Minis restored to **38400**, Max_Master=**7**. Router session stopped. Operator may re-attach FEC and restart `openfdd-fieldbus` for Open-FDD OT stress.

## Artifacts

- `summary.jsonl`, `passive-*.json`, `rp-*.json`, `runner.log`
- `rusty-bacnet-pin.txt` → `9e5168c5ac10bf06f3422f66fc2b0b9983a6ac5c`
- Runner: `scripts/lab_baud_matrix_fec_off.sh`
