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
        return {"ok": True, "about": _grid_to_dict(_client(request).about())}
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
        return {"ok": True, "result": {pid: _grid_to_dict(g) for pid, g in result.items()}}
    except Exception as e:
        raise HTTPException(status_code=502, detail=str(e)) from e


def _coerce(value):
    """Coerce a Haystack cell value to a JSON-safe primitive.

    Markers/Numbers/Refs and other rusty_haystack types are not JSON
    serializable, so anything that is not a plain primitive is rendered to its
    string form (readable and round-trip-safe for a read-only gateway).
    """
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if isinstance(value, (list, tuple)):
        return [_coerce(v) for v in value]
    if isinstance(value, dict):
        return {str(k): _coerce(v) for k, v in value.items()}
    return str(value)


def _grid_to_dict(grid) -> dict:
    """Serialize a rusty_haystack HGrid into a JSON-safe {cols, rows} dict."""
    # HGrid: extract column names + coerced row dicts.
    if hasattr(grid, "rows") and hasattr(grid, "col_names"):
        try:
            cols = grid.col_names() if callable(grid.col_names) else grid.col_names
            raw_rows = grid.rows() if callable(grid.rows) else grid.rows
            rows = []
            for r in raw_rows:
                d = r.to_dict() if hasattr(r, "to_dict") else dict(r)
                rows.append({str(k): _coerce(v) for k, v in d.items()})
            return {"cols": list(cols), "rows": rows, "count": len(rows)}
        except Exception:
            pass
    if hasattr(grid, "to_dict"):
        try:
            return {str(k): _coerce(v) for k, v in grid.to_dict().items()}
        except Exception:
            pass
    return {"repr": repr(grid)}
