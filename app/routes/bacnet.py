"""BACnet server + client REST routes."""

from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request

from app.models import (
    BacnetObjectRef,
    BacnetReadRequest,
    BacnetRpmRequest,
    BacnetWhoisRequest,
    BacnetWriteRequest,
    DeviceInstanceRequest,
    ServerUpdatePointsRequest,
)

router = APIRouter(prefix="/bacnet", tags=["BACnet"])


# ──────────── BACnet client (field bus) ────────────

@router.get("/points", summary="List configured field-device points")
async def list_field_points(request: Request):
    svc = request.app.state.bacnet_client
    return {"ok": True, "points": svc.list_points()}


@router.post("/read", summary="Client read property")
async def bacnet_read(body: BacnetReadRequest, request: Request):
    svc = request.app.state.bacnet_client
    try:
        result = await svc.read_property(
            body.device_instance,
            body.object_type,
            body.object_instance,
            body.property_id,
        )
        return {"ok": True, **result}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/write", summary="Client write property (priority + null release)")
async def bacnet_write(body: BacnetWriteRequest, request: Request):
    svc = request.app.state.bacnet_client
    if not body.approved:
        # Safety gate: unapproved writes never touch the bus — return the dry-run.
        try:
            result = svc.write_dry_run(
                body.device_instance,
                body.object_type,
                body.object_instance,
                body.value,
                body.property_id,
                body.priority,
                body.value_type,
            )
            return {"ok": True, "skipped": "not approved", **result}
        except ValueError as e:
            raise HTTPException(status_code=400, detail=str(e)) from e
    try:
        result = await svc.write_property(
            body.device_instance,
            body.object_type,
            body.object_instance,
            body.value,
            body.property_id,
            body.priority,
            body.value_type,
        )
        return {"ok": True, **result}
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e)) from e
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/write-dry-run", summary="Validate + encode a write without sending it")
async def bacnet_write_dry_run(body: BacnetWriteRequest, request: Request):
    svc = request.app.state.bacnet_client
    try:
        result = svc.write_dry_run(
            body.device_instance,
            body.object_type,
            body.object_instance,
            body.value,
            body.property_id,
            body.priority,
            body.value_type,
        )
        return {"ok": True, **result}
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e)) from e


@router.get("/poll/status", summary="Background poll engine status + last values")
async def bacnet_poll_status(request: Request):
    engine = getattr(request.app.state, "poll_engine", None)
    if engine is None:
        return {"ok": True, "enabled": False, "running": False, "last_values": []}
    return {"ok": True, **engine.status()}


@router.post("/poll/once", summary="Run one poll cycle now (present-value, all points)")
async def bacnet_poll_once(request: Request):
    engine = getattr(request.app.state, "poll_engine", None)
    if engine is None:
        raise HTTPException(status_code=503, detail="poll engine not initialized")
    try:
        result = await engine.poll_once()
        return {"ok": True, **result}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/rpm", summary="Client read multiple properties")
async def bacnet_rpm(body: BacnetRpmRequest, request: Request):
    svc = request.app.state.bacnet_client
    objects = [
        {
            "object_type": o.object_type,
            "object_instance": o.object_instance,
            "properties": [
                {"property_id": p.property_id, "array_index": p.array_index}
                for p in o.properties
            ],
        }
        for o in body.objects
    ]
    try:
        result = await svc.read_property_multiple(body.device_instance, objects)
        return {"ok": True, **result}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/whois", summary="Client Who-Is range")
async def bacnet_whois(body: BacnetWhoisRequest, request: Request):
    svc = request.app.state.bacnet_client
    try:
        devices = await svc.who_is(body.low, body.high)
        return {"ok": True, "count": len(devices), "devices": devices}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/whois-router", summary="Who-Is router-to-network (routed networks)")
async def bacnet_whois_router(request: Request):
    svc = request.app.state.bacnet_client
    try:
        routers = await svc.who_is_router_to_network()
        return {"ok": True, "count": len(routers), "routers": routers}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/discover", summary="Client point discovery (object-list + commandable)")
async def bacnet_discover(body: DeviceInstanceRequest, request: Request):
    svc = request.app.state.bacnet_client
    try:
        result = await svc.point_discovery(body.device_instance)
        return {"ok": True, **result}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/priority-array", summary="Client read priority array (16 slots)")
async def bacnet_priority_array(body: BacnetObjectRef, request: Request):
    svc = request.app.state.bacnet_client
    try:
        result = await svc.read_priority_array(
            body.device_instance, body.object_type, body.object_instance
        )
        return {"ok": True, **result}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/supervisory", summary="Client supervisory logic checks (override audit)")
async def bacnet_supervisory(body: DeviceInstanceRequest, request: Request):
    svc = request.app.state.bacnet_client
    try:
        result = await svc.supervisory_logic_check(body.device_instance)
        return {"ok": True, **result}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


# ──────────── BACnet server (hosted device 599999) ────────────

@router.get("/server/objects", summary="Server read all point values")
async def list_server_objects(request: Request):
    mgr = request.app.state.bacnet_server
    return {"ok": True, "objects": await mgr.list_objects()}


@router.get(
    "/server/commandable",
    summary="Server read commandable points (BACnet-writable, API read-only)",
)
async def list_server_commandable(request: Request):
    mgr = request.app.state.bacnet_server
    return {"ok": True, "objects": await mgr.list_commandable()}


@router.post(
    "/server/update",
    summary="Server update server-owned point values (commandable points are rejected)",
)
async def update_server_points(body: ServerUpdatePointsRequest, request: Request):
    mgr = request.app.state.bacnet_server
    try:
        result = await mgr.update_points(body.updates)
        return {"ok": True, "result": result}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e)) from e
