# Cursor agent continuation prompt

Copy the text below into a Cursor agent after installing this scaffold into the
repository. This is an implementation assignment, not permission to claim
unverified BACnet behavior.

> **M0 image pipeline / Actions acceptance:** use
> [M0_ARTIFACT_ACCEPTANCE_PROMPT.md](M0_ARTIFACT_ACCEPTANCE_PROMPT.md) first.
> Use this prompt for post-M0 milestones (M1+ routing integration).

---

You are the lead engineer resuming the DIY BACnet Router appliance repository.
Work autonomously and methodically until the current milestone is genuinely
green. Read every mandatory repository instruction before editing anything.

## Mission

Turn this repository into an original Linux BACnet/IP-to-MS/TP router appliance
using Rust, the current audited `jscott3201/rusty-bacnet` stack, a small Axum
management API, a React dashboard and reproducible Buildroot images for x86-64
and Raspberry Pi 3/4/5.

Build an original product. Do not copy third-party branding, UI, firmware,
HTML, images or trade dress. Functional comparison is allowed in private
engineering reasoning only.

## Known evidence coming into this repository

- `bbartling/py-bacnet-stacks-playground/vibe_code_apps_13` completed raw serial
  Phase 1 and a server-only, standard-frame Rust MS/TP mini-device Phase 2 at
  38,400 baud.
- Operator reports PR #127 merged to `develop`, Gates 1–4 and 4b passed, and the
  final historical rusty-bacnet short pin was `af4e886`.
- Upstream PRs #467/#468 supplied prior MS/TP fixes/bindings. Do not assume
  current `dev` is still identical; audit it.
- The reference adapter is Waveshare USB TO RS485 (C): FT232RNL, isolated field
  side, automatic direction control and onboard 120 Ω termination. Its
  termination makes it an endpoint, not a transparent mid-span tap.
- Phase 2 mini-device success is not router success. Gates 5–6 from the old
  project were not completed and no routing claim exists.

## Mandatory reading

1. `AGENTS.md`
2. `README.md`
3. `docs/agent/SPEC.md`
4. `docs/ARCHITECTURE.md`
5. `docs/TESTING.md`
6. `docs/UPSTREAM_LOCK.md`
7. `docs/hardware/WAVESHARE_USB_RS485_C.md`
8. all current workflows, Cargo manifests and source
9. the actual current rusty-bacnet source and its nested `AGENTS.md`
10. Vibe13 agent/runbook/results files as historical evidence only

Inspect `git status`, branches, remotes, open PRs and recent Actions before
changing anything. Preserve all user changes and captures. Never reset, clean,
force-push or delete evidence.

## Part 0 — Make the bootstrap honest and green

Before BACnet integration:

