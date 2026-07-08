#!/usr/bin/env bash
# Free UDP :47808 before starting the gateway BACnet server.
set -euo pipefail

echo "=== preflight: free BACnet UDP :47808 ==="

for c in openfdd-bridge openfdd-commission openfdd-haystack-gateway; do
  if docker ps -q -f name="^${c}$" 2>/dev/null | grep -q .; then
    echo "Stopping docker container: $c"
    docker stop "$c" 2>/dev/null || true
  fi
done

# Kill processes holding 47808/udp
if command -v fuser >/dev/null 2>&1; then
  if fuser 47808/udp 2>/dev/null; then
    echo "Releasing 47808/udp via fuser -k"
    fuser -k 47808/udp 2>/dev/null || true
    sleep 1
  fi
fi

# Kill known bench BACnet demo binaries by name
for pat in mini-device-revisited mock_scan bacnet_app openfdd-bacnet openfdd_bacnet bacnet-probe; do
  pids=$(pgrep -f "$pat" 2>/dev/null || true)
  if [ -n "$pids" ]; then
    echo "Stopping processes matching $pat: $pids"
    kill -TERM $pids 2>/dev/null || true
  fi
done
sleep 1

if ss -lun 2>/dev/null | grep -q ':47808'; then
  echo "WARNING: something still bound to :47808"
  ss -lun | grep 47808 || true
else
  echo "OK: UDP :47808 is free"
fi
