---
title: CI & publishing
nav_order: 11
---

# CI & publishing

GitHub Actions runs unit tests and publishes the Jekyll docs to GitHub Pages.

## Workflows

| Workflow | Purpose |
|----------|---------|
| `.github/workflows/ci.yml` | **Python 3.12** on `ubuntu-latest`: `pip install ".[dev]" rusty-bacnet rusty-haystack`, then `pytest tests/unit`. |
| `.github/workflows/docs-pages.yml` | `bundle install` + `jekyll build` from `docs/`, deploy with **deploy-pages**. |

## Tests

```bash
pip install -e ".[dev]" rusty-bacnet rusty-haystack
pytest tests/unit -q
pytest tests/integration -m integration -q   # needs live devices + :47808 free
```

Unit tests run without hardware. The integration suite talks to real BACnet
devices and the local Modbus/Haystack sidecars, so it is opt-in via the
`integration` marker.

## GitHub Pages (first time)

Repository **Settings → Pages → Build and deployment → Source: GitHub Actions**.

Published site: **`https://bbartling.github.io/diy-bacnet-server/`** — must match **`baseurl`** in `docs/_config.yml`.

## License

MIT License. See `LICENSE` in the repository root.
