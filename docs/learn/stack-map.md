---
title: Stack map (who does what)
parent: Learn
nav_order: 2
permalink: /learn/stack-map/
---

# Stack map — rusty-bacnet, this app, bacpypes3, and friends

Beginners often ask: *“Is everything rusty-bacnet?”* **No.** This page is the
map. Read it before [Architecture]({{ site.baseurl }}/architecture/).

```text
                    LAB / HUMAN TOOLS (not in the appliance image)
  ┌─────────────────────────────────────────────────────────────────┐
  │  bacpypes3 shell / scripts   Workbench (optional)   tcpdump     │
  │  (Python BACnet/IP client)   (Windows FX UI)        Wireshark   │
  └───────────────────────────────┬─────────────────────────────────┘
                                  │ UDP 47808 (BACnet/IP)
                                  ▼
  ┌─────────────────────────────────────────────────────────────────┐
  │  THIS REPO — DIY BACnet Router appliance                        │
  │                                                                 │
  │  React UI  ──►  routerd (Axum HTTP/WS)  ──►  router-core        │
  │                         │                                       │
  │                         ▼                                       │
  │              rusty-bacnet-adapter  (thin wrap only)             │
  │                         │                                       │
  │                         ▼                                       │
  │         rusty-bacnet crates (PINNED upstream git SHA)           │
  │         bacnet-transport / bacnet-network / encoding / types    │
  │                         │                                       │
  │              B/IP socket ◄──────► MS/TP USB RS-485              │
  └─────────────────────────────────────────────────────────────────┘
                                  │ RS-485 trunk
                                  ▼
  ┌─────────────────────────────────────────────────────────────────┐
  │  LAB FIXTURES / FIELD DEVICES                                   │
  │  mstp-mini-device (Vibe13)   ·   real FEC / other MS/TP devices  │
  └─────────────────────────────────────────────────────────────────┘
```

---

## 1. rusty-bacnet (upstream ecosystem)

| Item | Detail |
| --- | --- |
| What | Open Rust BACnet **stack** — transports, NPDU routing, MS/TP master logic, codecs |
| Where | [jscott3201/rusty-bacnet](https://github.com/jscott3201/rusty-bacnet) |
| How we use it | **Pinned full git SHA** in `Cargo.toml` + `config/upstream-lock.toml` (never a floating `dev` branch in commits) |
| Crates we consume | `bacnet-types`, `bacnet-encoding`, `bacnet-transport`, `bacnet-network` |
| What it is **not** | Not our web UI, not our Buildroot OS, not bacpypes3 |

We prefer upstream public APIs. We do **not** copy stack internals into this
repo to paper over an API mismatch. Pin bumps follow the **Repin gate** in
[`AGENTS.md`](https://github.com/bbartling/diy-bacnet-router/blob/master/AGENTS.md).

---

## 2. This repository (Rust + React + Buildroot) — outside “just the stack”

Everything below is **ours** (or our packaging). It sits *around* rusty-bacnet.

| Piece | Language | Job |
| --- | --- | --- |
| **`rusty-bacnet-adapter`** | Rust | Only crate allowed to call rusty-bacnet; maps ports/sessions into appliance types |
| **`router-core`** | Rust | Config, capabilities, metrics contracts — **no** Axum, serial, or BACnet deps |
| **`routerd`** | Rust | Process entry: open ports when opted in, HTTP/WebSocket/Prometheus, embed React |
| **`frontend/web`** | React / TypeScript | LAN operator dashboard (status, metrics). **Management only** — never blocks MS/TP |
| **Buildroot external** | Make / configs | Custom Linux OS image (kernel, eudev, USB-serial, service unit) |
| **Ansible lab playbooks** | YAML | Deploy dual mini fixtures / baud changes on Mint lab Pis |
| **Evidence + Learn docs** | Markdown | Honest gates; beginner tutorials on GitHub Pages |

{: .important }
The **web app is not rusty-bacnet**. It is React assets served by **`routerd`**
(Axum). Closing a browser tab must never stop token passing on the RS-485 trunk.

Default boot is **fail-closed**: management may run; BACnet forwarding stays off
until a controlled lab flag such as `--route-enable`.

---

## 3. bacpypes3 — lab / field troubleshooting client (not in the product)

| Item | Detail |
| --- | --- |
| What | Python BACnet/IP stack + interactive **shell** |
| Where | [bacpypes3](https://pypi.org/project/bacpypes3/); cheat sheet [gist](https://gist.github.com/bbartling/bb08c2f81fa4e6189d608fac1f7fef73) |
| How we use it | **Oracle** on the Ethernet side: `whois`, `wirtn`, `read 2001:7@router …` through our DIY router |
| Shipped in appliance image? | **No** |
| Same process as `routerd`? | **No** — separate venv / host; must use a **different BIP IP** than the router bind |

Typical lab flow:

1. Start DIY router on bensbench (BIP `.11`, MS/TP MAC1).
2. Run bacpypes3 on another address (e.g. Pi Ethernet `.59` or `.12` alias).
3. `read 2001:2@192.168.204.11 device,123102 object-name` — proves **routing**, not the Python stack inside the appliance.

Scripts such as `scripts/lab_routed_rp_oracle.py` wrap the same idea for
non-interactive evidence packs.

---

## 4. Other things outside rusty-bacnet (lab / optional)

| Piece | Role |
| --- | --- |
| **Vibe13 `mstp-mini-device`** | Fake MS/TP device (MAC/instance) for trunk tests — lives in [py-bacnet-stacks-playground](https://github.com/bbartling/py-bacnet-stacks-playground); **not** imported as the router data plane |
| **Real controllers (e.g. JCI FEC)** | Field devices on the trunk; read-only in our lab; baud often fixed (ours: **38400**) |
| **FX Workbench** | Optional Windows BACnet/IP UI evidence — not required for CI |
| **tcpdump / Wireshark** | Packet proof on UDP 47808 |
| **ace-bbmd-manager** (mentioned in the gist) | Optional BBMD topology walks — **product BBMD stays off** until named gates pass |
| **GitHub Actions** | `ci`, `bip-qualify`, `build-os` — software / netns / **Buildroot images** (no local heavy Buildroot on the lab PC) |

---

## One-sentence summary

**rusty-bacnet** moves BACnet bits; **this repo** wraps it in an appliance OS +
management UI; **bacpypes3** is how humans/agents **test** the Ethernet side
from outside; **minis/FEC** are what live on the RS-485 cable.

## Next

- [BACnet, IP, and MS/TP]({{ site.baseurl }}/learn/bacnet-mstp/)
- [Linux, USB serial, and MS/TP timing]({{ site.baseurl }}/learn/linux-mstp-timing/)
- [How we prove the live MS/TP trunk]({{ site.baseurl }}/learn/lab-trunk-check/)
- [Architecture]({{ site.baseurl }}/architecture/) (engineering detail)
