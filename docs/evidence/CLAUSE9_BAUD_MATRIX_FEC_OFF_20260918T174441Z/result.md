# Clause 9 baud matrix (FEC off) — 20260918T174441Z

FEC physically removed. Dual mini MAC2+MAC3 + bensbench router MAC1. FTDI latency_timer=1 on listen host.

| Baud | Passive sources | Passive valid_ratio | Routed RP (mini2+mini3) |
|------|----------------|--------------------|-------------------------|
| 38400 (FEC off baseline) | [2,3] | ~0.999 | (prior 38400+FEC 6/6 PASS) |
| **9600** | [2,3] | **~0.76 FAIL gate** | **FAIL** Timeout / no-response |
| **19200** | [2,3] | ~0.998 PASS | **FAIL** AbortPDU no-response |
| **76800** | [2,3] | ~0.999 PASS | **PASS** both object-name reads |

## PICS-facing claim (honest)

- **Supported / evidenced for this lab hardware:** **38400** (with or without FEC), **76800** (minis only, FEC off).
- **Not claimed yet:** 9600, 19200 on Waveshare C + CH343 dual-mini trunk (hear OK at 19200; app RP not green).
- Never plant DNET 2000. No FEC writes.

## Operator note

Minis restored to **38400**. Safe to **re-attach FEC** when ready; restart bensbench router at 38400 after FEC is back.
