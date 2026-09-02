# BASRT-B educational reference (not a clone spec)

This document explains **why** the DIY BACnet Router exists and **what** commercial
router patterns we study for education — without copying Contemporary Controls
branding, web assets, or trade dress.

## Reference device

The **Contemporary Controls BASRT-B** is a DIN-rail BACnet router bridging
BACnet/IP and MS/TP. Typical commissioning UIs expose:

- **Device identification** — name, instance, location
- **BACnet network settings** — Ethernet/B/IP network number, UDP port
- **IP settings** — address, subnet, gateway (Linux handles ours via SSH)
- **MS/TP settings** — MAC, network, Max_Master, Max_Info_Frames, baud, tolerance
- **Status** — packet counters, token state machine, memory, event flags
- **Advanced** — BBMD, foreign device registration, secondary B/IP port (out of M0–M3 scope)

Our educational goal: teach the same **operational concepts** on open Linux +
Rust + Buildroot, with honest evidence gates.

## What we mimic (functionally)

| BASRT-B concept | DIY BACnet Router approach |
|-----------------|----------------------------|
| B/IP packet counters | WebSocket snapshots: `bip_rx_packets`, `bip_tx_packets` |
| MS/TP packet counters | `mstp_rx_packets`, `mstp_tx_packets` |
| Token / PFM stats | `tx_tokens`, `rx_tokens`, `tx_poll_for_master`, `rx_poll_for_master` |
| RFSM / MNSM state | `rfsm_state`, `mnsm_state`, `next_station`, `poll_station` |
| Silence timer / events | `silence_timer_ms`, `event_count` |
| Available memory | `memory_available_bytes`, `process_rss_bytes` |
| Configuration save | SSH + TOML now; authenticated browser writes at M6 |
| IP setup | Standard Linux (`ip`, NetworkManager, or Buildroot init) over SSH |

Example status fields from a production BASRT-B (educational comparison only):

```text
BIP 1 Incoming/Outgoing Packets
MSTP Incoming/Outgoing Packets
SilenceTimer, EventCount, Flag
RFSM state, MNSM state (e.g. PassToken)
Next Station, Poll Station
TX/RX Token Count, TX/RX PFM count
Invalid long Frames, Available Memory
```

Our React dashboard already reserves schema slots for these; values stay zero or
`not started` until rusty-bacnet integration and hardware gates pass.

## What we do not copy

- Contemporary Controls logo, product photos, color-exact HTML/CSS
- Firmware binaries or web page source
- Marketing names (BASRT, BASRT-B) in our product UI — use **DIY BACnet Router**
- Claims of BTL certification, Clause 9 routing tables, or tested BBMD/FDR unless gated

## UI styling direction (original)

Inspired by industrial commissioning pages, implemented with our own design system:

- Color-coded **panels** for config groups (device, B/IP, IP, MS/TP)
- Sidebar navigation: Overview, MS/TP, BACnet/IP, System, Configuration
- Future: Advanced, Security (M6+)
- Live connection indicator driven by WebSocket state
- Read-only until auth/audit gates pass

Current scaffold: [frontend/web/src/App.tsx](../../frontend/web/src/App.tsx),
[frontend/web/src/styles.css](../../frontend/web/src/styles.css).

## Linux / Buildroot networking (operator path)

On the appliance image, operators use SSH for host IP configuration:

```bash
# Example — adjust for your Buildroot network backend
ip addr show
sudo ip addr add 192.168.204.200/24 dev eth0
sudo ip route add default via 192.168.204.1
```

USB adapter inventory:

```bash
ls -l /dev/serial/by-id/
dmesg | tail -20   # after plug-in
```

Application MS/TP settings remain in `/etc/diy-bacnet-router/router.toml`.

## Prototype bench

Phase 1–2 Waveshare C evidence: `py-bacnet-stacks-playground/vibe_code_apps_13`.
See its `docs/PHASE1_CHEATSHEET.md` and hardware runbooks before attaching to
a live building trunk.
