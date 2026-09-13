# Pcap / oracle proof table (source G7/G8)

| # | Expectation | Result |
|---|---|---|
| 1 | I-Am-Router-To-Network(2001) from workerpi1 | PASS — BVLC Original-Broadcast length 9 at session start (`01 80 01 07 d1`) |
| 2 | Who-Is scoped to remote network 2001 | PASS — unbounded Who-Is DNET 2001 DLEN 0 from tower `.55` |
| 3 | I-Am device 123102 SNET 2001 SADR 02 | PASS — `810a0019010807d101021000c40201e0de…` |
| 4 | Confirmed ReadProperty to DNET 2001 / DADR 02 | PASS — Object_Name + AI:1 Present_Value |
| 5 | ComplexACK SNET 2001 SADR 02 | PASS — Object_Name=`Rust MS/TP Mini Device`; AI:1 values `{1.0, 3.0, 4.0, 2.0}` |
| 6 | Negative control (router process exit) | PASS — no RP reply |
| 7 | Whole-process restart then fresh RP | PASS — PV=1.0; TTY free after stop |
| 8 | FX Workbench screenshot | PARTIAL — not supplied this run; tower BACnet oracle used |

Route report keeps `product_g7_g8_bip_mstp=OPEN` / `ready_to_route=false` by design (no forward counters at pin).
