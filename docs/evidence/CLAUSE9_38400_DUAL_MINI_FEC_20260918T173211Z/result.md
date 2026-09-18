# Clause 9 lab — 38400 dual-mini + FEC — 20260918T173211Z

## Topology
bensbench router MAC1 + pi2 mini MAC2 + pi1 mini MAC3 + FEC MAC7 @ 38400.

## Results
| Gate | Status |
|------|--------|
| Passive sources [2,3,7] | PASS |
| MAC1 qualify | see qualify-mac1.json |
| Routed RP mini2+mini3+FEC | **6/6 PASS** |

## Next
Human disconnect FEC before non-38400 baud matrix (9600/19200/76800).
