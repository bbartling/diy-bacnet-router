# M2B handoff — physical MS/TP port qualification (Pi lab)

Executable plan only. **Do not execute from CI or VMware.** Never claim M2B from netns or QEMU.

## Topology

```text
B/IP peer / later netns
        |
workerpi1: router @ exact SHA + rusty-bacnet pin, Waveshare C, MAC 1, port-only
        || isolated 38400 MS/TP
workerpi2: Vibe13 mini-device, Waveshare B, MAC 2
```

## Preconditions

1. Tip `master` has M1 closeout + M2A merged; record SHA and upstream pin
   `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` (or successor lock).
2. Flash Pi images built for that SHA; verify VERSION / `rusty_bacnet_rev` on device.
3. Isolated RS-485 segment only — **never** attach live BASRT/FEC trunk.
4. Passive decode gate first (duplicate MAC / token storm / rising CRC → stop).

## Procedure (lab operator)

1. Configure `workerpi1` fail-closed: `router.enabled=false`, distinct nets, Waveshare C
   `/dev/serial/by-id/...`, baud 38400, MAC 1, Max_Master ≥ 1, auto-direction profile.
2. Bring up MS/TP **port-only** session (no `BACnetRouter` forwarding) analogous to M2A
   `--bip-qualify` — explicit opt-in; ordinary boot remains management-only.
3. On `workerpi2`, run Vibe13 mini-device at MAC 2 / 38400. Vibe13 history **does not transfer**.
4. Metrics contract: aggregate WebSocket snapshots; token/PFM, RFSM/MNSM, CRC errors;
   forwarding counters stay 0; `ready_to_route=false`.
5. Record evidence under `docs/evidence/` with SHAs, commands, timestamps, exit codes,
   and pcap/log bounds. Mark **M2B only** — full M2 complete only after both A and B.

## Explicit refusals

- No VMware-fake M2B
- No forwarding / BBMD / FDR / Clause 9 claim
- No untrusted PR hardware jobs
