"""Modbus TCP REST routes."""

from __future__ import annotations

from fastapi import APIRouter, HTTPException

from app.models import ModbusReadRequest
from app.modbus_client import ModbusServiceError, execute_modbus_read

router = APIRouter(prefix="/modbus", tags=["Modbus"])


@router.post("/read")
async def modbus_read(body: ModbusReadRequest):
    try:
        return await execute_modbus_read(body.model_dump())
    except ModbusServiceError as e:
        raise HTTPException(status_code=400, detail=str(e)) from e
    except Exception as e:
        raise HTTPException(status_code=502, detail=f"modbus_error: {e}") from e
