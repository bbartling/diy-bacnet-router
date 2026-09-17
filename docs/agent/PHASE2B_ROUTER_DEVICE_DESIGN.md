# Phase 2b — Local router Device (design)

## Goal

Niagara discovers the DIY router itself (Who-Is → I-Am, ReadProperty on Device),
distinct from mini **123102**. Single UDP 47808 + single tty owner — demux
local Device vs forwarded NPDUs; no second BIP listener.

## Tip APIs (`7e0d13a`)

- `bacnet_objects::device::{DeviceConfig, DeviceObject}`
- `bacnet-server` / `rusty-bacnet` server lifecycle examples under
  `crates/rusty-bacnet/src/server/…`

## Appliance plan

1. Add optional `[device]` TOML: `enabled`, `instance`, `object_name`, `vendor_id`,
   `model_name`, `firmware_revision` (unique instance ≠ 0 / ≠ 123102 / ≠ 5007).
2. Depend on `bacnet-objects` + `bacnet-server` at the same git rev as other crates.
3. Spawn Device on the **same** BIP socket used by `BACnetRouter` (or documented
   upstream-supported attachment point) — never bind 47808 twice.
4. Evidence: bacpypes Who-Is sees router instance; RP Object_Name; mini still
   reachable via DNET 2001.

## Status

Config + matrix row opened; full demux implementation follows FEC trunk (phase 2)
so discovery tests include FEC coexistence.
