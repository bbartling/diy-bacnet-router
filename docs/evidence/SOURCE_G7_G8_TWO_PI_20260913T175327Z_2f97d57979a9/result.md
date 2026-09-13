# Source G7/G8 two-Pi result

- **UTC pack:** 20260913T175327Z
- **Router SHA:** `2f97d57979a9cd4b255663786e3929f579ccbba7` (tip matched local FETCH at run start)
- **rusty-bacnet pin:** `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad`
- **Vibe13 fixture:** `12e7d232b23023e1bb06da7b75972d79ca584cf9`
- **Topology:** B/IP net 1 @ 192.168.204.59 ↔ MS/TP net **2001** MAC1 ↔ MAC2 device **123102** @ 38400 Max_Master=2

## Gates

| Gate | Status | Notes |
|---|---|---|
| Preflight | PASS | aarch64, eth0, serial by-id match, UDP 47808 free |
| Wiring | PASS | Human ohms confirmation |
| Passive RX (Vibe sniff) | PASS | source 2, 2265 frames, PFM, valid_ratio 0.9996 (tool ok=false expected) |
| MS/TP qualify join | PASS | next_station=2, tokens=37, samples=300; TTY released |
| Source G7 ReadProperty | PASS | ComplexACK both Object_Name and AI:1 |
| Source G8 Who-Is/I-Am + I-Am-Router | PASS | unbounded remote Who-Is; I-Am SNET 2001 SADR 02 |
| Exact-image G7/G8 | OPEN | Buildroot appliance not in this slice |
| G9 restart/fault | OPEN | One whole-process neg/pos control only; not G9 |
| FX Workbench UI | PARTIAL | Operator screenshot not attached; packet oracle PASS |

## Single next blocker

Land router-owned `--mstp-passive` (this PR) + attach Workbench screenshot if desired for human UI evidence. Exact-image G7/G8 remains open until Buildroot repeat.
