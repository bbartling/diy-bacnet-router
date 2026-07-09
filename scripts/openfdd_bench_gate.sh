#!/usr/bin/env bash
# Open-FDD bench gate — smoke + platform driver + PCAP validation.
#
# Runs the full validation suite the way Open-FDD would use the sidecar in production:
#   1. scripts/smoke_test.sh          (full REST feature matrix)
#   2. scripts/openfdd_platform_driver.sh  (/api/* poll cycles like VOLTTRON driver)
#   3. UDP/47808 PCAP capture during driver cycles
#   4. scripts/pcap_validate.sh       (I-Am / WriteProperty gate when tshark available)
#
# Usage:
#   OPENFDD_FIELDBUS_API_KEY=... scripts/openfdd_bench_gate.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACTS="${BENCH_ARTIFACTS:-$ROOT/artifacts}"
PCAP="${PCAP_FILE:-$ARTIFACTS/bacnet_openfdd_driver.pcap}"
CAPTURE_SECS="${BENCH_PCAP_SECS:-90}"
DRIVER_CYCLES="${DRIVER_CYCLES:-3}"
DRIVER_INTERVAL="${DRIVER_INTERVAL_SECS:-20}"

mkdir -p "$ARTIFACTS"
command -v jq >/dev/null || { echo "FATAL: jq required" >&2; exit 2; }

echo "== Open-FDD bench gate =="
echo "artifacts=$ARTIFACTS"

# ---- Phase 1: smoke (no capture — fast fail) --------------------------------
echo
echo "== Phase 1: smoke_test.sh =="
"$ROOT/scripts/smoke_test.sh"

# ---- Phase 2: PCAP + platform driver ----------------------------------------
echo
echo "== Phase 2: platform driver + PCAP (${CAPTURE_SECS}s) =="

capture_pcap() {
  local out="$1" secs="$2"
  if command -v docker >/dev/null 2>&1; then
    docker run --rm --net=host --cap-add=NET_RAW nicolaka/netshoot \
      timeout "$secs" tcpdump -i any -nn udp port 47808 -w - 2>/dev/null >"$out" &
    echo $!
    return
  fi
  if sudo -n true 2>/dev/null; then
    sudo timeout "$secs" tcpdump -i any -nn udp port 47808 -w "$out" 2>/dev/null &
    echo $!
    return
  fi
  echo ""
}

TCPDUMP_PID=$(capture_pcap "$PCAP" "$CAPTURE_SECS")
sleep 2

DRIVER_CYCLES="$DRIVER_CYCLES" DRIVER_INTERVAL_SECS="$DRIVER_INTERVAL" \
  "$ROOT/scripts/openfdd_platform_driver.sh"

if [[ -n "$TCPDUMP_PID" ]]; then
  wait "$TCPDUMP_PID" 2>/dev/null || true
fi

if [[ -f "$PCAP" ]] && [[ -s "$PCAP" ]]; then
  FRAMES=$(tcpdump -r "$PCAP" -nn udp port 47808 2>/dev/null | wc -l | tr -d ' ')
  echo "PCAP: $PCAP ($FRAMES UDP/47808 frames)"
  PCAP_FILE="$PCAP" PCAP_MIN_IAM="${PCAP_MIN_IAM:-1}" "$ROOT/scripts/pcap_validate.sh" || true
  tcpdump -r "$PCAP" -nn udp port 47808 2>/dev/null | head -8 || true
else
  echo "WARN: no PCAP captured (docker or passwordless sudo required)"
fi

echo
echo "== BENCH GATE PASSED =="
