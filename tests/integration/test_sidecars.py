"""Modbus + Haystack integration (docker sidecars optional)."""

import os

import pytest

pytestmark = pytest.mark.integration

try:
    import rusty_modbus  # noqa: F401

    HAS_MODBUS = True
except ImportError:
    HAS_MODBUS = False

try:
    import rusty_haystack  # noqa: F401

    HAS_HAYSTACK = True
except ImportError:
    HAS_HAYSTACK = False


@pytest.mark.asyncio
@pytest.mark.skipif(not HAS_MODBUS, reason="rusty_modbus requires Python 3.14 wheel")
async def test_modbus_read_sim():
    from app.modbus_client import execute_modbus_read

    host = os.environ.get("MODBUS_DEFAULT_HOST", "127.0.0.1")
    port = int(os.environ.get("MODBUS_SIM_PORT", "5502"))
    try:
        result = await execute_modbus_read(
            {
                "host": host,
                "port": port,
                "unit_id": 1,
                "timeout": 3.0,
                "registers": [
                    {"address": 0, "count": 1, "function": "holding", "decode": "uint16"},
                ],
            }
        )
    except Exception as e:
        pytest.skip(f"modbus sim not reachable: {e}")
    assert result["ok"] is True
    assert result["readings"][0]["success"] is True


@pytest.mark.skipif(not HAS_HAYSTACK, reason="rusty_haystack not installed")
def test_haystack_about_demo():
    from app.config import load_settings
    from app.haystack_client import HaystackClientService

    s = load_settings()
    client = HaystackClientService(s.haystack.base_url, s.haystack.username, s.haystack.password)
    try:
        about = client.about()
    except Exception as e:
        pytest.skip(f"haystack demo not reachable: {e}")
    finally:
        client.close()
    assert about is not None
