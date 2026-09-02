---
name: basrt-educational-router
description: >-
  Product and UI context for the DIY BACnet Router — educational BASRT-class
  appliance, Vibe13 prototype lineage, Waveshare C adapter, WebSocket metrics
  contract, and original industrial-style dashboard. Use when implementing UI,
  metrics, configuration pages, or explaining product scope to the operator.
---

# BASRT-class educational router (agent context)

## Product

Build an **original** Linux BACnet/IP-to-MS/TP router for education — same
**operational ideas** as a Contemporary Controls BASRT-B, not a clone.

Read:

- [docs/agent/SOFTWARE_SPEC.md](../../docs/agent/SOFTWARE_SPEC.md)
- [docs/agent/FULL_STACK_AUDIT.md](../../docs/agent/FULL_STACK_AUDIT.md) — audits/refactors
- [docs/product/BASRT_EDUCATIONAL_REFERENCE.md](../../docs/product/BASRT_EDUCATIONAL_REFERENCE.md)
- [AGENTS.md](../../AGENTS.md)

## Prototype (external)

```text
py-bacnet-stacks-playground/vibe_code_apps_13
```

- Phase 1: Waveshare C wire test @ 38400 — serial path evidence
- Phase 2: MS/TP mini-device — **do not** import its object DB here
- Vibe13 specs: `vibe13_agent_spec/SUPERVISORY_METRICS.md`, `UI_STACK.md`

## Hardware

[Waveshare USB TO RS485 (C)](https://www.waveshare.com/usb-to-rs485-c.htm) —
FT232RNL, auto direction, onboard 120 Ω. See
[docs/hardware/WAVESHARE_USB_RS485_C.md](../../docs/hardware/WAVESHARE_USB_RS485_C.md).

## UI / WebSocket contract

- Endpoint: **`GET /api/ws/metrics`** — snapshot every 1000 ms (250–5000 allowed); MS/TP trunk health
- Schema: [frontend/web/src/types.ts](../../frontend/web/src/types.ts)
- Status counters to surface when data plane runs (BASRT-analog):

  | Group | Keys |
  |-------|------|
  | B/IP | `bip_rx_packets`, `bip_tx_packets` |
  | MS/TP | `mstp_rx_packets`, `mstp_tx_packets` |
  | Tokens | `tx_tokens`, `rx_tokens`, `tx_poll_for_master`, `rx_poll_for_master` |
  | FSM | `rfsm_state`, `mnsm_state`, `next_station`, `poll_station`, `silence_timer_ms` |
  | Errors | `invalid_frames`, `header_crc_errors`, `data_crc_errors`, `event_count` |

- Pages: Overview, MS/TP, BACnet/IP, System, Configuration (read-only until M6)
- Future: Advanced (BBMD M6), Security (M6) — **not** claimed until gated

## Styling rules

- Color-coded config **panels** (original palette, not BASRT HTML copy)
- Sidebar nav + live WebSocket connection badge
- No Contemporary Controls logo, photos, or copied CSS

## Networking

- Host IP/routes: SSH + standard Linux (Buildroot image)
- App config: `/etc/diy-bacnet-router/router.toml`
- Browser writes disabled until M6

## Agent boundaries

- Metrics from atomics/channel — management plane never blocks forwarding
- Zero counters / `not started` FSM until rusty-bacnet + hardware gates pass
- Do not claim routing, BBMD, or BTL without gate evidence in TESTING.md
