# ISSUE66 baseline — 2026-09-16T184523Z

Immutable pre-fix capture for [#66](https://github.com/bbartling/diy-bacnet-router/issues/66).

## Topology (unchanged)

| Host | Role | Notes |
| --- | --- | --- |
| workerpi1 `192.168.204.59` | DIY router BIP↔MS/TP | FTDI Waveshare C → `ttyUSB0`; DNET **2001** MAC1; `--route-enable --qualify-secs 0` |
| workerpi2 `192.168.204.60` | Vibe13 mstp-mini-device | CH343 → `ttyACM0`; MAC2; instance **123102** |
| bensbench `192.168.204.11` | Controlled BIP client | bacpypes3 `--route-aware` network 1 |

Baud 38400, Max_Master=2, Max_Info_Frames=1. **Never DNET 2000.**

## Process / binary inventory

| Host | Unit | Active | RSS | CPU | Binary SHA256 (prefix) | Source SHA |
| --- | --- | ---: | ---: | ---: | --- | --- |
| workerpi1 | `diy-bacnet-router.service` | yes (since 2026-09-13) | ~5.5 MiB | ~1.6% | `d41035de…` | `b5a5708` (lab persist) |
| workerpi2 | `mstp-mini-device.service` | yes (since 2026-09-13) | ~4.8 MiB | ~1.4% | `96486f78…` | vibe13 `12e7d232` / rusty `af4e8868` |

- throttled=`0x0` both; Pi3 Model B aarch64 ~905 MiB; FTDI `latency_timer=16`
- Config: `/home/ben/lab/dbr-two-pi.toml` (management `127.0.0.1:8080`; BIP `192.168.204.59:47808`; MS/TP by-id FTDI; net 2001)
- Fix branch under test in checkout: `fix/issue-66-confirmed-read-timeout` @ `0e53780` (#65 tip); rusty-bacnet adapter pin `24e3439…`

## Controlled BIP oracle (not Workbench)

Client: bacpypes3 0.0.106, bind `192.168.204.11/24`, `--route-aware --network 1`.

| Probe | Result |
| --- | --- |
| `read_property("2001:2", device:123102, object-name)` | **PASS** 39 ms → `Rust MS/TP Mini Device` |
| `read_property("2001:2", analog-input:1, present-value)` | **PASS** 44 ms |
| `read_property("192.168.204.59", …)` (router as device) | **Timeout** (expected — objects live on remote station) |
| Burst 5 concurrent AI:1 | **5/5 OK**; latency 47 → 224 ms (queueing under Max_Info_Frames=1) |
| Sequential 30× AI:1 | **30/30 OK**; p50 **48.1 ms**, p95 **3038 ms**, max **3042 ms** |

Raw: [`controlled-rp.json`](controlled-rp.json).

### Baseline conclusion (pre-fix)

- Source route **works** for confirmed ReadProperty to DNET 2001 / MAC 2.
- **Latency outliers ~3 s** already appear under mild sequential polling — enough to explain intermittent Workbench `{fault, stale}` / `TransactionTimeoutException` when 5 points poll concurrently.
- CPU/RAM exhaustion is **not** indicated.
- Leading hypotheses for A2: MS/TP token + Max_Info_Frames=1 serialization; possible late ReplyPostponed / queue drop in rusty-bacnet pin `24e3439` (to validate with spans).

## Captures

| Artifact | Status |
| --- | --- |
| `controlled-rp.json` | present |
| `whois.json` / direct `.59` reads | present (global Who-Is saw plant nets; routed `2001:2` is the valid path) |
| B/IP pcap on Pi/host | **unavailable this run** (no passwordless `tcpdump` on eth0). Workbench/operator pcap still welcome. |

## Storm threshold (for A4)

- **Sustained storm:** >2 missed scheduled polls for any mapped point within a rolling 60 s window, or any point orange for >15 s continuous.
- Isolated single misses: report separately; do not auto-PASS.

## Gate

| Gate | Status |
| --- | --- |
| A0 baseline preserved | **PASS** (inventory + controlled latency distribution) |
| Workbench 5-point operator capture | **OPEN** (needs FX Workbench on Windows during soak) |
