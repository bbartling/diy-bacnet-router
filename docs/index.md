---
title: Home
layout: default
nav_order: 1
permalink: /
---

# DIY BACnet Router

A small **custom Linux OS** (Buildroot) that routes **BACnet/IP ↔ MS/TP**, with a
Rust data plane and a LAN management UI.

{: .important }
Ordinary boot is **fail-closed** (no forwarding). Lab **source** routing is
evidenced; proving the same gates on a flashed **Buildroot image**, then shipping
**GitHub Release** downloads from this site, are the open product steps.

## Learn (start here)

1. [What is BACnet / IP / MS/TP?]({{ site.baseurl }}/learn/bacnet-mstp/) — beginner overview
2. [Stack map]({{ site.baseurl }}/learn/stack-map/) — rusty-bacnet vs bacpypes3 vs this app’s Rust/React/Buildroot
3. [Linux, USB serial, and MS/TP timing]({{ site.baseurl }}/learn/linux-mstp-timing/) — why latency and by-id paths matter
4. [How we prove the live MS/TP trunk]({{ site.baseurl }}/learn/lab-trunk-check/) — SSH + cable listen explained simply; what Clause 9 means (not BTL)
5. [Buildroot recipe]({{ site.baseurl }}/learn/buildroot-recipe/) — what services make the appliance OS
6. [How we test]({{ site.baseurl }}/learn/testing/) — CI vs lab vs exact-image honesty

## Build & run

| Page | What you get |
| --- | --- |
| [Quick start]({{ site.baseurl }}/quick-start/) | Run `routerd` on a workstation |
| [Build images]({{ site.baseurl }}/build-images/) | GitHub Actions `build-os` → artifacts (Releases = milestone M8) |
| [Architecture]({{ site.baseurl }}/architecture/) | Data plane vs management plane |
| [Hardware]({{ site.baseurl }}/hardware/) | Waveshare USB RS-485 reference |

## Pins

| Item | Value |
| --- | --- |
| Buildroot | **2026.05.2** |
| Rust (CI) | **1.93.0** |
| rusty-bacnet | `9e5168c5ac10bf06f3422f66fc2b0b9983a6ac5c` |

## Milestone snapshot

Keep the detailed checklist on the
[GitHub README](https://github.com/bbartling/diy-bacnet-router#milestones).

- **Done (source):** M0–M3b scaffold, B/IP qualify, physical MS/TP qualify, isolated + shared-trunk routing evidence
- **Open:** M4 exact-image prove → M8 Release downloads → M9 optional local Buildroot only
