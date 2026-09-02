# Agentic AI software specification — DIY BACnet Router

This is the canonical product and engineering spec for coding agents working on
this repository. Read it with [AGENTS.md](../../AGENTS.md), [SPEC.md](SPEC.md),
and the prototype evidence at `py-bacnet-stacks-playground/vibe_code_apps_13`.

## Product intent (educational BASRT-class appliance)

Build an **original**, Linux-based BACnet/IP-to-MS/TP **router appliance** for
education and lab use — functionally comparable to a Contemporary Controls
**BASRT-B** class device, but:

- no copied branding, HTML, images, firmware blobs or trade dress;
- no false conformance, BTL, or routing claims without gate evidence;
- management UI inspired by **industrial commissioning patterns** (color-coded
  configuration sections, status counters, advanced diagnostics) using our own
  React/CSS and DBR identity.

The BASRT-B is a **reference for capability and layout**, not a clone target.
See [docs/product/BASRT_EDUCATIONAL_REFERENCE.md](../product/BASRT_EDUCATIONAL_REFERENCE.md).

## Prototype lineage (Vibe13)

Prior evidence lives outside this repo:

```text
~/py-bacnet-stacks-playground/vibe_code_apps_13/
```

| Phase | What it proved | Reuse here |
|-------|----------------|------------|
| Phase 1 | Raw RS-485 on Waveshare C adapters, 10k wire gate @ 38400 | Serial safety, by-id paths, baud policy |
| Phase 2 | Standard-frame MS/TP mini-device via rusty-bacnet | Adapter patterns, passive decode gates — **not** the object DB |
| Phase 3 (planned there) | B/IP↔MS/TP router | Becomes this repository's data plane |

**Hard boundary:** Vibe13's AI/BI/AV/BV mini-device database must **never**
become the router data plane. Import test vectors, runbooks and metrics ideas only.

Key Vibe13 agent docs to consult on the bench machine:

- `vibe13_agent_spec/SPEC.md`
- `vibe13_agent_spec/SUPERVISORY_METRICS.md`
- `vibe13_agent_spec/UI_STACK.md` — Streamlit lab vs Rust appliance split
- `docs/PHASE1_TEST_RESULTS.md`, `docs/PHASE2_HARDWARE_RUNBOOK.md`

## Hardware reference

