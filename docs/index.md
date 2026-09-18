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
**Today:** Source BIP↔MS/TP routing gates (G7/G8) and B/IP G6 are evidenced on the
lab bench; ordinary boot stays **fail-closed**. Exact-image proves and
**HAOS-style GitHub Releases** (download → flash → boot) are still **OPEN**.
Windows/VMware Buildroot is temporary lab scaffolding — not the long-term install path.

## Milestones

Canonical list: [README milestones](https://github.com/bbartling/diy-bacnet-router#milestones) (keep in sync).

- [x] **M0** — Scaffold, CI, Buildroot images (x86 + Pi), QEMU smoke
- [x] **M1** — rusty-bacnet pin + concrete B/IP/MS/TP compile/config fixtures (forwarding still off at ordinary boot)
- [x] **M2A / M2B** — B/IP G6 PASS; physical MS/TP qualify (source) PASS; exact-image M2B OPEN
- [x] **M3** — Isolated NPDU routing (source G7/G8)
- [x] **M3b** — FEC shared trunk + routed RP (source); `ready_to_route` product flip still gated
- [ ] **M4** — Exact-image G7/G8 on Buildroot appliance
- [ ] **M5** — Faults and MS/TP timing characterization
- [ ] **M6** — Production-shaped Pi flash/boot
- [ ] **M7** — Authenticated config writes
- [ ] **M8** — HAOS-style GitHub Releases + docs download link
- [ ] **M9** — Retire Windows/VMware as required image workflow

## Get started

1. [Quick start]({{ site.baseurl }}/quick-start/) — run `routerd` locally
2. [Build images]({{ site.baseurl }}/build-images/) — Buildroot **2026.05.2** pin (CI artifacts today; Releases = M8)
3. [Architecture]({{ site.baseurl }}/architecture/) — data vs management plane
4. [VMware lab]({{ site.baseurl }}/operations/local-buildroot-vm/) — optional Buildroot debug (not install north star)
5. [Hardware]({{ site.baseurl }}/hardware/waveshare-rs485-c/) — reference RS-485 adapter

## Pins

| Item | Value |
| --- | --- |
| Buildroot | **2026.05.2** (`config/buildroot-lock.toml`) |
| Rust (CI) | **1.93.0** |
| Release | **VERSION** file → dashboard header |
| rusty-bacnet | `acbf7baefe69d05f2368763dcc659d68e4bc114c` (dev tip pin; forwarding still disabled) |
