"""Open-FDD compatibility aliases.

The field-bus sidecar exposes its native routes at the root (/bacnet/*,
/modbus/*, /haystack/*, /weather) and mirrors them under /api/* so an Open-FDD
deployment (or a thin proxy in the FDD app) can reach them with the /api prefix
it uses everywhere else. This module adds the few endpoints whose Open-FDD names
differ from the native ones.
"""

from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request

from app.config import git_sha
from app.models import DeviceInstanceRequest

router = APIRouter(prefix="/api", tags=["Open-FDD compat"])


@router.get("/health", summary="Liveness (Open-FDD shape)")
async def api_health(request: Request):
    settings = getattr(request.app.state, "settings", None)
    engine = getattr(request.app.state, "poll_engine", None)
    return {
        "ok": True,
        "service": "openfdd-fieldbus",
        "version": request.app.version,
        "git_sha": git_sha(),
        "poll_running": bool(engine and engine.status()["running"]),
        "bacnet_server_instance": settings.bacnet_server.device_instance if settings else None,
    }


@router.post("/bacnet/point-discovery", summary="Point discovery (Open-FDD alias of /discover)")
async def api_point_discovery(body: DeviceInstanceRequest, request: Request):
    svc = request.app.state.bacnet_client
    try:
        result = await svc.point_discovery(body.device_instance)
        return {"ok": True, **result}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.get("/bacnet/server/points", summary="Hosted server points (Open-FDD alias of /server/objects)")
async def api_server_points(request: Request):
    mgr = request.app.state.bacnet_server
    return {"ok": True, "objects": await mgr.list_objects()}
