"""BACnet field-bus client (rusty_bacnet) — 1:1 port of diy-bacnet-server client_utils.

Every operation the bacpypes3 ``client_utils.py`` exposed is reproduced here on
the Rust ``rusty_bacnet`` stack:

- read_property            (bacnet_read)
- write_property           (bacnet_write, incl. Null release @ priority)
- read_property_multiple   (bacnet_rpm / bacnet_rpm_chunked)
- who_is                   (perform_who_is)
- point_discovery          (object-list walk + commandable detection)
- read_priority_array      (read_point_priority_arr, all 16 slots)
- supervisory_logic_check  (override audit across commandable points)
- who_is_router_to_network (routed-network discovery)
"""

from __future__ import annotations

import asyncio
import logging
from typing import Any

from app.config import FieldDevice, Settings, load_field_devices

logger = logging.getLogger(__name__)

# RPM chunk size — stay under typical APDU/MTU limits (mirrors diy RPM_CHUNK_SIZE).
RPM_CHUNK_SIZE = 25

OBJECT_TYPE_MAP = {
    "analog-input": "ANALOG_INPUT",
    "analog-output": "ANALOG_OUTPUT",
    "analog-value": "ANALOG_VALUE",
    "binary-input": "BINARY_INPUT",
    "binary-output": "BINARY_OUTPUT",
    "binary-value": "BINARY_VALUE",
    "device": "DEVICE",
    "multi-state-input": "MULTI_STATE_INPUT",
    "multi-state-output": "MULTI_STATE_OUTPUT",
    "multi-state-value": "MULTI_STATE_VALUE",
    "integer-value": "INTEGER_VALUE",
    "large-analog-value": "LARGE_ANALOG_VALUE",
    "positive-integer-value": "POSITIVE_INTEGER_VALUE",
    "characterstring-value": "CHARACTERSTRING_VALUE",
    "character-string-value": "CHARACTERSTRING_VALUE",
    "schedule": "SCHEDULE",
    "calendar": "CALENDAR",
    "trend-log": "TREND_LOG",
    "loop": "LOOP",
}

# Canonical raw-int -> ASHRAE hyphenated name for object-list decoding.
RAW_TO_NAME = {
    0: "analog-input",
    1: "analog-output",
    2: "analog-value",
    3: "binary-input",
    4: "binary-output",
    5: "binary-value",
    6: "calendar",
    7: "command",
    8: "device",
    9: "event-enrollment",
    10: "file",
    11: "group",
    12: "loop",
    13: "multi-state-input",
    14: "multi-state-output",
    15: "notification-class",
    16: "program",
    17: "schedule",
    18: "averaging",
    19: "multi-state-value",
    20: "trend-log",
    21: "life-safety-point",
    22: "life-safety-zone",
    23: "accumulator",
    24: "pulse-converter",
    25: "event-log",
    26: "global-group",
    27: "trend-log-multiple",
    28: "load-control",
    29: "structured-view",
    30: "access-door",
    31: "timer",
    45: "characterstring-value",
    48: "integer-value",
    49: "large-analog-value",
    51: "positive-integer-value",
    56: "network-port",
}

# Object types that carry a priority-array (commandable).
COMMANDABLE_TYPES = {
    "analog-output",
    "analog-value",
    "binary-output",
    "binary-value",
    "multi-state-output",
    "multi-state-value",
    "integer-value",
    "large-analog-value",
    "positive-integer-value",
}

# BACnet unknown-property error (class=property=2, code=32) — used to detect
# whether an object supports priority-array (i.e. is commandable).
_ERR_UNKNOWN_PROPERTY = (2, 32)


def _object_type(name: str):
    from rusty_bacnet import ObjectType

    key = name.strip().lower()
    raw = OBJECT_TYPE_MAP.get(key, name.upper().replace("-", "_"))
    ot = getattr(ObjectType, raw, None)
    if ot is not None:
        return ot
    if name.strip().isdigit():
        return ObjectType.from_raw(int(name.strip()))
    return None


def _object_type_name(ot) -> str:
    """ObjectType -> canonical hyphenated name (e.g. 'analog-input')."""
    try:
        raw = ot.to_raw()
        if raw in RAW_TO_NAME:
            return RAW_TO_NAME[raw]
    except Exception:
        pass
    s = str(ot)
    if "." in s:
        s = s.split(".")[-1]
    return s.strip().lower().replace("_", "-")


