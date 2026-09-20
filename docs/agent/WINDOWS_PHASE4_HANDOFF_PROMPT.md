# Windows Cursor handoff — DIY BACnet router (phase 4)

> **Context (2026-09-18):** This handoff is **temporary lab scaffolding**. Product north star is Home Assistant OS–like install: GH Actions Buildroot → **GitHub Releases** → docs download link. Prefer Mint/CI for image builds as soon as Releases exist; do **not** treat Windows+VMware as the long-term user path. FX Workbench on Windows may remain for optional Niagara UI evidence only.

**Canonical copy (always pull latest):**  
https://github.com/bbartling/diy-bacnet-router/blob/master/docs/agent/WINDOWS_PHASE4_HANDOFF_PROMPT.md  

Until PR #70 merges, tip is still:  
https://github.com/bbartling/diy-bacnet-router/blob/fix/repin-rusty-bacnet-7e0d13a/docs/agent/WINDOWS_PHASE4_HANDOFF_PROMPT.md  

---

## Paste block — start of Windows / VMware Cursor session

```text
You are on the Windows host / VMware Ubuntu guest for diy-bacnet-router image work (phase 4b — temporary lab path).

### 0) Pull updates first (do this every session)

On the VMware Ubuntu builder (or wherever the clone lives):

  cd ~/src/diy-bacnet-router   # or your clone path
  git fetch origin
  git checkout master
  git pull --ff-only origin master

If master does not yet have the tip pin / FEC evidence, use the merge branch until PR #70 is green+merged:

  git fetch origin
  git checkout fix/repin-rusty-bacnet-7e0d13a
  git pull --ff-only origin fix/repin-rusty-bacnet-7e0d13a

Confirm pins:

  git log -1 --oneline
  grep revision config/upstream-lock.toml
  # expect revision_short = 9e5168c5 (full 9e5168c5ac10bf06f3422f66fc2b0b9983a6ac5c)

Re-read this handoff after pull:
  docs/agent/WINDOWS_PHASE4_HANDOFF_PROMPT.md

PR (merge when CI green): https://github.com/bbartling/diy-bacnet-router/pull/70

### 1) Exact pins

| Item | Value |
|------|-------|
| rusty-bacnet | 9e5168c5ac10bf06f3422f66fc2b0b9983a6ac5c (dev tip) |
| FEC RP evidence | docs/evidence/PHASE2_FEC_VIA_DIY_20260918T124240Z/ |
| Protocol matrix | docs/evidence/PHASE3_PROTOCOL_MATRIX_20260917.md |

### 2) Mint lab truth (do NOT reconfigure from Windows)

- Router: bensbench 192.168.204.11 — Waveshare C FTDI; DIY MAC1; DNET 2001; --route-enable
- Mini: workerpi2 .60 — MAC2 / device 123102
- FEC: MAC7 / device 5007 — READ-ONLY (no writes)
- BIP oracle: workerpi1 .59 (or Workbench) — MUST be a different host than router BIP bind
- Baud 38400; Max_Master ≥ 7; never plant DNET 2000

Source routed RP to 2001:2 and 2001:7 already PASS on Mint. Your job is exact-image / Buildroot, not re-proving the trunk from Windows.

### 3) Your job (Windows)

1. Buildroot image P0s: USB-serial (FTDI + real CH343 bind), NIC drivers, eudev, POSIX FTDI latency helper, CD-ROM smoke hard-fail, SSH provisioning, /data persistence.
2. Carry the SAME rusty-bacnet SHA 9e5168c5… into image manifests / Buildroot package pins.
3. After images build: flash spare SD / disposable VMware ISO boot and repeat G7/G8-style oracle on the appliance binary (exact-image M4) — do not relabel Mint source evidence as image PASS.
4. Management UI: honest not-implemented for BBMD/extended until Mint matrix rows PASS; no fake BASRT branding.
5. When Mint lands HAOS-style GitHub Releases (M8), prefer downloading CI Release artifacts over local Buildroot; shrink this Windows path.

### 4) Do not

- Relabel lab/source evidence as image completion.
- Float rusty-bacnet without a coordinated lock PR.
- Touch live FEC wiring or write FEC points.
- Run BIP clients on the same host/IP as the DIY router UDP 47808 bind (false Timeout / unknown-route).
```

---

## After PR #70 merges

1. On Windows/VMware: `git checkout master && git pull --ff-only origin master`
2. Prefer the **master** blob link above (always current).
3. Continue image P0s / exact-image prove only — Mint owns live trunk + tip pin.
