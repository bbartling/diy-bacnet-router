# AGENTS.md — DIY BACnet Router engineering contract

These rules apply to the entire repository. Read this file, [README.md](README.md),
[docs/agent/SOFTWARE_SPEC.md](docs/agent/SOFTWARE_SPEC.md),
[docs/agent/FULL_STACK_AUDIT.md](docs/agent/FULL_STACK_AUDIT.md),
[docs/agent/SPEC.md](docs/agent/SPEC.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md),
[docs/TESTING.md](docs/TESTING.md), and [docs/UPSTREAM_LOCK.md](docs/UPSTREAM_LOCK.md)
before changing code.

## Mission

Build a trustworthy, **original** Linux BACnet/IP-to-MS/TP router appliance for
**education and lab use** — functionally in the class of a Contemporary Controls
BASRT-B, but implemented as open Rust + Buildroot with honest evidence gates.

Deliverables:

- reproducible OS images (x86_64, Raspberry Pi 3/4/5);
- Rust data plane (`routerd`) for NPDU forwarding between distinct BACnet networks;
- management plane: Axum REST, OpenAPI, bounded WebSocket metrics, React dashboard;
- SSH-managed Linux networking and TOML application config.

This is **not** a BACnet application-device project (no AI/BI/AV/BV object database
as the router data plane).

## Prototype lineage

Phase 1–2 serial and MS/TP evidence lives in the external prototype:

```text
py-bacnet-stacks-playground/vibe_code_apps_13
```

Reuse: Waveshare C wiring runbooks, passive-decode gates, supervisory metrics
ideas, rusty-bacnet integration lessons.

**Do not import** the Vibe13 mini-device object database into this router.

## Reference hardware

