# MS/TP health — dual mini + FEC @ 38400 (20260918T173211Z)

| Item | Value |
|------|-------|
| Baud | 38400 |
| FEC | ON trunk |
| Passive sources_seen | see passive.json (expect 2,3,7) |
| Qualify | see qualify-mac1.json |
| Routed RP | 6/6 PASS — mini2, mini3, FEC (controlled-rp-fec.json) |

Router: bensbench MAC1 DNET 2001. Minis: pi2 MAC2/123102, pi1 MAC3/123103.
BIP oracle: pi1 bacpypes3 → 192.168.204.11. Never DNET 2000. No FEC writes.
