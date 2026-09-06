# M2B handoff — physical MS/TP port qualification (Pi lab)

Executable plan only. **Do not execute from CI, VMware, QEMU, or GitHub Actions.**
Never claim M2B from netns evidence. End software work with a **human wiring pause**.

## Topology (locked)

```text
Ubuntu x86 tower = Ansible controller ONLY (not an MS/TP endpoint)
        |
workerpi1: exact-SHA native AArch64 diy-bacnet-router --mstp-qualify
           Waveshare C / FTDI, MAC 1, Max_Master = 2, 38400, port-only
        || isolated RS-485 @ 38400 (C ↔ B), two terminations, no BASRT/FEC trunk
workerpi2: Vibe13 mini-device, Waveshare B / CH343, MAC 2, Max_Master = 2
```

## Preconditions

1. Tip `master` has G6/M2A merged; record SHA and rusty-bacnet pin
   `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` (or audited successor).
2. First protocol qualify deploys **exact-SHA native AArch64** binaries to Raspberry Pi OS.
   Flashing Buildroot Pi images is a **later** image gate — not required for first M2B protocol pass.
3. Isolated RS-485 segment only — **never** attach live BASRT/FEC trunk on the tower.
4. Passive decode gate first (duplicate MAC / token storm / rising CRC → stop).
5. Two-node ring: MAC 1 + MAC 2 ⇒ **`Max_Master = 2` exactly** unless measured reasons justify otherwise.

## Software surface (already in tree when this PR lands)

```bash
# On workerpi1 (after scp of exact SHA binary + router.toml):
diy-bacnet-router --config /etc/diy-bacnet-router/router.toml --mstp-qualify --qualify-secs 120 \
  --mstp-report /tmp/mstp-qualify-report.json
```

- Opens exactly one `/dev/serial/by-id/...` with Waveshare **C** auto-direction profile.
- No B/IP socket; no `BACnetRouter`; `ready_to_route=false`; forwarding counters 0.
- Mirrors only public `MasterNode` fields (event_count, poll/next station, token_count-since-PFM).
- Aggregate CRC / product `tx_tokens` totals remain an **upstream gap** at the frozen pin — do not synthesize.

## Procedure (lab operator — after Vibe13 C↔B regression pair PASSes)

1. Configure workerpi1 fail-closed TOML: `router.enabled=false`, distinct nets, Waveshare C by-id,
   baud 38400, **mac=1**, **max_master=2**, max_info_frames=1.
2. Configure workerpi2 Vibe13 mini-device MAC 2 / 38400 / Max_Master 2. Vibe13 history **does not transfer**.
3. **HUMAN WIRING PAUSE:** verify isolated C↔B cabling, bias, exactly two terminations, no live trunk.
4. Passive decode gate, then start `--mstp-qualify` on workerpi1.
5. Record evidence under `docs/evidence/` with SHAs, commands, timestamps, exit codes, report JSON.
6. Mark **M2B only** — full M2 complete only after G6 + M2B.

## Explicit refusals

- No VMware/QEMU-fake M2B
- No forwarding / BBMD / FDR / Clause 9 claim
- No untrusted PR hardware jobs
- No Windows host serial ownership of the Pi trunk
