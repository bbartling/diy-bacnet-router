"""Integration tests — require bench BACnet devices or docker sidecars."""

import os

import pytest
from httpx import ASGITransport, AsyncClient

pytestmark = pytest.mark.integration

BENCH_BIND = os.environ.get("RUSTY_GATEWAY_BIND", "192.168.204.55")
SKIP_NO_RUSTY = pytest.importorskip("rusty_bacnet", reason="rusty_bacnet not installed")


@pytest.fixture
async def live_app():
    """Start app lifespan without blocking forever — use TestClient pattern."""
    from app.main import create_app

    app = create_app()
    async with app.router.lifespan_context(app):
        yield app


@pytest.mark.asyncio
async def test_health(live_app):
    transport = ASGITransport(app=live_app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        r = await client.get("/health")
        assert r.status_code == 200
        assert r.json()["ok"] is True


@pytest.mark.asyncio
async def test_server_objects_include_weather(live_app):
    transport = ASGITransport(app=live_app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        r = await client.get("/bacnet/server/objects")
        assert r.status_code == 200
        names = {o["name"] for o in r.json()["objects"]}
        assert "OA-WEATHER-T" in names


@pytest.mark.asyncio
async def test_weather_refresh(live_app):
    transport = ASGITransport(app=live_app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        r = await client.post("/weather/refresh")
        assert r.status_code == 200
        body = r.json()
        assert body.get("ok") is True
        assert "temp_f" in body


@pytest.mark.asyncio
async def test_bacnet_read_3456789(live_app):
    transport = ASGITransport(app=live_app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        r = await client.post(
            "/bacnet/read",
            json={
                "device_instance": 3456789,
                "object_type": "analog-input",
                "object_instance": 2,
                "property_id": "present-value",
            },
        )
        if r.status_code != 200:
            pytest.skip(f"bench device unavailable: {r.text}")
        val = r.json().get("value")
        assert val is not None


@pytest.mark.asyncio
async def test_bacnet_read_5007_routed(live_app):
    transport = ASGITransport(app=live_app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        r = await client.post(
            "/bacnet/read",
            json={
                "device_instance": 5007,
                "object_type": "analog-input",
                "object_instance": 1173,
                "property_id": "present-value",
            },
        )
        if r.status_code != 200:
            pytest.skip(f"MSTP 5007 unavailable: {r.text}")
        val = r.json().get("value")
        assert isinstance(val, (int, float))


@pytest.mark.asyncio
async def test_bacnet_whois_finds_devices(live_app):
    transport = ASGITransport(app=live_app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        r = await client.post("/bacnet/whois", json={"low": 0, "high": 4194303})
        assert r.status_code == 200
        devices = r.json().get("devices", [])
        if not devices:
            pytest.skip("no devices on wire (who-is empty)")
        insts = {d["device_instance"] for d in devices}
        assert len(insts) >= 1


@pytest.mark.asyncio
async def test_bacnet_rpm_3456790(live_app):
    transport = ASGITransport(app=live_app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        r = await client.post(
            "/bacnet/rpm",
            json={
                "device_instance": 3456790,
                "objects": [
                    {
                        "object_type": "analog-input",
                        "object_instance": 1,
                        "properties": [
                            {"property_id": "present-value"},
                            {"property_id": "object-name"},
                        ],
                    }
                ],
            },
        )
        if r.status_code != 200:
            pytest.skip(f"3456790 unavailable: {r.text}")
        assert "results" in r.json()
