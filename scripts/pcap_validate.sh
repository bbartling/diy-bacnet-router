#!/usr/bin/env bash
# BACnet PCAP validation gate — fails CI if captured traffic violates expected patterns.
#
# Usage:
#   scripts/pcap_validate.sh [pcap_file]
#
# Env:
#   PCAP_FILE          — default capture path
#   PCAP_MIN_IAM       — minimum I-Am responses expected (default 0)
#   PCAP_FORBID_WRITE  — if 1, fail on WriteProperty frames (default 0)
#   PCAP_DEVICE_ID     — optional device instance to filter (e.g. 599999)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PCAP="${1:-${PCAP_FILE:-${ROOT}/artifacts/bacnet_capture.pcap}}"

if [[ ! -f "$PCAP" ]]; then
  echo "pcap_validate: skip — no capture at $PCAP"
  exit 0
fi

if ! command -v tshark >/dev/null 2>&1; then
  echo "pcap_validate: tshark not installed; skipping"
  exit 0
fi

DEVICE_FILTER=""
if [[ -n "${PCAP_DEVICE_ID:-}" ]]; then
  DEVICE_FILTER="bacnet.instance_number == ${PCAP_DEVICE_ID}"
fi

count_frames() {
  local display_filter="$1"
  local filter="$display_filter"
  if [[ -n "$DEVICE_FILTER" ]]; then
    filter="($display_filter) && ($DEVICE_FILTER)"
  fi
  tshark -r "$PCAP" -Y "$filter" -T fields -e frame.number 2>/dev/null | wc -l | tr -d ' '
}

IAM_COUNT=$(count_frames "bacnet.msgtype == 0x00 && bacnet.apdu.type == 0x10")
WRITE_COUNT=$(count_frames "bacnet.apdu.service == 0x0f")

MIN_IAM="${PCAP_MIN_IAM:-0}"
FORBID_WRITE="${PCAP_FORBID_WRITE:-0}"

echo "pcap_validate: file=$PCAP iam=$IAM_COUNT writes=$WRITE_COUNT min_iam=$MIN_IAM forbid_write=$FORBID_WRITE"

if (( IAM_COUNT < MIN_IAM )); then
  echo "pcap_validate: FAIL — expected at least $MIN_IAM I-Am, got $IAM_COUNT"
  exit 1
fi

if (( FORBID_WRITE == 1 && WRITE_COUNT > 0 )); then
  echo "pcap_validate: FAIL — WriteProperty detected ($WRITE_COUNT) but PCAP_FORBID_WRITE=1"
  exit 1
fi

echo "pcap_validate: OK"
exit 0
