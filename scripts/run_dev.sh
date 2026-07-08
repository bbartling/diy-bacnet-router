#!/usr/bin/env bash
# Local dev launcher (Python 3.12+ with pip rusty-bacnet / rusty-haystack).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

"${ROOT}/scripts/preflight_free_47808.sh"

if [ ! -d .venv ]; then
  python3 -m venv .venv
fi
# shellcheck disable=SC1091
source .venv/bin/activate
pip install -q -U pip
pip install -q -e ".[dev]" rusty-bacnet rusty-haystack 2>/dev/null || pip install -q -e ".[dev]"

if [ -z "${RUSTY_GATEWAY_API_KEY:-}" ]; then
  export RUSTY_GATEWAY_API_KEY="$(openssl rand -hex 24 2>/dev/null || python3 -c 'import secrets; print(secrets.token_hex(24))')"
  echo "RUSTY_GATEWAY_API_KEY generated (save for Swagger Authorize)"
fi

export RUSTY_GATEWAY_BIND="${RUSTY_GATEWAY_BIND:-192.168.204.55}"
export RUSTY_GATEWAY_OPENAPI=1

echo "Starting gateway on http://0.0.0.0:8080 (Swagger /docs)"
exec python -m app.main
