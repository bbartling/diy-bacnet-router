---
title: Home
layout: default
nav_order: 1
permalink: /
---

# DIY BACnet Router

**Open-source BACnet/IP to MS/TP router** — a custom Linux appliance (Buildroot +
Rust + React) for field routing, commissioning, and MS/TP trunk observability.

The end goal is a Home Assistant OS–style image: minimal Linux tuned for **MS/TP
timing**, a Rust data plane, and a built-in dashboard on the LAN — not a public
internet SaaS.

{: .important }
**Milestone 0 today:** the OS image pipeline, management API, and web UI scaffold
are in place. **NPDU forwarding is not enabled yet** — the router stays
fail-closed until routing gates pass with evidence.

## Milestones

- [x] **M0** — Scaffold, CI, Buildroot images (x86 + Pi), QEMU smoke
- [ ] **M1** — rusty-bacnet adapter pin + loopback tests (forwarding still off)
- [ ] **M2** — B/IP and MS/TP port qualification
- [ ] **M3** — Isolated NPDU routing on a bench
- [ ] **M4** — Faults and MS/TP timing characterization
- [ ] **M5** — Production-shaped Pi images
- [ ] **M6** — Authenticated config writes

## Get started

1. [Quick start]({{ site.baseurl }}/quick-start/) — run `routerd` locally
2. [Build images]({{ site.baseurl }}/build-images/) — Buildroot **2026.05.2** pin
3. [Architecture]({{ site.baseurl }}/architecture/) — data vs management plane
4. [VMware lab]({{ site.baseurl }}/operations/local-buildroot-vm/) — SSH Buildroot debug
5. [Hardware]({{ site.baseurl }}/hardware/waveshare-rs485-c/) — reference RS-485 adapter

## Pins

| Item | Value |
| --- | --- |
| Buildroot | **2026.05.2** (`config/buildroot-lock.toml`) |
| Rust (CI) | **1.93.0** |
| Release | **VERSION** file → dashboard header |
| rusty-bacnet | `24e3439694b7d286e57e0a80cf7f1df4bd39d8ad` (M1 pin; forwarding still disabled) |
