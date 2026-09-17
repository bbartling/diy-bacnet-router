# Phase 2 — READY for FEC wire (operator gate)

## Already done on LAN (2026-09-17)

- rusty-bacnet tip **`7e0d13a`** deployed to workerpi1 (musl aarch64).
- Phase-1 five-point SLO **PASS** (see `PHASE1_FIVE_POINT_20260917T180541Z`).
- **Max_Master = 7** on DIY router TOML and mini systemd unit (prep for FEC MAC7).
- Example TOML: `config/router.lab-fec-shared-trunk.example.toml`
- Smoke after Max_Master bump: AI:1 seq20 **20/20**, p95 **37.3** ms.

## Ben must do (cannot remote)

1. Approve maintenance window; **physically isolate BASRT** from this RS-485 segment.
2. Daisy-chain **DIY (Waveshare C) + JCI FEC + mini (Waveshare B)** @ 38400; two end terminations only; no adapter 5V→FEC.
3. Confirm live MACs: DIY=1, mini=2, FEC=**7** (or update Max_Master if different).
4. Keep DNET **2001** (or reclaim 2000 only with BASRT fully off BIP ads for that net).
5. Niagara: discover FEC **5007** + mini **123102**; read-only; **no FEC writes**.
6. 30+ min concurrent poll; save screenshots + note intervals/timeouts.

## Agent will do immediately after you say “FEC wired”

- Who-Is / RP soak for FEC AI targets you verify.
- Capture p50/p95/p99/max + timeouts.
- Restore BASRT instructions if FAIL.

## Rollback

- Router TOML bak: `/home/ben/lab/dbr-two-pi.toml.bak-maxmaster2`
- Mini unit bak: `/etc/systemd/system/mstp-mini-device.service.bak-mm2`
- Binary bak: `/home/ben/lab/diy-bacnet-router.bak-b5a5708`
