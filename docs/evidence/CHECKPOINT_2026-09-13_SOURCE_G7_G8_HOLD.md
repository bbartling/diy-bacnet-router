# Checkpoint — 2026-09-13 — source G7/G8 PASS; lab left running; project may pause

**Date (UTC):** 2026-09-13  
**Status:** Source-level BIP↔physical MS/TP **PASS**. Exact-image / Buildroot G7–G8 **OPEN**.  
**Hold note:** Safe to pause the product roadmap here. Resume checklist is at the bottom.

## What landed

| Item | Location |
|---|---|
| Source G7/G8 evidence pack | [`SOURCE_G7_G8_TWO_PI_20260913T175327Z_2f97d57979a9/`](SOURCE_G7_G8_TWO_PI_20260913T175327Z_2f97d57979a9/result.md) |
| Fail-closed `--mstp-passive` | merged in PR [#64](https://github.com/bbartling/diy-bacnet-router/pull/64) |
| Persistent lab (`--qualify-secs 0`) + Ansible | this checkpoint / `ansible/` |
| Gate ledger | [`docs/TESTING.md`](../TESTING.md) — source vs exact-image split |

## Locked lab topology (still the resume key)

```text
FX Workbench (any PC on 192.168.204.0/24)  B/IP net 1 UDP 47808
        |
workerpi1 192.168.204.59  diy-bacnet-router.service
  --route-enable --qualify-secs 0
  MS/TP net 2001 MAC 1  Waveshare C / FTDI BH002I9S
        |
  isolated RS-485 @ 38400 Max_Master=2
        |
workerpi2 192.168.204.60  mstp-mini-device.service
  MAC 2 device 123102  Waveshare B / CH343 5A98075745
```

Do **not** use MS/TP network 2000 (live BASRT advertises it).

## Pins at evidence time

| Pin | Value |
|---|---|
| Router SHA (bench) | `2f97d57979a9cd4b255663786e3929f579ccbba7` |
| rusty-bacnet | `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` |
| Vibe13 fixture | `12e7d232b23023e1bb06da7b75972d79ca584cf9` |
| Merge that recorded evidence | PR #64 → `56f27ef` on `master` |

## How to talk to the lab while it is running

**Workbench discovery**

1. Discover → Remote network **2001** (Global 65535 fallback)
2. Instance **123102**
3. Poll `device:123102 Object_Name` and `analogInput:1 Present_Value` (cycles ~1–4)

**Management UI (localhost on Pi)**

```powershell
ssh -N -L 18080:127.0.0.1:8080 ben@192.168.204.59
```

Open `http://127.0.0.1:18080`.

**systemd**

```bash
ssh ben@192.168.204.59 'systemctl status diy-bacnet-router --no-pager'
ssh ben@192.168.204.60 'systemctl status mstp-mini-device --no-pager'
```

**Redeploy**

```bash
ansible-playbook -i ansible/inventory/lab.yml ansible/playbooks/two_pi_lab.yml
```

## Explicitly still OPEN (do not claim)

- Exact-image / Buildroot appliance G7/G8
- G9 USB/serial restart ownership (upstream rusty-bacnet transport stop)
- BBMD/FDR, extended frames, segmentation, BTL
- Forwarding counters / dashboard BACnet proof-quality metrics
- FX Workbench screenshot in the evidence pack (tower BACnet oracle was used)

## Resume checklist (when un-pausing)

1. Read this file + [`SOURCE_G7_G8_.../result.md`](SOURCE_G7_G8_TWO_PI_20260913T175327Z_2f97d57979a9/result.md).
2. Confirm Pis still on `192.168.204.59/60`, serial by-id paths, ohms ~60–70 across bus.
3. `systemctl is-active diy-bacnet-router mstp-mini-device` (on respective hosts).
4. Workbench Remote 2001 / 123102 smoke.
5. Next product milestone: **exact-image G7/G8** on Buildroot with the same topology/oracle — not more Vibe13 feature work.
6. Optional: fix upstream `BACnetRouter::stop()` serial ownership before claiming G9.

## Stop lab before long power-off

```bash
ssh ben@192.168.204.59 'sudo systemctl disable --now diy-bacnet-router'
ssh ben@192.168.204.60 'sudo systemctl disable --now mstp-mini-device'
```
