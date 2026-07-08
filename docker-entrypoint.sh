#!/usr/bin/env sh
set -e
if [ "$#" -gt 0 ]; then
  exec "$@"
fi
/scripts/preflight_free_47808.sh || true
exec python -m app.main