Primary MS/TP adapter: [Waveshare USB TO RS485 (C)](https://www.waveshare.com/usb-to-rs485-c.htm)
(FT232RNL, isolated field side, automatic direction, onboard 120 Ω termination).

Project doc: [docs/hardware/WAVESHARE_USB_RS485_C.md](../hardware/WAVESHARE_USB_RS485_C.md).

Rules:

- persist `/dev/serial/by-id/...` only;
- 8N1, no Linux RS-485 ioctl / RTS / GPIO direction on Waveshare C;
- default baud 38400; allowed set 9600–115200 as listed in AGENTS.md;
- treat adapter as a **terminated endpoint**, not a mid-span tap on live trunks.

## Architecture

```text
  BACnet/IP (UDP)          routerd (Rust)           MS/TP (RS-485)
  host NIC / veth    <-->  NPDU forwarder    <-->  Waveshare C USB
                               |
                    atomics + bounded channel
                               |
              Axum REST + OpenAPI + WebSocket (management only)
                               |
                    React dashboard (static, embedded in image)
```

Linux **host networking** (IP, routes, DNS, SSH) is configured with normal
Buildroot/rootfs tools — not hidden BACnet env vars. Application policy lives in
`/etc/diy-bacnet-router/router.toml`.

## Management UI specification

### Transport

- REST: `/healthz`, `/api/v1/status`, `/api/v1/capabilities`, `/api/v1/metrics/snapshot`
- WebSocket: `/api/v1/ws/metrics` — **aggregate snapshots only** (default 1000 ms,
  bounded 250–5000 ms). Never one message per BACnet frame.
- Prometheus: `/metrics`

### Status page (BASRT-inspired, original styling)

When the data plane is operational, the UI must expose counters analogous to
commercial router status pages, including:

| Counter group | Fields (stable API names) |
|---------------|---------------------------|
| BACnet/IP 1 | `bip_rx_packets`, `bip_tx_packets` |
| MS/TP | `mstp_rx_packets`, `mstp_tx_packets` |
| Forwarding | `forwarded_bip_to_mstp`, `forwarded_mstp_to_bip`, `dropped_packets` |
| Token / master | `tx_tokens`, `rx_tokens`, `tx_poll_for_master`, `rx_poll_for_master` |
| MS/TP FSM | `rfsm_state`, `mnsm_state`, `next_station`, `poll_station`, `silence_timer_ms` |
| Integrity | `invalid_frames`, `header_crc_errors`, `data_crc_errors`, `event_count` |
| System | `cpu_percent`, `memory_available_bytes`, load averages, uptime |

Schema: [frontend/web/src/types.ts](../../frontend/web/src/types.ts). Counter
names are API contracts — change only with schema version bump and tests.

### Configuration page (phased)

| Milestone | Browser | SSH / TOML |
|-----------|---------|------------|
| M0–M3 | Read-only effective config JSON | Edit `router.toml`, restart service |
| M6+ | Authenticated writes with audit trail | Recovery / advanced |

BASRT-style **grouped sections** (device ID, B/IP network, IP settings, MS/TP)
are the UI target for M6. Colors and layout must be original (see product doc).

### Pages (current scaffold)

1. **Overview** — route map, commissioning lock, headline counters
2. **MS/TP** — token and FSM diagnostics
3. **BACnet/IP** — B/IP packet and forwarding stats
4. **System** — CPU, memory, load, temperature
5. **Configuration** — SSH instructions (read-only until M6)

Future: **Advanced** (BBMD/FDR — M6 gate), **Security** (TLS/auth — M6 gate).

## Buildroot / appliance

- Images: x86_64 (QEMU smoke), rpi3_64, rpi4_64, rpi5_64
- Entry: [scripts/build-image.sh](../../scripts/build-image.sh)
- CI: `.github/workflows/build-os.yml`
- Lab VM: VirtualBox `ubuntu2` @ `127.0.0.1:2222` — artifact acceptance before
  Buildroot debugging ([docs/operations/LOCAL_BUILDROOT_VM.md](../operations/LOCAL_BUILDROOT_VM.md))

Buildroot rootfs should include: `openssh`, basic `ip`/`systemd-networkd` or
Buildroot network init, USB serial udev by-id symlinks, unprivileged `routerd`
service, embedded web root.

## Agent execution order (current)

### If M0 image pipeline is uncertain

Follow [M0_ARTIFACT_ACCEPTANCE_PROMPT.md](M0_ARTIFACT_ACCEPTANCE_PROMPT.md):

1. Inspect newest `build-os` runs — do not assume failure.
2. Download successful x86 artifact; verify SHA256SUMS.
3. Boot under QEMU on lab VM; hit `/healthz`.
4. Optional clean local rebuild at same SHA; compare manifests.
5. Report M0 PASS or a specific blocker.

### After M0 is proven

Follow [CURSOR_CONTINUATION_PROMPT.md](CURSOR_CONTINUATION_PROMPT.md) one
milestone at a time (M1 rusty-bacnet adapter → M2 ports → M3 routing).

## Evidence and honesty

- Compilation, unit tests and QEMU **do not** prove RS-485 routing.
- Describe unavailable checks as `OPEN` or `BLOCKED`, never `PASS`.
- Do not copy commercial web UI HTML/CSS verbatim.
- Do not enable `router.enabled` or transmit on a live trunk without gate evidence.

## Related files

| File | Role |
|------|------|
| [AGENTS.md](../../AGENTS.md) | Non-negotiable engineering contract |
| [SPEC.md](SPEC.md) | Milestones M0–M6 and gate ledger |
| [M0_ARTIFACT_ACCEPTANCE_PROMPT.md](M0_ARTIFACT_ACCEPTANCE_PROMPT.md) | Current M0 agent assignment |
| [TESTING.md](../TESTING.md) | Gate labels G0–G11 |
| [.cursor/skills/local-buildroot-vm/SKILL.md](../../.cursor/skills/local-buildroot-vm/SKILL.md) | VirtualBox lab workflow |
| [.cursor/skills/basrt-educational-router/SKILL.md](../../.cursor/skills/basrt-educational-router/SKILL.md) | Product/UI context for agents |