def _property_id(name: str):
    from rusty_bacnet import PropertyIdentifier

    key = name.strip().lower().replace("-", "_")
    if hasattr(PropertyIdentifier, key.upper()):
        return getattr(PropertyIdentifier, key.upper())
    return PropertyIdentifier.PRESENT_VALUE


def _normalize_oid(oid) -> str:
    """Canonical 'type,instance' string from an ObjectIdentifier (mirrors diy)."""
    try:
        return f"{_object_type_name(oid.object_type)},{oid.instance}"
    except Exception:
        return str(oid)


def _pv_to_python(pv) -> Any:
    if pv is None:
        return None
    tag = pv.tag
    val = pv.value
    if tag in ("real", "double"):
        return float(val)
    if tag in ("unsigned", "signed", "enumerated"):
        return int(val)
    if tag == "boolean":
        return bool(val)
    if tag == "character_string":
        return str(val)
    if tag == "null":
        return None
    if tag == "object_identifier":
        try:
            return _normalize_oid(val)
        except Exception:
            return str(val)
    return val


def _make_property_value(value: Any, value_type: str | None):
    """Build a rusty_bacnet PropertyValue from a Python value + optional type hint.

    Mirrors bacpypes3's implicit encoding: default numeric -> real, bool ->
    enumerated (0/1), str -> character-string, None -> null.
    """
    from rusty_bacnet import PropertyValue

    vt = (value_type or "").strip().lower()
    if vt == "null" or value is None:
        return PropertyValue.null()
    if vt:
        ctor = {
            "real": lambda v: PropertyValue.real(float(v)),
            "double": lambda v: PropertyValue.double(float(v)),
            "unsigned": lambda v: PropertyValue.unsigned(int(v)),
            "signed": lambda v: PropertyValue.signed(int(v)),
            "enumerated": lambda v: PropertyValue.enumerated(int(v)),
            "boolean": lambda v: PropertyValue.boolean(bool(v)),
            "character_string": lambda v: PropertyValue.character_string(str(v)),
            "character-string": lambda v: PropertyValue.character_string(str(v)),
        }.get(vt)
        if ctor is not None:
            return ctor(value)
    if isinstance(value, bool):
        return PropertyValue.enumerated(1 if value else 0)
    if isinstance(value, float):
        return PropertyValue.real(value)
    if isinstance(value, int):
        return PropertyValue.real(float(value))
    if isinstance(value, str):
        return PropertyValue.character_string(value)
    return PropertyValue.real(float(value))


def _serialize_rpm(results: Any) -> Any:
    """RPM result list -> JSON-safe (diy-style object_identifier/property records)."""
    out: list[dict[str, Any]] = []
    if not isinstance(results, list):
        return out
    for obj in results:
        oid = obj.get("object_id")
        oid_str = _normalize_oid(oid) if oid is not None else None
        for r in obj.get("results", []):
            pid = r.get("property_id")
            err = r.get("error")
            rec = {
                "object_identifier": oid_str,
                "property_identifier": _property_name(pid),
                "property_array_index": r.get("array_index"),
            }
            if err:
                rec["value"] = f"Error: {err}"
            else:
                rec["value"] = _pv_to_python(r.get("value"))
            out.append(rec)
    return out


def _property_name(pid) -> str:
    s = str(pid)
    if "." in s:
        s = s.split(".")[-1]
    return s.strip().lower().replace("_", "-")


def _is_unknown_property(err: Any) -> bool:
    """Detect an unknown-property BACnet error from a rusty_bacnet exception/str."""
    ec = getattr(err, "error_class", None)
    code = getattr(err, "error_code", None)
    if ec is not None and code is not None:
        return (int(ec), int(code)) == _ERR_UNKNOWN_PROPERTY
    s = str(err).lower()
    return "class=2" in s and "code=32" in s


