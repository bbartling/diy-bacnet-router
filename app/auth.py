"""Bearer API-key auth middleware."""

from __future__ import annotations

import os
import secrets
from typing import Callable

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import JSONResponse
from starlette.types import ASGIApp


def api_key() -> str:
    """Configured API key. OPENFDD_FIELDBUS_API_KEY wins over RUSTY_GATEWAY_API_KEY."""
    return (
        os.environ.get("OPENFDD_FIELDBUS_API_KEY")
        or os.environ.get("RUSTY_GATEWAY_API_KEY")
        or ""
    ).strip()


def auth_path_exempt(path: str) -> bool:
    # Liveness endpoints (native + Open-FDD compat alias) are unauthenticated.
    if path in ("/", "/health", "/api/health"):
        return True
    if path in ("/docs", "/redoc", "/openapi.json"):
        return True
    if path.startswith("/docs/") or path.startswith("/redoc/"):
        return True
    return False


class GatewayAuthMiddleware(BaseHTTPMiddleware):
    """Require Authorization: Bearer <RUSTY_GATEWAY_API_KEY> except exempt paths."""

    def __init__(self, app: ASGIApp, api_key: str):
        super().__init__(app)
        self._api_key = api_key.strip()
        if not self._api_key:
            raise ValueError("GatewayAuthMiddleware requires non-empty api_key")

    async def dispatch(self, request: Request, call_next: Callable):
        if auth_path_exempt(request.url.path):
            return await call_next(request)
        auth = request.headers.get("Authorization") or ""
        if not auth.startswith("Bearer "):
            return JSONResponse(
                {"detail": "Missing or invalid Authorization header"},
                status_code=401,
            )
        token = auth[7:].strip()
        if not secrets.compare_digest(token, self._api_key):
            return JSONResponse({"detail": "Invalid API key"}, status_code=403)
        return await call_next(request)


def install_openapi_bearer(app) -> None:
    """Inject BearerAuth into OpenAPI schema for Swagger Authorize."""

    def _custom_openapi():
        if app.openapi_schema:
            return app.openapi_schema
        from fastapi.openapi.utils import get_openapi

        schema = get_openapi(
            title=app.title,
            version=app.version,
            openapi_version=app.openapi_version,
            description=app.description,
            routes=app.routes,
            tags=getattr(app, "openapi_tags", None),
            servers=getattr(app, "servers", None),
        )
        schema["components"] = schema.get("components") or {}
        schema["components"]["securitySchemes"] = {
            "BearerAuth": {
                "type": "http",
                "scheme": "bearer",
                "bearerFormat": "API Key",
                "description": (
                    "When RUSTY_GATEWAY_API_KEY is set, use that value. "
                    "Send `Authorization: Bearer <key>` on protected routes."
                ),
            }
        }
        schema["security"] = [{"BearerAuth": []}]
        app.openapi_schema = schema
        return app.openapi_schema

    app.openapi = _custom_openapi


def install_openapi_servers_url(app) -> None:
    base = (os.environ.get("RUSTY_GATEWAY_SWAGGER_SERVERS_URL") or "").strip()
    if not base:
        return
    _prev = app.openapi

    def _combined():
        if app.openapi_schema is not None:
            return app.openapi_schema
        schema = _prev()
        schema["servers"] = [{"url": base}]
        return schema

    app.openapi = _combined


def install_auth_if_configured(app) -> None:
    key = api_key()
    if not key:
        return
    app.add_middleware(GatewayAuthMiddleware, api_key=key)
