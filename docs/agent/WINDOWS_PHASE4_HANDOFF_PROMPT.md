# Windows Cursor handoff — DIY BACnet router (phase 4)

Paste this into the Windows / VMware Cursor session after Mint confirms phase-2 FEC trunk (or in parallel for image P0s that do not need the trunk).

## Exact pins (update if Mint advances)

| Item | Value |
|------|-------|
| App branch / tip | `fix/repin-rusty-bacnet-7e0d13a` — see latest master after merge of PR https://github.com/bbartling/diy-bacnet-router/pull/70 |
| rusty-bacnet | **`7e0d13a7c527da726b5aa27ff263e7eb8375131b`** (`dev` tip, crates **v0.11.0**) |
| Prior pin | `24e3439` (do not regress) |
| Phase-1 evidence | `docs/evidence/PHASE1_FIVE_POINT_20260917T180541Z/` — five-point SLO PASS |
| Phase-2 gate | `docs/evidence/PHASE2_FEC_TRUNK_READY_20260917.md` — **Ben wires FEC**; Max_Master already 7 on Pis |
| Protocol matrix | `docs/evidence/PHASE3_PROTOCOL_MATRIX_20260917.md` |

## Lab hardware (Mint-owned; do not reconfigure from Windows)

- workerpi1 `192.168.204.59` — FTDI `usb-FTDI_FT232R_USB_UART_BH002I9S-if00-port0`, latency_timer=**1**
- workerpi2 `192.168.204.60` — CH343 `usb-1a86_USB_Single_Serial_5A98075745-if00`
- Baud 38400; DNET **2001**; DIY MAC1; mini MAC2 / **123102**

## Your job (Windows)

1. Buildroot image P0s: USB-serial (FTDI + real CH343 bind), NIC drivers, eudev, POSIX FTDI latency helper, CD-ROM smoke hard-fail, SSH provisioning, `/data` persistence.
2. Carry the **same** rusty-bacnet SHA `7e0d13a…` into image manifests.
3. After Mint phase-2/3 PASS on source, flash spare SD and **repeat** five-point + FEC trunk + protocol smokes on exact image.
4. Management UI: show honest not-implemented for BBMD/extended until Mint matrix rows PASS; no fake BASRT branding.

## Do not

- Relabel lab evidence as image completion.
- Float rusty-bacnet without coordinating with Mint lock PR.
- Touch live BASRT / plant FEC wiring from the Windows agent.
