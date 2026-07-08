---
title: CI & publishing
nav_order: 11
---

# CI & publishing

GitHub Actions runs the unit tests on every push and pull request.

## Workflows

| Workflow | Purpose |
|----------|---------|
| `.github/workflows/ci.yml` | **Python 3.12** on `ubuntu-latest`: `pip install -e ".[dev]" rusty-bacnet rusty-haystack`, then `pytest tests/unit`. |

## Tests

```bash
pip install -e ".[dev]" rusty-bacnet rusty-haystack
pytest tests/unit -q
pytest tests/integration -m integration -q   # needs live devices + :47808 free
```

Unit tests run without hardware. The integration suite talks to real BACnet
devices and the local Modbus/Haystack sidecars, so it is opt-in via the
`integration` marker.

## License

MIT License. See `LICENSE` in the repository root.