class BacnetClientService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self._field_devices = load_field_devices(settings.field_devices_toml)

    # ---- config-driven point catalog -------------------------------------

    def list_points(self) -> list[dict[str, Any]]:
        out: list[dict[str, Any]] = []
        for d in self._field_devices:
            if not d.enabled:
                continue
            for p in d.points:
                out.append(
                    {
                        "device_name": d.name,
                        "device_instance": d.device_instance,
                        "host": d.host,
                        "port": d.port,
                        "routed": d.is_routed,
                        "mstp_network": d.mstp_network,
                        "object_type": p.object_type,
                        "object_instance": p.object_instance,
                        "point_name": p.point_name,
                    }
                )
        return out

    def _find_device(self, device_instance: int) -> FieldDevice | None:
        for d in self._field_devices:
            if d.device_instance == device_instance:
                return d
        return None

    # ---- client lifecycle helpers ----------------------------------------

    def _bind_port(self, device: FieldDevice | None) -> int:
        """Routed/unknown devices need :47808 to hear I-Am; direct reads use ephemeral."""
        cfg = self.settings.bacnet_client
        if device is None or device.is_routed:
            return cfg.whois_bind_port
        return cfg.read_bind_port

    def _new_client(self, device: FieldDevice | None):
        from rusty_bacnet import BACnetClient

        cfg = self.settings.bacnet_client
        return BACnetClient(
            interface=cfg.interface,
            port=self._bind_port(device),
            broadcast_address=cfg.broadcast,
            apdu_timeout_ms=cfg.apdu_timeout_ms,
        )

    async def _prepare(self, client, device: FieldDevice | None, device_instance: int) -> None:
        """Populate routing so read/write-from-device works (who-is or static add)."""
        cfg = self.settings.bacnet_client
        if device is not None and device.is_routed:
            await client.who_is(device_instance, device_instance)
            await asyncio.sleep(min(3.0, cfg.whois_timeout_secs))
        elif device is not None:
            await client.add_device(device_instance, device.address)
        else:
            await client.who_is(device_instance, device_instance)
            await asyncio.sleep(min(3.0, cfg.whois_timeout_secs))

    # ---- read (bacnet_read) ----------------------------------------------

    async def read_property(
        self,
        device_instance: int,
        object_type: str,
        object_instance: int,
        property_id: str = "present-value",
    ) -> dict[str, Any]:
        from rusty_bacnet import ObjectIdentifier

        device = self._find_device(device_instance)
        ot = _object_type(object_type)
        if ot is None:
            raise ValueError(f"unknown object_type {object_type}")
        oid = ObjectIdentifier(ot, object_instance)
        pid = _property_id(property_id)
        bind_port = self._bind_port(device)

        async with self._new_client(device) as client:
            await self._prepare(client, device, device_instance)
            try:
                ack = await client.read_property_from_device(device_instance, oid, pid, None)
            except Exception:
                if device is not None:
                    ack = await client.read_property(device.address, oid, pid, None)
                else:
                    raise

        return {
            "device_instance": device_instance,
            "object_type": object_type,
            "object_instance": object_instance,
            "property_id": property_id,
            "tag": ack.tag,
            "value": _pv_to_python(ack),
            "client_bind_port": bind_port,
        }

    # ---- write (bacnet_write, incl. Null release) ------------------------

    async def write_property(
        self,
        device_instance: int,
        object_type: str,
        object_instance: int,
        value: Any,
        property_id: str = "present-value",
        priority: int | None = None,
        value_type: str | None = None,
    ) -> dict[str, Any]:
        from rusty_bacnet import ObjectIdentifier

        device = self._find_device(device_instance)
        ot = _object_type(object_type)
        if ot is None:
            raise ValueError(f"unknown object_type {object_type}")
        oid = ObjectIdentifier(ot, object_instance)
        pid = _property_id(property_id)

        is_release = value is None or (isinstance(value, str) and value.strip().lower() == "null")
        if is_release:
            if priority is None or not (1 <= int(priority) <= 16):
                raise ValueError("Null requires a priority (1-16) to release override")
            pv = _make_property_value(None, "null")
            write_priority = int(priority)
        else:
            pv = _make_property_value(value, value_type)
            write_priority = int(priority) if priority is not None else None

        async with self._new_client(device) as client:
            await self._prepare(client, device, device_instance)
            try:
                await client.write_property_to_device(
                    device_instance, oid, pid, pv, write_priority, None
                )
            except Exception:
                if device is not None:
                    await client.write_property(
                        device.address, oid, pid, pv, write_priority, None
                    )
                else:
                    raise

        return {
            "status": "success",
            "device_instance": device_instance,
            "object_type": object_type,
            "object_instance": object_instance,
            "property_id": property_id,
            "released": is_release,
            "priority": write_priority,
        }

    # ---- RPM (bacnet_rpm) -------------------------------------------------

    async def read_property_multiple(
        self, device_instance: int, objects: list[dict]
    ) -> dict[str, Any]:
        from rusty_bacnet import ObjectIdentifier

        device = self._find_device(device_instance)
        specs = []
        for obj in objects:
            ot = _object_type(obj["object_type"])
            oid = ObjectIdentifier(ot, obj["object_instance"])
            props = [
                (_property_id(p["property_id"]), p.get("array_index"))
                for p in obj["properties"]
            ]
            specs.append((oid, props))
        bind_port = self._bind_port(device)

        async with self._new_client(device) as client:
            await self._prepare(client, device, device_instance)
            results = await client.read_property_multiple_from_device(device_instance, specs)

        return {
            "device_instance": device_instance,
            "results": _serialize_rpm(results),
            "client_bind_port": bind_port,
        }

    # ---- who-is (perform_who_is) -----------------------------------------

    async def who_is(self, low: int | None = None, high: int | None = None) -> list[dict[str, Any]]:
        from rusty_bacnet import BACnetClient

        cfg = self.settings.bacnet_client
        async with BACnetClient(
            interface=cfg.interface,
            port=cfg.whois_bind_port,
            broadcast_address=cfg.broadcast,
            apdu_timeout_ms=cfg.apdu_timeout_ms,
        ) as client:
            await client.who_is(low, high)
            await asyncio.sleep(cfg.whois_timeout_secs)
            devices = await client.discovered_devices()

        return [self._device_summary(d) for d in devices]

    @staticmethod
    def _device_summary(d) -> dict[str, Any]:
        mac = d.mac_address
        addr = (
            f"{mac[0]}.{mac[1]}.{mac[2]}.{mac[3]}:{(mac[4] << 8) | mac[5]}"
            if len(mac) == 6
            else mac.hex()
        )
        return {
            "device_instance": d.object_identifier.instance,
            "address": addr,
            "vendor_id": d.vendor_id,
            "source_network": d.source_network,
            "max_apdu": d.max_apdu_length,
        }

    # ---- point discovery (point_discovery) -------------------------------

    async def _read_object_list(self, client, device_instance: int) -> list:
        """Object-list via array-index walk (no segmentation on field devices)."""
        from rusty_bacnet import ObjectIdentifier, ObjectType, PropertyIdentifier

        dev_oid = ObjectIdentifier(ObjectType.DEVICE, device_instance)
        length_pv = await client.read_property_from_device(
            device_instance, dev_oid, PropertyIdentifier.OBJECT_LIST, 0
        )
        length = int(length_pv.value)
        oids: list = []
        # Batch object-list indices via RPM to minimize round-trips.
        for start in range(1, length + 1, RPM_CHUNK_SIZE):
            idxs = list(range(start, min(start + RPM_CHUNK_SIZE, length + 1)))
            specs = [(dev_oid, [(PropertyIdentifier.OBJECT_LIST, i) for i in idxs])]
            try:
                res = await client.read_property_multiple_from_device(device_instance, specs)
                for r in res[0]["results"]:
                    if r.get("error") is None and r.get("value") is not None:
                        oids.append(r["value"].value)
            except Exception as e:
                logger.warning("object-list RPM chunk failed (%s); per-index fallback", e)
                for i in idxs:
                    try:
                        pv = await client.read_property_from_device(
                            device_instance, dev_oid, PropertyIdentifier.OBJECT_LIST, i
                        )
                        oids.append(pv.value)
                    except Exception as e2:
                        logger.warning("object-list[%s] failed: %s", i, e2)
        return oids

    async def point_discovery(self, device_instance: int) -> dict[str, Any]:
        from rusty_bacnet import ObjectIdentifier, PropertyIdentifier

        device = self._find_device(device_instance)
        async with self._new_client(device) as client:
            await self._prepare(client, device, device_instance)
            addr = await self._resolve_address(client, device, device_instance)

            raw_oids = await self._read_object_list(client, device_instance)
            # Exclude the device object itself.
            oids = [o for o in raw_oids if _object_type_name(o.object_type) != "device"]

            # Object-name for each point (chunked RPM).
            name_specs_all = [
                (o, [(PropertyIdentifier.OBJECT_NAME, None)]) for o in oids
            ]
            name_map: dict[str, str] = {}
            for start in range(0, len(name_specs_all), 15):
                chunk = name_specs_all[start : start + 15]
                try:
                    res = await client.read_property_multiple_from_device(device_instance, chunk)
                    for obj in res:
                        oid_str = _normalize_oid(obj["object_id"])
                        for r in obj["results"]:
                            if r.get("error") is None and r.get("value") is not None:
                                name_map[oid_str] = _pv_to_python(r["value"])
                except Exception as e:
                    logger.warning("object-name RPM chunk failed: %s", e)

            # Commandable detection: read priority-array[0]; success => commandable.
            commandable: set[str] = set()
            cand = [o for o in oids if _object_type_name(o.object_type) in COMMANDABLE_TYPES]
            pa_specs = [(o, [(PropertyIdentifier.PRIORITY_ARRAY, 0)]) for o in cand]
            for start in range(0, len(pa_specs), 15):
                chunk = pa_specs[start : start + 15]
                try:
                    res = await client.read_property_multiple_from_device(device_instance, chunk)
                    for obj in res:
                        oid_str = _normalize_oid(obj["object_id"])
                        for r in obj["results"]:
                            if r.get("error") is None and r.get("value") is not None:
                                commandable.add(oid_str)
                except Exception as e:
                    logger.warning("priority-array probe chunk failed: %s", e)

        objects = []
        for o in oids:
            oid_str = _normalize_oid(o)
            objects.append(
                {
                    "object_identifier": oid_str,
                    "name": name_map.get(oid_str, "ERROR - Missing Data"),
                    "commandable": oid_str in commandable,
                }
            )
        return {
            "device_address": addr,
            "device_instance": device_instance,
            "objects": objects,
        }

    async def _resolve_address(self, client, device, device_instance: int) -> str | None:
        if device is not None and not device.is_routed:
            return device.address
        try:
            d = await client.get_device(device_instance)
            if d is not None:
                return self._device_summary(d)["address"]
        except Exception:
            pass
        return device.address if device is not None else None

    # ---- priority array (read_point_priority_arr) ------------------------

    async def read_priority_array(
        self, device_instance: int, object_type: str, object_instance: int
    ) -> dict[str, Any]:
        from rusty_bacnet import ObjectIdentifier, PropertyIdentifier

        device = self._find_device(device_instance)
        ot = _object_type(object_type)
        if ot is None:
            raise ValueError(f"unknown object_type {object_type}")
        oid = ObjectIdentifier(ot, object_instance)

        async with self._new_client(device) as client:
            await self._prepare(client, device, device_instance)
            slots = await self._read_priority_slots(client, device_instance, oid)

        return {
            "device_instance": device_instance,
            "object_identifier": _normalize_oid(oid),
            "priority_array": slots,
        }

    async def _read_priority_slots(self, client, device_instance: int, oid) -> list[dict[str, Any]]:
        """Read all 16 priority-array slots via one batched RPM (per-slot array_index)."""
        from rusty_bacnet import PropertyIdentifier

        specs = [(oid, [(PropertyIdentifier.PRIORITY_ARRAY, i) for i in range(1, 17)])]
        slots: list[dict[str, Any]] = []
        try:
            res = await client.read_property_multiple_from_device(device_instance, specs)
            for r in res[0]["results"]:
                idx = r.get("array_index") or 0
                if r.get("error") is not None:
                    slots.append({"priority_level": idx, "type": "error", "value": str(r["error"])})
                    continue
                pv = r.get("value")
                tag = pv.tag if pv is not None else "null"
                slots.append(
                    {
                        "priority_level": idx,
                        "type": tag,
                        "value": None if tag == "null" else _pv_to_python(pv),
                    }
                )
        except Exception:
            # Per-slot fallback if the device rejects a 16-property RPM.
            for i in range(1, 17):
                try:
                    pv = await client.read_property_from_device(
                        device_instance, oid, PropertyIdentifier.PRIORITY_ARRAY, i
                    )
                    tag = pv.tag
                    slots.append(
                        {
                            "priority_level": i,
                            "type": tag,
                            "value": None if tag == "null" else _pv_to_python(pv),
                        }
                    )
                except Exception as e:
                    slots.append({"priority_level": i, "type": "error", "value": str(e)})
        slots.sort(key=lambda s: s["priority_level"])
        return slots

    # ---- supervisory override audit (supervisory_logic_check) ------------

    async def supervisory_logic_check(self, device_instance: int) -> dict[str, Any]:
        from rusty_bacnet import ObjectIdentifier, PropertyIdentifier

        disc = await self.point_discovery(device_instance)
        device_address = disc["device_address"]
        objects = disc["objects"]

        empty = {
            "device_id": device_instance,
            "address": device_address,
            "points": [],
            "points_with_overrides": [],
            "summary": {
                "total_points": len(objects),
                "with_priority_array": 0,
                "without_priority_array": 0,
                "points_with_override_count": 0,
            },
        }
        if not objects:
            return empty

        commandable = [o for o in objects if o.get("commandable")]
        name_by_oid = {o["object_identifier"]: o["name"] for o in objects}

        device = self._find_device(device_instance)
        points: list[dict[str, Any]] = []
        overrides_by_oid: dict[str, list[dict[str, Any]]] = {}
        with_pa = 0

        async with self._new_client(device) as client:
            await self._prepare(client, device, device_instance)
            for o in commandable:
                oid_str = o["object_identifier"]
                type_name, inst = oid_str.split(",")
                oid = ObjectIdentifier(_object_type(type_name), int(inst))
                slots = await self._read_priority_slots(client, device_instance, oid)
                active = [s for s in slots if s["type"] not in ("null", "error")]
                if slots:
                    with_pa += 1
                for s in active:
                    rec = {
                        "priority_level": s["priority_level"],
                        "object_identifier": oid_str,
                        "object_name": o["name"],
                        "type": s["type"],
                        "value": s["value"],
                    }
                    points.append(rec)
                    overrides_by_oid.setdefault(oid_str, []).append(
                        {"priority_level": s["priority_level"], "type": s["type"], "value": s["value"]}
                    )

        points_with_overrides = []
        for oid_str, slots in overrides_by_oid.items():
            levels = [s["priority_level"] for s in slots]
            points_with_overrides.append(
                {
                    "object_identifier": oid_str,
                    "object_name": name_by_oid.get(oid_str, ""),
                    "override_priority_levels": levels,
                    "has_multiple_overrides": len(levels) > 1,
                    "overrides": slots,
                }
            )

        return {
            "device_id": device_instance,
            "address": device_address,
            "points": points,
            "points_with_overrides": points_with_overrides,
            "summary": {
                "total_points": len(objects),
                "with_priority_array": with_pa,
                "without_priority_array": len(objects) - with_pa,
                "points_with_override_count": len(points_with_overrides),
            },
        }

    # ---- who-is router-to-network (perform_who_is_router_to_network) -----

    async def who_is_router_to_network(self) -> list[dict[str, Any]]:
        """Discover routed networks. rusty_bacnet auto-routes; we derive router
        reachability from I-Am source-network info gathered via a global Who-Is."""
        from rusty_bacnet import BACnetClient

        cfg = self.settings.bacnet_client
        async with BACnetClient(
            interface=cfg.interface,
            port=cfg.whois_bind_port,
            broadcast_address=cfg.broadcast,
            apdu_timeout_ms=cfg.apdu_timeout_ms,
        ) as client:
            await client.who_is(None, None)
            await asyncio.sleep(cfg.whois_timeout_secs)
            devices = await client.discovered_devices()

        by_router: dict[str, set[int]] = {}
        for d in devices:
            net = d.source_network
            if not net:
                continue
            summ = self._device_summary(d)
            # The device's own MAC is the reachable next-hop for its remote net.
            router_addr = summ["address"].split(":")[0]
            by_router.setdefault(router_addr, set()).add(int(net))

        return [
            {"source": router, "networks": sorted(nets)}
            for router, nets in by_router.items()
        ]