1. Run the full local contract:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --locked -- -D warnings
   cargo test --workspace --all-targets --locked
   npm --prefix frontend/web ci
   npm --prefix frontend/web run check
   npm --prefix frontend/web run build
   ./scripts/validate-repository.sh
   ```

2. Correct real defects in the scaffold. Keep the public metrics field names
   stable unless a test proves they are wrong.
3. Run `actionlint` and validate every workflow. Pin third-party Actions to full
   commit SHAs after verifying their current releases.
4. Regenerate `Cargo.lock` and `frontend/web/package-lock.json` from their
   manifests in a networked clean checkout. The scaffold lockfiles were seeded
   offline and may contain harmless extra historical package records; do not
   treat that as a reviewed dependency set.
5. Verify the x86 Buildroot job from a clean checkout. Print the Buildroot host
   `rustc --version`. The repository rustup pin does not select Buildroot's host
   compiler, and current rusty-bacnet advertises Rust 1.93. Fix Buildroot
   package, MSRV/toolchain or QEMU boot issues rather than marking jobs
   optional.
6. Then verify all Pi image jobs compile. It is acceptable that hosted CI cannot
   boot the physical Pi images, but image creation, checksums, manifest and
   legal-info must be real.
7. Do not enable an automatic OS release until the image matrix is green.

Buildroot is an OS-image workflow, not an OCI multiarch build. Borrow Open-FDD's
matrix discipline, caching, immutable tagging and verification; do not paste its
container publishing action as the image builder.

## Part 1 — Audit and lock current rusty-bacnet

Use an isolated checkout:

```bash
mkdir -p ~/src
test -d ~/src/rusty-bacnet/.git || git clone https://github.com/jscott3201/rusty-bacnet.git ~/src/rusty-bacnet
cd ~/src/rusty-bacnet
git fetch origin --prune
git switch dev
git pull --ff-only origin dev
export RUSTY_BACNET_REV="$(git rev-parse HEAD)"
git status --short --branch
```

Record the full SHA, date and relevant merged PRs. Inspect rather than infer:

- MS/TP frame, serial and state-machine modules;
- standard/extended frame support and limits;
- serial TX drain/timer anchoring;
- Rust configuration validation;
- transport disconnect/health reporting;
- `TransportPort`, heterogeneous transport and `AnyTransport` APIs;
- `bacnet-network` routing table, forwarding, hop count and network messages;
- B/IP broadcast, BBMD and FDR paths;
- current conformance ledger and exact claim language.

Run at least the current upstream-prescribed gates plus focused serial/network
tests. Do not use commands copied from old notes if upstream renamed packages or
features. Record exact commands and results.

Update `config/upstream-lock.toml` and `docs/UPSTREAM_LOCK.md` with the full SHA.
Never commit `branch = "dev"`. Pin every consumed `bacnet-*` crate to the same
exact revision and commit `Cargo.lock`.

## Part 2 — Decide what belongs upstream

Re-evaluate, against current source, the old candidates:

- Rust `MstpConfig` validation parity;
- physical TX completion (`tcdrain`) before timer anchoring;
- clean transport health/disconnect events;
- a shared MS/TP endpoint/lifecycle usable by network routing;
- standard-frame size rejection without truncation;
- extended frame types 32/33, COBS and CRC-32K status.

If a generic stack defect remains:

1. create a focused branch from current upstream `dev`;
2. add a failing deterministic test first;
3. make the smallest generic correction;
4. run upstream gates;
5. open one focused upstream PR with no appliance code;
6. keep this application pinned to a reviewed commit/fork SHA while the PR is
   open and document why.

Do not open speculative PRs for behavior already fixed upstream. Do not copy a
stack implementation into this repo to avoid review.

## Part 3 — Implement the adapter, not another BACnet stack

Add `crates/rusty-bacnet-adapter`. It is the sole translation boundary between
upstream APIs and `router-core`.

Required shape:

- one B/IP transport bound to an explicitly configured Linux interface/address;
- one MS/TP master transport owning exactly one `/dev/serial/by-id/...` device;
- one network-layer/router lifecycle over the two ports;
- distinct valid network numbers;
- fail-closed startup and bounded shutdown;
- router status and actual state-machine counters mapped into the existing
  atomic metrics contract;
- no Phase 2 object database, AI/BI/AV/BV points or BACnet application-device
  role as a substitute for routing;
- no browser or logging call in the forwarding hot path;
- no unbounded queue or per-packet WebSocket publication.

If current upstream lacks a complete two-port router constructor, first add an
in-memory adapter compile/test fixture that identifies the exact missing public
surface. Propose that generic surface upstream. Do not invent a second NPDU
router in application code unless the owner explicitly changes direction.

Add deterministic integration tests for:

- local versus remote DNET classification;
- routed unicast both directions;
- global/remote broadcasts without echo loops;
- hop-count decrement/drop behavior;
- I-Am-Router-To-Network and Who-Is-Router-To-Network handling;
- unknown network and oversize/capability rejection;
- port loss and recovery;
- metrics accounting exactly once per accepted/forwarded/dropped packet.

Keep `router.enabled=true` rejected until those tests pass. Then replace the
temporary validation stop with a capability-aware gate; never merely remove it.

## Part 4 — Web/API and performance

Keep the original UI. It must display:

- B/IP incoming/outgoing;
- MS/TP incoming/outgoing;
- each forwarding direction;
- TX/RX Token and Poll For Master counts;
- CRC/invalid/timeout/drop/reconnect counters;
- silence timer, RFSM/MNSM, next/poll station when upstream exposes them;
- Linux uptime, CPU, load, memory, process RSS and temperature when available;
- exact project/image/rusty-bacnet revisions and capability/evidence status.

Use aggregate WebSocket snapshots, default 1 Hz and bounded 250–5000 ms. Add a
bounded connection limit and prove ten slow/disconnecting clients do not affect
the forwarding test. Prometheus and REST must use the same canonical snapshot.

Do not add browser configuration writes yet. SSH + TOML is the accepted initial
management path. Never return secrets in `/config/effective`.

## Part 5 — Hardware work requires operator readiness

Do no active bench transmission without explicit operator confirmation in the
current session. Before any test:

- use the exact by-id path and prove the tty is free;
- record the Waveshare C identity and FTDI latency;
- verify it is a physical endpoint because of onboard termination;
- verify exactly two terminations, reference and bias;
- begin with 38,400, unique MAC, conservative Max_Info_Frames=1;
- passive valid-frame/token/CRC gate before TX;
- stop immediately if the existing controller/workstation trunk degrades.

The self-hosted Actions runner may run only trusted manually dispatched commits
through protected environments. Never run fork PR code on it.

Isolated router acceptance needs two BACnet networks and independent clients:

```text
B/IP test client <-- Ethernet --> new router <-- Waveshare C / MS/TP --> known MS/TP peer
```

Prove a routed Who-Is/I-Am and ReadProperty both directions where the peers
support them. A local FEC ReadProperty by the router is client behavior, not
routing evidence.

## Part 6 — GitHub delivery discipline

- Work in focused branches; no force pushes after publication.
- Keep application and upstream stack PRs separate.
- Update docs and evidence with actual SHAs/results.
- Ensure no stale failing run is ignored and no duplicate PR exists.
- Use artifacts for PR builds and signed GitHub Release assets only after all
  required jobs pass.
- Never put signing keys on the BACnet lab runner.

## Required final report

Return a concise table with:

1. repository branch and SHA;
2. current rusty-bacnet full SHA and audit result;
3. commands/tests run with pass/fail/blocked status;
4. Buildroot image results per target and QEMU evidence;
5. exact upstream gaps and PR/issue links, if any;
6. router gates now passed versus still open;
7. hardware actions not run and why;
8. next smallest safe ticket.

Allowed claim before isolated routing evidence:

> The repository builds a management-plane appliance scaffold and preserves
> prior MS/TP prototype evidence; BACnet/IP-to-MS/TP forwarding is not yet
> verified.

Do not call the product a working router until G7 and G8 pass on the exact image
and dependency revision.

---
