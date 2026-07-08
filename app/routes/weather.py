"""Weather REST routes."""

from __future__ import annotations

from fastapi import APIRouter, Request

router = APIRouter(prefix="/weather", tags=["Weather"])


@router.get("")
async def get_weather(request: Request):
    wx = request.app.state.weather
    return wx.to_dict()


@router.post("/refresh")
async def refresh_weather(request: Request):
    wx = request.app.state.weather
    reading = await wx.refresh_now()
    return {"ok": True, **wx.to_dict()}
