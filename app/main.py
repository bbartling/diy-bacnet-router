"""DIY BACnet Server — FastAPI entrypoint.

BACnet is served and driven by rusty-bacnet, Modbus by rusty-modbus, and
Haystack by rusty-haystack. Python only glues the HTTP/Swagger layer together.
"""

from __future__ import annotations

import logging
import os
from contextlib import asynccontextmanager

import uvicorn
from fastapi import FastAPI

from app.auth import install_auth_if_configured, install_openapi_bearer, install_openapi_servers_url
from app.bacnet_client import BacnetClientService
from app.bacnet_server import BacnetServerManager
from app.config import load_settings
from app.haystack_client import HaystackClientService
from app.routes import bacnet, haystack, modbus, weather
from app.weather import WeatherService

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
logger = logging.getLogger("diy_bacnet_server")


@asynccontextmanager
async def lifespan(app: FastAPI):
    settings = load_settings()
    app.state.settings = settings

    bacnet_srv = BacnetServerManager(settings)
    await bacnet_srv.start()
    app.state.bacnet_server = bacnet_srv

    wx = WeatherService(settings, bacnet_srv)
    await wx.start()
    app.state.weather = wx

    app.state.bacnet_client = BacnetClientService(settings)

    hs = HaystackClientService(
        settings.haystack.base_url,
        settings.haystack.username,
        settings.haystack.password,
    )
    app.state.haystack_client = hs

    logger.info("DIY BACnet Server (Rust) started (HTTP %s:%s)", settings.http_host, settings.http_port)
    yield

    hs.close()
    await wx.stop()
    await bacnet_srv.stop()
    logger.info("DIY BACnet Server (Rust) stopped")


def create_app() -> FastAPI:
    openapi = os.environ.get("RUSTY_GATEWAY_OPENAPI", "1").strip().lower() not in (
        "0",
        "false",
        "no",
    )
    app = FastAPI(
        title="DIY BACnet Server (Rust)",
        version="1.0.0",
        description=(
            "Rust-backed FastAPI + Swagger over rusty-bacnet, rusty-modbus, and rusty-haystack. "
            "Hosts the BACnet server device (599999) with Open-Meteo weather objects and provides "
            "full BACnet client tooling: read, write (priority + null release), RPM, Who-Is, "
            "Who-Is-router-to-network, point discovery, priority-array reads, and supervisory audits."
        ),
        lifespan=lifespan,
        docs_url="/docs" if openapi else None,
        redoc_url="/redoc" if openapi else None,
        openapi_url="/openapi.json" if openapi else None,
    )

    app.include_router(bacnet.router)
    app.include_router(weather.router)
    app.include_router(modbus.router)
    app.include_router(haystack.router)

    @app.get("/")
    async def root():
        return {
            "service": "diy-bacnet-server",
            "backend": "rusty-bacnet / rusty-modbus / rusty-haystack",
            "docs": "/docs",
            "health": "/health",
            "bacnet_server": "/bacnet/server/objects",
            "weather": "/weather",
        }

    @app.get("/health")
    async def health():
        return {"ok": True, "service": "diy-bacnet-server"}

    if openapi:
        install_openapi_bearer(app)
        install_openapi_servers_url(app)
    install_auth_if_configured(app)
    return app


app = create_app()


def main() -> None:
    settings = load_settings()
    uvicorn.run(
        "app.main:app",
        host=settings.http_host,
        port=settings.http_port,
        reload=os.environ.get("RUSTY_GATEWAY_RELOAD", "").strip() in ("1", "true"),
    )


if __name__ == "__main__":
    main()
