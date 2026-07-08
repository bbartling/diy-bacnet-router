"""Hosted BACnet server (rusty_bacnet.BACnetServer) from objects.csv."""

from __future__ import annotations

import logging
from typing import Any

from app.config import HostedObjectRow, Settings, load_objects_csv

logger = logging.getLogger(__name__)

# BACnet engineering units (ASHRAE 135)
UNITS_MAP = {
    "degreesfahrenheit": 62,
    "degf": 62,
    "percent": 98,
    "pct": 98,
    "milesphour": 72,
    "mph": 72,
    "nounits": 95,
    "status": 95,
    "": 95,
}


def _units_code(units: str) -> int:
    return UNITS_MAP.get(units.replace(" ", "").lower(), 95)


def _parse_float(s: str, default: float = 0.0) -> float:
    try:
        return float(s)
    except (TypeError, ValueError):
        return default


class BacnetServerManager:
    """Owns UDP :47808 and mirrors weather/FDD values via write_property_local.

    Read/write split (prevents a REST-vs-BACnet data race):

    - **Server-owned points** (``Commandable=N``): weather feeds, fault counts,
      status. These are written by this process (weather loop, FDD updates) and
      are the only points the REST API may update.
    - **Commandable points** (``Commandable=Y``): a field or supervisory BACnet
      device may command these at any time. The REST API is **read-only** for
      them so an API write can never race a BACnet write. The API can still
      *observe* whatever a BACnet client wrote via the read endpoints.
    """

    def __init__(self, settings: Settings):
        self.settings = settings
        self._server = None
        self._object_index: dict[tuple[str, int], str] = {}  # (type, inst) -> name

    @staticmethod
    def api_writable(row: HostedObjectRow) -> bool:
        """True if the REST API may write this point (server-owned, Commandable=N).

        Commandable points are BACnet-writable and therefore read-only via the
        API to avoid racing a device that may also be commanding them.
        """
        return not row.commandable

    @property
    def server(self):
        if self._server is None:
            raise RuntimeError("BACnet server not started")
        return self._server

    async def start(self) -> None:
        from rusty_bacnet import BACnetServer

        cfg = self.settings.bacnet_server
        rows = load_objects_csv(self.settings.objects_csv)
        self._server = BACnetServer(
            device_instance=cfg.device_instance,
            device_name=cfg.device_name,
            interface=cfg.interface,
            port=cfg.port,
            broadcast_address=cfg.broadcast,
        )
        for row in rows:
            self._add_object(row)
            key = (row.point_type.upper(), row.instance)
            self._object_index[key] = row.name
        await self._server.start()
        await self._apply_csv_defaults(rows)
        addr = await self._server.local_address()
        logger.info(
            "BACnet server %s (%s) listening on %s",
            cfg.device_instance,
            cfg.device_name,
            addr,
        )

    def _add_object(self, row: HostedObjectRow) -> None:
        s = self._server
        pt = row.point_type.upper()
        inst = row.instance
        name = row.name
        units = _units_code(row.units)

        if pt == "AV":
            s.add_analog_value(inst, name, units=units)
        elif pt == "AI":
            pv = _parse_float(row.default, 0.0)
            s.add_analog_input(inst, name, units=units, present_value=pv)
        elif pt == "BI":
            s.add_binary_input(inst, name)
        elif pt == "BV":
            s.add_binary_value(inst, name)
        elif pt == "CSV":
            s.add_character_string_value(inst, name)
        else:
            raise ValueError(f"Unsupported PointType {row.point_type} for {row.name}")

    async def _apply_csv_defaults(self, rows: list[HostedObjectRow]) -> None:
        for row in rows:
            pt = row.point_type.upper()
            if pt == "AV":
                await self.write_present_value(pt, row.instance, _parse_float(row.default, 0.0))
            elif pt == "CSV" and row.default:
                await self.write_present_value(pt, row.instance, row.default)
            elif pt == "BI" and row.default.lower() == "inactive":
                pass  # BI present-value is read-only on server; skip default write
            elif pt == "BV" and row.default.lower() == "inactive":
                from rusty_bacnet import PropertyIdentifier, PropertyValue

                oid = self._oid("BV", row.instance)
                await self.server.write_property_local(
                    oid,
                    PropertyIdentifier.PRESENT_VALUE,
                    PropertyValue.enumerated(0),
                    None,
                    None,
                )

    async def stop(self) -> None:
        if self._server is not None:
            await self._server.stop()
            self._server = None

    async def list_objects(self) -> list[dict[str, Any]]:
        from rusty_bacnet import ObjectIdentifier, ObjectType, PropertyIdentifier

        out: list[dict[str, Any]] = []
        rows = load_objects_csv(self.settings.objects_csv)
        for row in rows:
            oid = self._oid(row.point_type, row.instance)
            try:
                pv = await self.server.read_property(
                    oid, PropertyIdentifier.PRESENT_VALUE, None
                )
                value = pv.value
                tag = pv.tag
            except Exception as e:
                value = None
                tag = f"error: {e}"
            out.append(
                {
                    "name": row.name,
                    "object_type": row.point_type,
                    "instance": row.instance,
                    "units": row.units,
                    "commandable": bool(row.commandable),
                    "api_writable": self.api_writable(row),
                    "present_value": value,
                    "tag": tag,
                }
            )
        return out

    async def list_commandable(self) -> list[dict[str, Any]]:
        """Commandable hosted points (Commandable=Y): BACnet-writable, API read-only.

        Returns present-values so the API can observe whatever a BACnet client
        last wrote to each point.
        """
        from rusty_bacnet import PropertyIdentifier

        out: list[dict[str, Any]] = []
        rows = load_objects_csv(self.settings.objects_csv)
        for row in rows:
            if not row.commandable:
                continue
            oid = self._oid(row.point_type, row.instance)
            try:
                pv = await self.server.read_property(oid, PropertyIdentifier.PRESENT_VALUE, None)
                value, tag = pv.value, pv.tag
            except Exception as e:
                value, tag = None, f"error: {e}"
            out.append(
                {
                    "name": row.name,
                    "object_type": row.point_type,
                    "instance": row.instance,
                    "commandable": True,
                    "api_writable": False,
                    "present_value": value,
                    "tag": tag,
                }
            )
        return out

    async def update_points(self, updates: dict[str, Any]) -> dict[str, str]:
        """Write present-values on **server-owned** points by name.

        Commandable points are rejected: they are BACnet-writable, so allowing an
        API write would race a field/supervisory device commanding the same slot.
        """
        rows = {r.name: r for r in load_objects_csv(self.settings.objects_csv)}
        result: dict[str, str] = {}
        for name, value in updates.items():
            row = rows.get(name)
            if row is None:
                result[name] = "not found"
                continue
            if not self.api_writable(row):
                result[name] = "rejected: commandable point is BACnet-writable (read-only via API)"
                continue
            pt = row.point_type.upper()
            try:
                if pt in ("BV", "BI"):
                    active = value if isinstance(value, bool) else str(value).strip().lower() in (
                        "1", "true", "active", "on", "yes",
                    )
                    await self.write_binary_active(row.instance, active)
                elif pt == "CSV":
                    await self.write_present_value(pt, row.instance, str(value))
                else:
                    await self.write_present_value(pt, row.instance, float(value))
                result[name] = "updated"
            except Exception as e:
                result[name] = f"error: {e}"
        return result

    def _oid(self, point_type: str, instance: int):
        from rusty_bacnet import ObjectIdentifier, ObjectType

        mapping = {
            "AV": ObjectType.ANALOG_VALUE,
            "AI": ObjectType.ANALOG_INPUT,
            "BI": ObjectType.BINARY_INPUT,
            "BV": ObjectType.BINARY_VALUE,
            "CSV": ObjectType.CHARACTERSTRING_VALUE,
        }
        ot = mapping.get(point_type.upper())
        if ot is None:
            raise ValueError(f"unknown point type {point_type}")
        return ObjectIdentifier(ot, instance)

    async def write_present_value(self, point_type: str, instance: int, value) -> None:
        from rusty_bacnet import PropertyIdentifier, PropertyValue

        oid = self._oid(point_type, instance)
        if isinstance(value, bool):
            pv = PropertyValue.boolean(value)
        elif isinstance(value, int):
            pv = PropertyValue.enumerated(value)
        elif isinstance(value, float):
            pv = PropertyValue.real(value)
        elif isinstance(value, str):
            pv = PropertyValue.character_string(value)
        else:
            pv = PropertyValue.real(float(value))
        await self.server.write_property_local(
            oid, PropertyIdentifier.PRESENT_VALUE, pv, None, None
        )

    async def write_binary_active(self, instance: int, active: bool) -> None:
        from rusty_bacnet import PropertyIdentifier, PropertyValue

        oid = self._oid("BV", instance)
        await self.server.write_property_local(
            oid,
            PropertyIdentifier.PRESENT_VALUE,
            PropertyValue.enumerated(1 if active else 0),
            None,
            None,
        )
