# Cursor prompt — close Vibe13 without expanding it

> **External repository only** — run this in `bbartling/py-bacnet-stacks-playground`,
> not in `diy-bacnet-router`. For this appliance, use
> [SOFTWARE_SPEC.md](SOFTWARE_SPEC.md) and [M0_ARTIFACT_ACCEPTANCE_PROMPT.md](M0_ARTIFACT_ACCEPTANCE_PROMPT.md).

Use this in `bbartling/py-bacnet-stacks-playground` after the operator is ready
to finish the prototype evidence.

---

Close out `vibe_code_apps_13` as a stable historical prototype. Do not turn it
into the router appliance and do not repin it merely because rusty-bacnet `dev`
has moved.

Read the root and Vibe13 `AGENTS.md`, README, phase results, hardware runbook,
PICS evidence and the actual pinned rusty-bacnet source before acting. Inspect
the working tree, current `develop`, PR #127 merge, recent Actions, remote
branches and untracked captures. Preserve every user change and evidence file.
Never reset, clean, force-push or kill an unnamed serial owner.

## First establish the checkpoint

1. Confirm the exact project SHA and full rusty-bacnet SHA represented by the
   merged Vibe13 code. Operator reports the historical short pin as `af4e886`;
   resolve the full commit from the lock/source, not by guessing.
2. Confirm Gates 1–4 and 4b evidence still points to the exact tested pin and
   label any earlier captures historical.
3. Run the existing offline contract exactly as defined by the repository:
   formatting, Clippy, locked workspace tests, no-IP proof and loopback/gate
   profiles.
4. Confirm GitHub has no accidental open Vibe13 PR, stale feature branch or
   failing current-tip workflow. Do not manufacture a cleanup PR if no change
   is needed.

## Only worthwhile remaining Phase 2 hardware evidence

Hardware work requires explicit operator confirmation in the current session,
the exact `/dev/serial/by-id/...` path and confirmation that the existing
supervisory workstation/trunk is healthy.

The reference Waveshare USB TO RS485 (C) has automatic direction control and an
onboard 120 Ω resistor. Treat it as one physical endpoint termination. Do not
place it mid-span on a bus that already has two terminations. Do not enable
Linux RS-485 ioctl, RTS or GPIO direction control.

Run, in order, stopping on trunk degradation, duplicate MAC, token storm,
rising CRC errors or APDU timeouts:

1. preflight inventory, tty ownership and passive valid-frame/token gate;
2. existing mini-device hardware gate at 38,400 with current tested pin;
3. one-hour mini-device soak while the supervisory workstation remains online;
4. only if clean, schedule a separate 24-hour soak;
5. USB unplug test: bounded error/exit and clean restart, not a hung process;
6. Linux timing/load baseline: kernel/PREEMPT/high-resolution timer flags, CPU
   governor, FTDI latency, USB topology, idle and loaded worst-case latency;
7. optional isolated C↔C maximum-standard-frame test and segmented/oversized
   negative tests. Never run these stress cases on the live controller trunk.

Every artifact must include project SHA, full rusty-bacnet SHA, kernel/arch,
USB IDs and serials, by-id path, baud, latency setting, topology, termination,
reference/bias notes, start/end, counters and exit reason.

## Explicitly do not add to Vibe13

- no shared client+server MS/TP endpoint work;
- no 30-second FEC point mirror;
- no B/IP-to-MS/TP router;
- no dashboard or Buildroot image;
- no BBMD/FDR;
- no extended-frame or conformance claim;
- no continual chase of moving upstream `dev`.

Generic shared-endpoint, transport-health or routing gaps belong in focused
rusty-bacnet work. Appliance integration belongs in the new DIY BACnet Router
repository. A local FEC read or mirrored point is application-client behavior,
not router evidence.

## Closeout result

Update only truthful evidence/status documents. If changes are necessary, use a
focused branch and PR; otherwise report that no code change was needed. Tag the
checkpoint only after offline CI and the selected hardware closeout are clean.

Return:

- project and dependency SHAs;
- exact test commands and results;
- one-hour/24-hour status;
- unplug and timing status;
- captures committed versus preserved locally;
- GitHub PR/Actions/branch status;
- final allowed claim, limited to a stable server-only standard-frame MS/TP lab
  device at the actually tested baud/topology.

---

