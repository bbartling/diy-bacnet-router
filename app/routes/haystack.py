"""Haystack read-only REST routes."""

from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request

from app.haystack_client import HaystackNotAllowedError
from app.models import HaystackHisReadRequest, HaystackNavRequest, HaystackReadRequest

router = APIRouter(prefix="/haystack", tags=["Haystack"])


def _client(request: Request):
    return request.app.state.haystack_client


@router.get("/about")
async def haystack_about(request: Request):
    try:
        return {"ok": True, "about": _client(request).about()}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/read")
async def haystack_read(body: HaystackReadRequest, request: Request):
    try:
        grid = _client(request).read(body.filter)
        return {"ok": True, "grid": _grid_to_dict(grid)}
    except HaystackNotAllowedError as e:
        raise HTTPException(status_code=403, detail=str(e)) from e
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/nav")
async def haystack_nav(body: HaystackNavRequest, request: Request):
    try:
        grid = _client(request).nav(body.nav_id)
        return {"ok": True, "grid": _grid_to_dict(grid)}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


@router.post("/his-read")
async def haystack_his_read(body: HaystackHisReadRequest, request: Request):
    try:
        result = _client(request).his_read(body.ids, body.range_start, body.range_end)
        return {"ok": True, "result": _grid_to_dict(result) if hasattr(result, "to_dict") else result}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


def _grid_to_dict(grid) -> dict:
    if hasattr(grid, "to_dict"):
        return grid.to_dict()
    if hasattr(grid, "__dict__"):
        return str(grid)
    return {"repr": repr(grid)}
