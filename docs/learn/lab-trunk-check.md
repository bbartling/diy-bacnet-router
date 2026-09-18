---
title: Lab trunk check (beginner)
parent: Learn
nav_order: 3
permalink: /learn/lab-trunk-check/
---

# How we prove the live MS/TP trunk (beginner)

This page explains a real lab check in plain language: **can we reach the little
computers over the network, and do we hear them talking on the shared serial
cable?** You do **not** need to know MS/TP, USB-UART chips, or Linux internals
first — those words are translated below.

{: .important }
This is a **lab / source** check on real hardware. It is **not** the same as a
green GitHub Actions job, and it is **not** a BTL certificate. See also
[How we test]({{ site.baseurl }}/learn/testing/) and
[What “Clause 9” means](#what-clause-9-means-without-the-scary-words).

---

## Picture of the lab

Think of three kinds of gear on one **shared two-wire cable** (RS-485), like a
party line:

```text
  Desktop PC ("bensbench")
       |  USB stick that talks RS-485 (Waveshare)
       |
  ==== shared cable (the "trunk") ====
       |              |              |
   Raspberry Pi 1   Raspberry Pi 2   Building controller (FEC)
   (fake MS/TP      (fake MS/TP      (real device — read only)
    mini device)     mini device)
```

Separately, each Pi and the desktop also have a normal **Ethernet** plug so we
can `ssh` in (remote login) and so BACnet/IP tools can talk to a **router**
program that bridges Ethernet ↔ the serial trunk.

| Friendly name | Network address | Job in this check |
| --- | --- | --- |
| bensbench | `192.168.204.11` | Desktop that owns one USB↔RS-485 adapter; we listen here |
| workerpi1 | `192.168.204.59` | First Raspberry Pi + its own USB↔RS-485 adapter |
| workerpi2 | `192.168.204.60` | Second Raspberry Pi + USB↔RS-485 adapter; already running a “mini” device |
| FEC | on the cable only | Real controller; **baud fixed at 38400**; we only **read** it |

---

## Word bank (30 seconds)

| Term | Plain meaning |
| --- | --- |
| **SSH** | Secure remote login to a Pi from your PC (`ssh ben@192.168.204.59`). Proves “the computer is on and reachable,” not that the serial cable works. |
| **USB-UART / FTDI / CH343** | Tiny chips that turn USB into a serial port. Linux shows them as `/dev/ttyUSB0` or `/dev/ttyACM0`. |
| **`/dev/serial/by-id/...`** | A **stable nickname** for that USB stick. Prefer this over `ttyUSB0`, which can renumber after unplug. |
| **RS-485 / trunk** | The shared two-wire cable all stations listen/talk on. |
| **MS/TP** | BACnet’s rules for taking turns on that cable (masters pass a **token**). |
| **MAC (0–127)** | Station number on the cable — like a seat number. Must be unique. |
| **Baud (e.g. 38400)** | How fast bits go on the cable. **Everyone on the trunk must match.** Our FEC only does 38400. |
| **Passive listen** | Software that **only hears** frames; it does not join the token game. Safe first step. |
| **`sources_seen`** | List of station numbers the listener actually heard talking. |
| **`dialout` group** | Linux permission club that may open serial ports. If you are not a member, you get “Permission denied.” |
| **Clause 9** | The chapter of the BACnet standard about MS/TP behavior (timing, tokens, frames). Separate from paid **BTL** listing. |

More protocol background: [BACnet, IP, and MS/TP]({{ site.baseurl }}/learn/bacnet-mstp/).  
More Linux serial tips: [Linux, USB serial, and MS/TP timing]({{ site.baseurl }}/learn/linux-mstp-timing/).

---

## What we ran (the validation)

### Step 1 — Can we log into both Pis?

We used SSH to each Pi and checked:

- hostname and Ethernet address
- whether a USB serial adapter showed up under `/dev/serial/by-id/`
- whether an MS/TP “mini device” program was already running

**Result:**

| Check | Result (plain English) |
| --- | --- |
| SSH workerpi1 `.59` | OK — Pi answers; FTDI USB adapter `BH002I9S` is plugged in |
| SSH workerpi2 `.60` | OK — Pi answers; mini program already running as station **MAC 2**, device instance **123102** |

{: .warning }
SSH success only means **Ethernet + login** work. It does **not** prove the
RS-485 cable is correct. That is the next step.

### Step 2 — Who is talking on the cable?

On the desktop we ran a **passive** listen on the Waveshare USB adapter
(about 12–14 seconds). The tool counts complete frames and reports which
station numbers it heard (`sources_seen`).

First, workerpi1 was **not** running a mini program — so it could not appear as
a talker yet. We started a **temporary** mini on pi1 as **MAC 3** / instance
**123103**, then listened again from the desktop.

**Result:**

| Check | Result (plain English) |
| --- | --- |
| Trunk hear (bensbench Waveshare) | Heard stations **`[2, 3, 7]`** — pi2 mini, pi1 mini, and the FEC |

So:

- **2** = workerpi2 mini (was already on the wire)
- **3** = workerpi1 mini (started for this check; still running afterward)
- **7** = FEC (real controller on the same cable @ 38400)

Valid frame ratio was ~99.9% — the cable decode looked healthy for this sample.

### Step 3 — A Linux “gotcha” we hit (and fixed around)

After a reboot, the desktop user was **not** in the `dialout` group, so opening
`/dev/ttyUSB0` failed with **Permission denied**. We still completed the listen
by running the same binary inside a short-lived container that could open the
device.

**Fix when convenient (desktop):**

```bash
sudo usermod -aG dialout $USER
# then log out and back in (or reboot)
groups   # should list dialout
```

---

## What this does *not* prove

| Claim | Status after this check |
| --- | --- |
| Both Pis reachable on Ethernet | **Yes** |
| Both Pis (as minis) + FEC audible on one MS/TP trunk @ 38400 | **Yes** |
| DIY router is forwarding BACnet/IP → MS/TP right now | **Not this page** — needs `--route-enable` + a BIP client on another IP |
| Sensor values read through the router (bacpypes `read …`) | Separate oracle — see FEC evidence under `docs/evidence/PHASE2_FEC_VIA_DIY_*` |
| Flashed Buildroot appliance image behaves the same | **Open** (milestone M4) |
| Full Clause 9 / BTL | **No** — see below |

---

## What “Clause 9” means (without the scary words)

**Clause 9** is the part of the BACnet standard that defines **MS/TP**: how
stations share the RS-485 cable, pass the token, time their replies, and format
frames.

| Phrase people say | What it should mean here |
| --- | --- |
| “We heard the trunk” | Passive listen / join saw tokens and station numbers — **necessary**, not full Clause 9 |
| “Field-ready on our bench” | Router + real cable + real/fake devices + routed reads in **our** lab evidence |
| “Clause 9 self-tested” | Written checklist (PICS-like): which bauds, frame types, and behaviors we **proved on the wire**, and which we **do not claim** |
| “BTL listed” | Paid third-party lab stamp — **out of scope** for this project unless separately funded |

### What still has to be proven for a serious Clause 9 *self-test* claim

These are separate from “we heard MACs 2, 3, and 7 once”:

1. **Same baud everywhere** — and for bauds other than 38400, **unplug the FEC first** (it cannot change speed).
2. **Token / timing under load** — many reads in a row, USB latency set correctly.
3. **Reply Postponed / busy master** behavior when the station cannot answer immediately.
4. **Optional extended frames** — only claim them after dedicated wire proof.
5. **Longer soak** — hours, not seconds.
6. **Honest docs** — say what works; never imply BTL.

Routing between BACnet/IP and MS/TP is mostly **Clause 6** work. A product that
is useful in the field needs **both** healthy MS/TP (Clause 9) **and** correct
routing (Clause 6). This page only covered the “are they on the cable?” slice.

---

## Try the same ideas at home (commands)

Replace addresses and by-id paths with yours.

```bash
# 1) Reach a Pi
ssh ben@192.168.204.59

# 2) See USB serial nicknames
ls -la /dev/serial/by-id/

# 3) On the PC that holds the listen adapter: passive hear (example)
#    (requires dialout or equivalent access to the tty)
./target/debug/diy-bacnet-router \
  --config /path/to/router.toml \
  --mstp-passive --qualify-secs 12 \
  --expect-source 2 \
  --mstp-report /tmp/passive.json

# 4) Read the listener's summary
python3 -m json.tool /tmp/passive.json
# look for "sources_seen" — that is your "who talked" list
```

Interactive BACnet/IP troubleshooting (after a router is up) often uses
[BACpypes3 shell](https://gist.github.com/bbartling/bb08c2f81fa4e6189d608fac1f7fef73)
with `--route-aware`, from a **different IP** than the router.

---

## Next Learn pages

1. [BACnet, IP, and MS/TP]({{ site.baseurl }}/learn/bacnet-mstp/)
2. [Linux, USB serial, and MS/TP timing]({{ site.baseurl }}/learn/linux-mstp-timing/)
3. [How we test]({{ site.baseurl }}/learn/testing/) — CI vs lab vs image honesty
4. [Buildroot recipe]({{ site.baseurl }}/learn/buildroot-recipe/)
