# Agent implementation specification

For product intent, UI/metrics contract, Vibe13 prototype lineage and BASRT-class
educational scope, read [SOFTWARE_SPEC.md](SOFTWARE_SPEC.md) first.

For repository-wide audits, refactors, or pre-merge hardening, read
[FULL_STACK_AUDIT.md](FULL_STACK_AUDIT.md).

## Product outcome

Produce a small Linux appliance that routes BACnet NPDUs between one BACnet/IP
network and one BACnet MS/TP network, exposes an original management UI and
read-only observability API, and ships reproducible Raspberry Pi 3/4/5 and
x86-64 images.

The first production-shaped milestone is intentionally narrow:

- one B/IP port;
- one Waveshare USB TO RS485 (C) MS/TP port;
- standard MS/TP frames only until extended-frame support has evidence;
- no BBMD, FDR, BACnet/Ethernet, multi-MS/TP or browser configuration writes;
- SSH-managed Linux networking and application TOML;
- metrics, diagnostics and safe failure behavior.

## Execution contract for coding agents

An agent works one milestone at a time and must preserve a machine-checkable
evidence trail. Before editing, it records the repository default branch,
working branch, starting SHA, remotes, dirty files, open pull requests and the
latest relevant Actions runs. It never assumes that `main`, `master` or
`develop` is the active integration branch.

Every change must satisfy all of these rules:

1. preserve user files, captures and Git history;
2. add or identify an executable failing gate before a behavioral fix;
3. keep hardware, upstream-stack and appliance changes in separate commits and
   pull requests;
4. never weaken a test, mark a job optional or use `continue-on-error` to obtain
   a green check;
5. never expose the lab runner to pull requests from forks or unreviewed code;
6. record exact dependency SHAs, commands, timestamps and artifact names;
7. stop and report if success requires credentials, hardware transmission,
   signing, publishing or a destructive Git operation;
8. describe an unavailable check as `OPEN` or `BLOCKED`, never `PASS`.

The initial agent may alter scaffold implementation details, but it must keep
the public configuration and metrics contracts stable or include an explicit
schema migration and tests.

## Milestones

### M0 — Repository and management scaffold

Exit criteria: strict config tests, read-only API/OpenAPI, aggregate WebSocket,
Prometheus, Linux metrics, React build, CI and Buildroot skeleton are green.

M0 is not complete until all ordinary CI jobs pass from a clean checkout, the
x86 image boots in QEMU and answers `/healthz`, all Pi images build, every image
has checksums/manifest/legal-info, lockfiles are regenerated and reviewed, and
third-party Actions are pinned to audited full commit SHAs. Buildroot's host
Rust compiler is checked independently from `rust-toolchain.toml`.

### M1 — Upstream audit and adapter compile fixture

Audit current rusty-bacnet `dev`; pin a full SHA. Add
`crates/rusty-bacnet-adapter` using public B/IP, MS/TP and network-layer APIs.
Create deterministic in-memory tests before opening OS ports. File focused
upstream issues/PRs for missing generic behavior rather than forking internals.

### M2 — Independent port qualification

Qualify B/IP behavior on veth/network namespaces. Re-run Vibe13 passive and
master-node tests for the Waveshare C using this process and metrics contract.
No forwarding yet.

### M3 — Isolated routing

Implement actual NPDU forwarding, route discovery/network messages, broadcast
handling, hop-count behavior, size/capability rejection and loop prevention.
Use a fully isolated two-network bench. Prove routed Who-Is and ReadProperty in
both directions without adding an application object database.

### M4 — Faults, load and timing

Test USB unplug/replug, serial ownership, duplicate MAC, duplicate network,
wrong baud, NIC loss, malformed NPDUs, high request load, dashboard load and
clean shutdown/restart. Compare normal PREEMPT and PREEMPT_RT only with measured
worst-case evidence; do not require RT merely as a label.

### M5 — Appliance images

Boot x86 in QEMU, then test Pi 3/4/5 images. Run as an unprivileged service,
persist only configuration/evidence, provide SSH recovery, generate legal-info,
checksums, SBOM and version manifest. Add signed A/B updates only after base
images are stable.

### M6 — Management writes and optional capabilities

After routing is stable, add authenticated atomic configuration, audit trail,
TLS strategy, BBMD/FDR if required, extended MS/TP frames and formal
conformance/BTL work. Each is a separate gate.

## Definition of done for a router claim

A repository build, green unit tests, a running dashboard, a mini-device or a
single FEC read is insufficient. A router claim requires G7 and G8 evidence from
`docs/TESTING.md`, on an isolated topology, from the exact released image and
full dependency SHA.

## Evidence and release ledger

Each completed gate adds a small committed Markdown or JSON record containing:

- project SHA and dirty/clean state;
- full rusty-bacnet SHA when applicable;
- Buildroot version, image manifest and kernel version when applicable;
- exact command, start/end time, exit code and test topology;
- bounded summary counters plus links/paths to raw artifacts;
- operator confirmation for any active hardware run;
- explicit limitations and the next unopened gate.

Generated binaries and large captures stay in Actions artifacts or release
assets, not Git. A release is never created merely because a tag exists: it
requires the gate ledger for that milestone and immutable checksums.