Primary MS/TP adapter: [Waveshare USB TO RS485 (C)](https://www.waveshare.com/usb-to-rs485-c.htm)
(FT232RNL, isolated RS-485, automatic direction, onboard 120 Ω termination).

Read [docs/hardware/WAVESHARE_USB_RS485_C.md](docs/hardware/WAVESHARE_USB_RS485_C.md)
before bench or trunk work.

## Stack map (agents — do not conflate these)

| Layer | What it is | In product image? |
| --- | --- | --- |
| **rusty-bacnet** (upstream) | BACnet transport + network/router crates; **pinned SHA** | Yes (linked into `routerd`) |
| **`rusty-bacnet-adapter` / `router-core` / `routerd`** | This repo’s Rust appliance + fail-closed policy | Yes |
| **`frontend/web` (React)** | LAN management UI served by Axum — **not** the data plane | Yes (static assets) |
| **Buildroot external** | Custom Linux OS, eudev, USB-serial, service unit | Yes (when imaged) |
| **bacpypes3** | Python BIP **lab oracle** / shell (`whois`, routed `read`) | **No** — external client only |
| **Vibe13 `mstp-mini-device`** | Lab MS/TP fixture device | **No** — separate playground binary |
| **Workbench / tcpdump / BBMD tools** | Optional human evidence / debug | **No** |

Beginner Pages tutorial: [docs/learn/stack-map.md](docs/learn/stack-map.md)
(`https://bbartling.github.io/diy-bacnet-router/learn/stack-map/` after merge).

**bacpypes3 never replaces rusty-bacnet inside the appliance.** It proves the
Ethernet side can talk *through* our router to MS/TP stations. Bind bacpypes on
a **different host/IP** than the router’s UDP 47808 socket.

## Non-negotiable boundaries

- The router forwards NPDUs between distinct BACnet networks. It must not reuse
  the Vibe13 mini-device AI/BI/AV/BV database as its data plane.
- The dashboard, REST API and WebSocket are **management surfaces only**. They must
  not block token handling or packet forwarding.
- Do not copy commercial branding, firmware, HTML, images or trade dress (including
  BASRT-B web pages). Functional comparison for education is fine in private docs.
- Do not claim Clause 9 conformance, BTL certification, segmentation, extended
  frames, BBMD, FDR, routing, or a tested baud unless the named gate has current
  evidence.
- Prefer the pinned upstream rusty-bacnet APIs. Do not copy stack internals into
  this repository to make an API mismatch disappear.
- The default configuration is fail-closed: forwarding disabled, management
  bound to loopback, no default password, no write API.

## Management UI and metrics

The React dashboard is a **LAN-only operator console** behind the firewall — not
a public internet application. There is **no URL versioning** on management
routes (`/api/status`, not `/api/v1/status`); the appliance ships as one
cohesive firmware + UI unit.

- WebSocket **`/api/ws/metrics`** delivers **aggregate snapshots** (default 1000 ms,
  bounded 250–5000 ms) including MS/TP trunk health (token/PFM, FSM state, CRC
  errors). Never one message per BACnet frame.
- The browser **must always display** the release from root [`VERSION`](VERSION)
  (compiled into `routerd` as `DBR_VERSION`); the sidebar and header show it by default.
- Counter names in the metrics schema are stable API contracts (B/IP and MS/TP
  packet counts, token/PFM counters, RFSM/MNSM state, CRC errors, system stats).
- Browser configuration writes remain disabled until M6 auth/audit gates pass.
  Host IP and routes are configured over **SSH** with normal Linux tools.
- UI layout may follow industrial router **patterns** (grouped config sections,
  status counters); styling must be original DBR branding.

See [docs/agent/SOFTWARE_SPEC.md](docs/agent/SOFTWARE_SPEC.md) and
[docs/product/BASRT_EDUCATIONAL_REFERENCE.md](docs/product/BASRT_EDUCATIONAL_REFERENCE.md).

## Dependency policy

- Never depend on a moving `dev` branch in a committed `Cargo.toml`.
- Audit current upstream `dev`, run its relevant tests, then record a full
  40-character commit in `config/upstream-lock.toml` and `docs/UPSTREAM_LOCK.md`.
- Commit `Cargo.lock`. Use `--locked` in CI and Buildroot.
- Upstream stack changes belong in focused rusty-bacnet PRs with failing tests
  first. Application policy and appliance integration stay here.

### Daily rusty-bacnet / MS/TP watch (mandatory for agents)

Upstream [jscott3201/rusty-bacnet](https://github.com/jscott3201/rusty-bacnet) is under
**active daily enhancement**, especially MS/TP transport, frame decode, and router
paths. Every session that touches this appliance (or resumes after a hold) must:

1. Read the current pin in [`config/upstream-lock.toml`](config/upstream-lock.toml)
   and [`docs/UPSTREAM_LOCK.md`](docs/UPSTREAM_LOCK.md).
2. Fetch upstream tip (`git ls-remote` / compare `dev` or default branch HEAD) and
   note whether MS/TP, `mstp_frame`, serial, or `bacnet-network` router commits
   landed since the pin. **Prefer staying on tip of `jscott3201/rusty-bacnet@dev`
   when auditing a bump** — do not leave the pin months behind without a written reason.
3. Skim new upstream PRs/issues/changelog for MS/TP timing, token, CRC, stop/TTY
   ownership, and forwarding fixes — those directly affect lab timeouts
   ([issue #66](https://github.com/bbartling/diy-bacnet-router/issues/66)).
4. **Do not silently float the pin.** If a bump is warranted, follow **Repin gate**
   below. Prefer a focused PR here after any required rusty-bacnet PR merges.
5. If the pin stays: say so explicitly in the handoff (“upstream checked
   YYYY-MM-DD; pin still `9e5168c5…`; no MS/TP delta”).

### Repin gate (mandatory when changing any `bacnet-*` git `rev`)

Keep these **identical** after every bump (CI fails if they drift):

| Location | Field |
| --- | --- |
| [`config/upstream-lock.toml`](config/upstream-lock.toml) | `revision` (40 hex) + `revision_short` |
| Workspace [`Cargo.toml`](Cargo.toml) | all four `bacnet-{types,encoding,transport,network}` `rev =` |
| [`Cargo.lock`](Cargo.lock) | `cargo update -p …` so lock `rev=` matches |
| [`crates/rusty-bacnet-adapter/src/lib.rs`](crates/rusty-bacnet-adapter/src/lib.rs) | `UPSTREAM_REVISION` + `UPSTREAM_REVISION_SHORT` |
| [`docs/UPSTREAM_LOCK.md`](docs/UPSTREAM_LOCK.md) | documented full SHA |

**Commands agents must run locally before pushing a repin PR:**

```bash
# after editing revs + consts + lock docs:
cargo update -p bacnet-network -p bacnet-transport -p bacnet-encoding -p bacnet-types
bash scripts/test-upstream-pin.sh
cargo test -p rusty-bacnet-adapter --locked
cargo test --workspace --locked
bash scripts/validate-repository.sh
```

What those gates cover:

- `scripts/test-upstream-pin.sh` — lock ↔ Cargo.toml ↔ Cargo.lock ↔ adapter consts ↔ `UPSTREAM_LOCK.md`
- unit tests `pin_is_full_sha` + `pin_matches_workspace_cargo_toml_and_upstream_lock` in `rusty-bacnet-adapter` (run under `cargo test --workspace` in CI)
- `scripts/validate-repository.sh` → calls `test-upstream-pin.sh` + appliance contract (no hard-coded SHA; reads the lock)
- image verify (`build-os.yml` / `vm-debug-build.sh`) asserts `build-manifest.json` `rusty_bacnet` equals the lock file dynamically

Do **not** reintroduce hard-coded tip SHAs in contract scripts when the pin advances.

### Lab trunk baud + Open-FDD test bench (hold)

**Default / live trunk baud is `38400`.** Leave it there after FEC-off matrix work.
Re-attach the JCI FEC at **38400 only** (read-only). This dual-mini + optional FEC
ring on bensbench is the shared **Open-FDD OT / MQTT stress bench** — do not retune
baud for curiosity during Open-FDD soaks.

| Baud | Status on this Waveshare C + FTDI/CH343 lab |
| --- | --- |
| **38400** | **PASS** — supported (with or without FEC) |
| **76800** | **PASS** minis-only (FEC off) — claimed for that topology only |
| **19200** | OPEN — passive OK; routed RP → AbortPDU no-response (USB / fixed-ms reply window) |
| **9600** | OPEN — passive invalid / no tokens on this mixed trunk |

Evidence + hold: [`docs/evidence/CHECKPOINT_2026-09-18_CLAUSE9_BAUD_HOLD.md`](docs/evidence/CHECKPOINT_2026-09-18_CLAUSE9_BAUD_HOLD.md),
[`CLAUSE9_BAUD_MATRIX_FEC_OFF_*`](docs/evidence/CLAUSE9_BAUD_MATRIX_FEC_OFF_20260918T174441Z/result.md).
Upstream context: [rusty-bacnet#707](https://github.com/jscott3201/rusty-bacnet/issues/707) (related `#502`).

Do **not** claim 9600/19200 in PICS/docs until re-proven. Prefer discussing timing with
upstream rather than papering over USB-UART limits in this appliance.

## Spec and evidence (read before coding)

Source of truth for gates and claims:

| Doc | Role |
| --- | --- |
| [docs/agent/SPEC.md](docs/agent/SPEC.md) | Milestone / gate intent |
| [docs/TESTING.md](docs/TESTING.md) | Gate ledger G0–G11; **source vs exact-image** |
| [docs/agent/SOFTWARE_SPEC.md](docs/agent/SOFTWARE_SPEC.md) | Software contracts |
| [docs/evidence/CHECKPOINT_2026-09-13_SOURCE_G7_G8_HOLD.md](docs/evidence/CHECKPOINT_2026-09-13_SOURCE_G7_G8_HOLD.md) | Hold/resume after source G7/G8 |
| [docs/evidence/CHECKPOINT_2026-09-18_CLAUSE9_BAUD_HOLD.md](docs/evidence/CHECKPOINT_2026-09-18_CLAUSE9_BAUD_HOLD.md) | Baud matrix hold @38400; Open-FDD stress bench |
| [ansible/README.md](ansible/README.md) | Persistent two-Pi lab deploy |

Keep [README.md](README.md) milestone checkboxes honest: check **source** wins when
evidence exists; leave **exact-image / Buildroot** boxes open until that gate passes.
Update checkboxes in the same PR that lands the evidence.

## BACnet and serial safety

- Use `/dev/serial/by-id/...`, never persist `ttyUSB0`.
- Allow only 9600, 19200, 38400, 57600, 76800 and 115200 baud. Default 38400.
- Configure 8N1, no flow control. Waveshare automatic direction means no
  simultaneous Linux RS-485 ioctl, RTS or GPIO direction control.
- Exactly one process owns a tty. Never kill an unknown owner automatically.
- Validate that B/IP and MS/TP network numbers are distinct and in 1..=65534.
- Validate MS/TP MAC <= Max_Master <= 127 and Max_Info_Frames in 1..=255.
- Passive decode must pass before any hardware job transmits. Stop on duplicate
  MAC, loss of the existing trunk, token storm or rising CRC/timeouts.
- Never run hardware tests from an untrusted pull request.

## Metrics

- Data-plane counters use atomics or a bounded nonblocking channel.
- Browser updates are aggregate snapshots; no per-packet WebSocket messages.
- The WebSocket interval is bounded to 250..=5000 ms and defaults to 1000 ms.
- All queues, histories, captures and support bundles are bounded.
- Counter names and units are stable API contracts and require tests.

## Required tests before handoff

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
npm --prefix frontend/web ci
npm --prefix frontend/web run check
npm --prefix frontend/web run build
```

Also run `scripts/validate-repository.sh`. Hardware and Buildroot tests must be
reported separately and truthfully; lack of hardware is not a failure and is
never relabeled as a pass.

For repository-wide audits or refactors, follow
[docs/agent/FULL_STACK_AUDIT.md](docs/agent/FULL_STACK_AUDIT.md) and include its
completion report. Key stack facts: **rusty-bacnet (Rust) for BACnet** — no Python
in the data plane; **QEMU `-snapshot` smoke** for x86_64 images; **Buildroot**
for appliance images; **SSH-managed Linux networking** for host IP/routes.

## Agent workflow

1. Inspect the working tree and preserve user changes.
2. **Daily upstream check:** rusty-bacnet tip vs pin (see Dependency policy). Record
   result before assuming MS/TP behavior is unchanged.
3. For M0 image pipeline work, start with
   [docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md](docs/agent/M0_ARTIFACT_ACCEPTANCE_PROMPT.md)
   — verify Actions artifacts before editing Buildroot.
4. Identify one gate from [docs/agent/SPEC.md](docs/agent/SPEC.md) and confirm status
   in [docs/TESTING.md](docs/TESTING.md) / README milestone checkboxes.
5. Add a failing test or executable acceptance check.
6. Make the smallest implementation that passes it.
7. Run the required checks.
8. Update the evidence ledger, README checkboxes, and upstream lock if relevant.
9. Stop at hardware, signing, network mutation or release approval boundaries.

## Buildroot and local lab

- **Buildroot:** pin the **latest stable bugfix** in
  [`config/buildroot-lock.toml`](config/buildroot-lock.toml) (currently **2026.05.2**).
  Bump only after CI **and** lab VM x86 QEMU smoke pass. See
  [docs/UPSTREAM_LOCK.md](docs/UPSTREAM_LOCK.md).
- **Local builds:** Ubuntu guest in **VMware**, SSH from Windows host to
  `127.0.0.1:2222` — **not WSL**. Full topology in
  [docs/operations/LOCAL_BUILDROOT_VM.md](docs/operations/LOCAL_BUILDROOT_VM.md)
  and [docs/agent/SOFTWARE_SPEC.md](docs/agent/SOFTWARE_SPEC.md).
- When `build-os` fails: reproduce with `scripts/vm-debug-build.sh` on the guest,
  fix on a branch, push, confirm green Actions before claiming PASS.

## Cursor skills (project)

- [.cursor/skills/local-buildroot-vm/SKILL.md](.cursor/skills/local-buildroot-vm/SKILL.md) —
  VMware Ubuntu lab, artifact acceptance, Buildroot debug loop.
- [.cursor/skills/basrt-educational-router/SKILL.md](.cursor/skills/basrt-educational-router/SKILL.md) —
  product intent, UI/metrics contract, Vibe13 and Waveshare context.
