---
title: Client BACnet
nav_order: 7
---

# Client BACnet (external devices)

The gateway acts as a BACnet **client** on the LAN: Who-Is / I-Am, ReadProperty,
WriteProperty, ReadPropertyMultiple, point discovery, priority-array reads, and
supervisory override audits. Field devices are declared in
`config/field_devices.toml`; every operation is a REST endpoint.

The client sends Who-Is on `:47808` (so it hears broadcast I-Am) and performs
unicast reads on ephemeral ports. Routed MS/TP devices are reached via their
router automatically.

## Endpoint matrix

| Feature | Endpoint | Notes |
|---------|----------|--------|
| Who-Is range scan | `POST /bacnet/whois` | Returns device list (address, vendor id, source network, max APDU). |
| Who-Is router-to-network | `POST /bacnet/whois-router` | Reachable routers and the networks behind them. |
| Read one property | `POST /bacnet/read` | Returns `{ tag, value, ... }`. |
| Write property (priority / release) | `POST /bacnet/write` | JSON **`null`** or **`"null"`** + **priority** to release a slot. |
| Read Property Multiple | `POST /bacnet/rpm` | Chunked internally (**25** object/property pairs per chunk). |
| Point discovery | `POST /bacnet/discover` | Object-list walk + object names + commandable detection. |
| Priority array (one object) | `POST /bacnet/priority-array` | All 16 slots as `{ priority_level, type, value }`. |
| Supervisory override audit | `POST /bacnet/supervisory` or `/api/bacnet/supervisory` | Commandable points and active override slots. |

## `POST /bacnet/whois`

- **Body:** `{ "low": 0, "high": 4194303 }` (both optional; omit for a global Who-Is).
- **Returns:** `{ "count": N, "devices": [ { "device_instance", "address", "vendor_id", "source_network", "max_apdu" }, ... ] }`.

## `POST /bacnet/read`

- **Body:** `{ "device_instance": 5007, "object_type": "analog-input", "object_instance": 1173, "property_id": "present-value" }`
- **Returns:** `{ "tag", "value", "device_instance", ... }`.

## `POST /bacnet/write`

- **Body:** `{ "device_instance", "object_type", "object_instance", "value", "property_id", "priority": <1-16>, "value_type": <optional> }`
- **Release:** `"value": null` (or `"value": "null"`) **with** a `priority` to relinquish that slot.
- **`value_type`:** force encoding (`real`, `double`, `unsigned`, `signed`, `enumerated`, `boolean`, `character_string`, `null`). Default: float→real, bool→enumerated, str→character_string.
- **Returns:** `{ "status": "success", "released": <bool>, "priority": <n> }`.

## `POST /bacnet/rpm`

- **Body:** `{ "device_instance", "objects": [ { "object_type", "object_instance", "properties": [ { "property_id", "array_index": <optional> } ] } ] }`
- **Returns:** `{ "results": [ { "object_identifier", "property_identifier", "property_array_index", "value" }, ... ] }` (value may be an error string per property).

## `POST /bacnet/discover`

- **Body:** `{ "device_instance": 3456790 }`
- **Returns:** `{ "device_address", "device_instance", "objects": [ { "object_identifier", "name", "commandable" }, ... ] }`.

## `POST /bacnet/priority-array`

- **Body:** `{ "device_instance", "object_type", "object_instance" }`
- **Returns:** `{ "object_identifier", "priority_array": [ { "priority_level", "type", "value" }, ... ] }` (16 slots).

## `POST /bacnet/supervisory`

- **Body:** `{ "device_instance": 3456790 }`
- **Returns:** `device_id`, `address`, `points`, `points_with_overrides`, and a `summary` of commandable/override counts.

## `POST /bacnet/whois-router`

- **Body:** none.
- **Returns:** `{ "count": N, "routers": [ { "source", "networks": [ ... ] }, ... ] }`.
