"""Open-Meteo weather fetch + BACnet mirror."""

from __future__ import annotations

import asyncio
import logging
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any, Optional
from urllib.parse import quote

import httpx

from app.bacnet_server import BacnetServerManager
from app.config import Settings

logger = logging.getLogger(__name__)


def dewpoint_f_from_db_rh(temp_f: float, rh_percent: float) -> float:
    """Magnus-Tetens dewpoint (°F) from dry-bulb (°F) and RH (%)."""
    t_c = (temp_f - 32.0) * 5.0 / 9.0
    rh = max(0.1, min(100.0, rh_percent))
    a, b = 17.27, 237.7
    alpha = (a * t_c) / (b + t_c) + __import__("math").log(rh / 100.0)
    dp_c = (b * alpha) / (a - alpha)
    return dp_c * 9.0 / 5.0 + 32.0


@dataclass
class WeatherReading:
    temp_f: float
    humidity: float
    wind_mph: float
    dewpoint_f: float
    location: str
    from_api: bool
    reason: str = ""
    updated_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())


class WeatherService:
    """Poll Open-Meteo every interval_secs; mirror to BACnet AVs + REST cache."""

    # Object instances from objects.csv
    AV_TEMP = 1
    AV_RH = 2
    AV_WIND = 3
    AV_DP = 4
    CSV_LOC = 5
    BI_APP_FAULT = 1  # hosted as BV (binary-value); BI is read-only in rusty_bacnet server

    def __init__(self, settings: Settings, bacnet: BacnetServerManager):
        self.settings = settings
        self.bacnet = bacnet
        self._cache: Optional[WeatherReading] = None
        self._task: Optional[asyncio.Task] = None
        self._stop = asyncio.Event()

    @property
    def cache(self) -> Optional[WeatherReading]:
        return self._cache

    def to_dict(self) -> dict[str, Any]:
        if self._cache is None:
            return {"ok": False, "reason": "no data yet"}
        c = self._cache
        return {
            "ok": True,
            "temp_f": c.temp_f,
            "humidity": c.humidity,
            "wind_mph": c.wind_mph,
            "dewpoint_f": c.dewpoint_f,
            "location": c.location,
            "from_api": c.from_api,
            "reason": c.reason,
            "updated_at": c.updated_at,
        }

    async def start(self) -> None:
        cfg = self.settings.weather
        # APP-FAULT active until first successful API fetch
        await self.bacnet.write_binary_active(self.BI_APP_FAULT, active=True)
        self._cache = self._fallback("startup")
        await self._mirror_to_bacnet(self._cache)
        self._stop.clear()
        self._task = asyncio.create_task(self._loop())

    async def stop(self) -> None:
        self._stop.set()
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
            self._task = None

    async def refresh_now(self) -> WeatherReading:
        reading = await self._poll_once()
        self._cache = reading
        await self._mirror_to_bacnet(reading)
        return reading

    def _fallback(self, reason: str) -> WeatherReading:
        cfg = self.settings.weather
        dp = dewpoint_f_from_db_rh(cfg.fallback_temp_f, cfg.fallback_humidity)
        return WeatherReading(
            temp_f=cfg.fallback_temp_f,
            humidity=cfg.fallback_humidity,
            wind_mph=cfg.fallback_wind_mph,
            dewpoint_f=dp,
            location=f"{cfg.city} (fallback)",
            from_api=False,
            reason=reason,
        )

    async def _loop(self) -> None:
        cfg = self.settings.weather
        interval = max(60, cfg.interval_secs)
        while not self._stop.is_set():
            try:
                reading = await self._poll_once()
                self._cache = reading
                await self._mirror_to_bacnet(reading)
            except Exception as e:
                logger.warning("weather poll failed: %s", e)
                self._cache = self._fallback(str(e))
                await self._mirror_to_bacnet(self._cache)
            try:
                await asyncio.wait_for(self._stop.wait(), timeout=interval)
            except asyncio.TimeoutError:
                pass

    async def _mirror_to_bacnet(self, r: WeatherReading) -> None:
        await self.bacnet.write_present_value("AV", self.AV_TEMP, r.temp_f)
        await self.bacnet.write_present_value("AV", self.AV_RH, r.humidity)
        await self.bacnet.write_present_value("AV", self.AV_WIND, r.wind_mph)
        await self.bacnet.write_present_value("AV", self.AV_DP, r.dewpoint_f)
        await self.bacnet.write_present_value("CSV", self.CSV_LOC, r.location)
        await self.bacnet.write_binary_active(self.BI_APP_FAULT, active=not r.from_api)

    async def _poll_once(self) -> WeatherReading:
        cfg = self.settings.weather
        timeout = cfg.http_timeout_secs
        async with httpx.AsyncClient(timeout=timeout) as client:
            loc = await self._geocode_city(client, cfg.city)
            weather = await self._fetch_current(client, loc)
            cur = weather["current"]
            temp_f = float(cur["temperature_2m"])
            humidity = float(cur.get("relative_humidity_2m") or cfg.fallback_humidity)
            wind = float(cur.get("wind_speed_10m") or cfg.fallback_wind_mph)
            dp = dewpoint_f_from_db_rh(temp_f, humidity) if humidity > 0 else float(
                cur.get("dew_point_2m") or dewpoint_f_from_db_rh(temp_f, cfg.fallback_humidity)
            )
            label = f"{loc.get('name', cfg.city)}, {loc.get('admin1', '')} {loc.get('country', '')}".strip()
            return WeatherReading(
                temp_f=temp_f,
                humidity=humidity,
                wind_mph=wind,
                dewpoint_f=dp,
                location=label,
                from_api=True,
                reason="ok",
            )

    async def _geocode_search(self, client: httpx.AsyncClient, name: str, count: int) -> list[dict]:
        url = (
            f"https://geocoding-api.open-meteo.com/v1/search"
            f"?name={quote(name.strip())}&count={count}&language=en&format=json"
        )
        r = await client.get(url)
        r.raise_for_status()
        return r.json().get("results") or []

    async def _geocode_city(self, client: httpx.AsyncClient, city: str) -> dict:
        city = city.strip()
        results = await self._geocode_search(client, city, 1)
        if results:
            return results[0]
        parts = city.replace(",", " ").split()
        if len(parts) >= 2:
            candidates = await self._geocode_search(client, parts[0], 10)
            hint = " ".join(parts[1:]).lower()
            for c in candidates:
                admin = (c.get("admin1") or "").lower()
                if admin and (admin == hint or hint in admin or admin in hint):
                    return c
            if candidates:
                return candidates[0]
        raise RuntimeError(f"no geocode result for city '{city}'")

    async def _fetch_current(self, client: httpx.AsyncClient, loc: dict) -> dict:
        tz = loc.get("timezone") or "auto"
        url = (
            f"https://api.open-meteo.com/v1/forecast"
            f"?latitude={loc['latitude']}&longitude={loc['longitude']}"
            f"&current=temperature_2m,relative_humidity_2m,wind_speed_10m,dew_point_2m"
            f"&temperature_unit=fahrenheit&wind_speed_unit=mph&timezone={quote(tz)}"
        )
        r = await client.get(url)
        r.raise_for_status()
        return r.json()
