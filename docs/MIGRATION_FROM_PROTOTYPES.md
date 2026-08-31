# Migration from the prototypes

## Keep Vibe13 as evidence

Do not move or delete `vibe_code_apps_13`. It contains raw serial and MS/TP
device tests that should become reusable fixtures and hardware procedures.

Use `docs/agent/VIBE13_CLOSEOUT_PROMPT.md` for the small remaining soak,
unplug and timing closeout; do not continue its feature surface.

Promote selectively:

- baud/MAC/Max_Master validation cases;
- Clause 9 CRC and frame vectors that are license-compatible;
- by-id serial inventory and passive gate logic;
- JSON hardware evidence schema;
- FEC read and trunk-survival procedures;
- Linux timing baseline scripts.

Do not promote:

- Phase 2 mini-device objects into the router;
- historical captures as current appliance passes;
- B/IP application-device code as an NPDU router;
- workaround copies of rusty-bacnet internals.

## Repurpose the previous DSM repository

The previous DSM functionality has moved to Open-FDD. This replacement keeps
only reusable engineering ideas: Axum lifecycle, JSON contracts, static React
serving, strict config parsing and API tests. Weather, Modbus, Haystack, FDD,
point polling and hosted BACnet application objects are intentionally absent.

## Open-FDD CI lessons

Reuse its multi-architecture discipline, immutable version tags, buildx/QEMU
setup, artifact verification and branch hygiene. A Buildroot disk image is not a
multi-architecture OCI image, so this repository uses one Buildroot matrix job
per board rather than copying the container-publish action verbatim.
