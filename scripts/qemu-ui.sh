#!/usr/bin/env bash
# Persistent QEMU UI preview for the x86_64 Buildroot appliance (lab use).
# Uses -snapshot and loopback-only hostfwd. Never mutates rootfs.ext2.
#
# Usage:
#   scripts/qemu-ui.sh start <images-dir>
#   scripts/qemu-ui.sh status
#   scripts/qemu-ui.sh stop
#
# From Windows, tunnel then browse:
#   ssh -N -L 18080:127.0.0.1:18080 ubuntu2-buildroot
#   http://127.0.0.1:18080
set -euo pipefail

STATE_DIR="${DBR_QEMU_UI_DIR:-/tmp/dbr-qemu-ui}"
PID_FILE="$STATE_DIR/qemu.pid"
LOG_FILE="$STATE_DIR/qemu.log"
IMAGES_FILE="$STATE_DIR/images-dir"
HOST_PORT="${DBR_QEMU_UI_PORT:-18080}"
DASHBOARD_URL="http://127.0.0.1:${HOST_PORT}"

usage() {
  cat <<EOF
usage: $0 start <images-dir>
       $0 status
       $0 stop
EOF
}

is_running() {
  if [[ ! -f "$PID_FILE" ]]; then
    return 1
  fi
  local pid
  pid="$(cat "$PID_FILE")"
  if [[ -z "$pid" ]]; then
    return 1
  fi
  kill -0 "$pid" 2>/dev/null
}

print_status() {
  if is_running; then
    local pid images
    pid="$(cat "$PID_FILE")"
    images="$(cat "$IMAGES_FILE" 2>/dev/null || echo '(unknown)')"
    echo "QEMU UI preview: RUNNING"
    echo "  pid:      $pid"
    echo "  images:   $images"
    echo "  log:      $LOG_FILE"
    echo "  health:   ${DASHBOARD_URL}/healthz"
    echo "  dashboard:${DASHBOARD_URL}"
    echo "  stop:     $0 stop"
    if curl --fail --silent --max-time 2 "${DASHBOARD_URL}/healthz" >/dev/null 2>&1; then
      echo "  healthz:  OK"
    else
      echo "  healthz:  not ready yet (see log)"
    fi
    return 0
  fi
  echo "QEMU UI preview: STOPPED"
  echo "  log: $LOG_FILE (if present)"
  return 1
}

cmd_start() {
  local images="${1:-}"
  if [[ -z "$images" ]]; then
    usage >&2
    exit 2
  fi
  if [[ ! -f "$images/bzImage" || ! -f "$images/rootfs.ext2" ]]; then
    echo "missing bzImage or rootfs.ext2 under $images" >&2
    exit 1
  fi
  if is_running; then
    echo "refusing to start: QEMU UI already running (pid $(cat "$PID_FILE"))" >&2
    print_status >&2 || true
    exit 1
  fi

  mkdir -p "$STATE_DIR"
  rm -f "$PID_FILE"
  : >"$LOG_FILE"
  printf '%s\n' "$images" >"$IMAGES_FILE"

  # Loopback-only forward; -snapshot so rootfs.ext2 is never mutated.
  qemu-system-x86_64 \
    -M pc -m 512 -smp 2 \
    -snapshot \
    -kernel "$images/bzImage" \
    -drive "file=$images/rootfs.ext2,if=virtio,format=raw" \
    -append "root=/dev/vda console=ttyS0 noapic dbr.bind=0.0.0.0:8080" \
    -nic "user,model=virtio-net-pci,hostfwd=tcp:127.0.0.1:${HOST_PORT}-:8080" \
    -nographic -no-reboot >"$LOG_FILE" 2>&1 &
  local pid=$!
  echo "$pid" >"$PID_FILE"

  echo "Started QEMU UI preview (pid $pid)"
  echo "  dashboard: ${DASHBOARD_URL}"
  echo "  health:    ${DASHBOARD_URL}/healthz"
  echo "  log:       $LOG_FILE"
  echo "  stop:      $0 stop"
  echo
  echo "Windows tunnel (from host):"
  echo "  ssh -N -L ${HOST_PORT}:127.0.0.1:${HOST_PORT} ubuntu2-buildroot"
  echo "  then open ${DASHBOARD_URL}"
}

cmd_stop() {
  if [[ ! -f "$PID_FILE" ]]; then
    echo "no PID file; nothing to stop"
    exit 0
  fi
  local pid
  pid="$(cat "$PID_FILE")"
  if [[ -z "$pid" ]]; then
    rm -f "$PID_FILE"
    echo "empty PID file removed"
    exit 0
  fi
  if kill -0 "$pid" 2>/dev/null; then
    echo "stopping QEMU pid $pid"
    kill "$pid" 2>/dev/null || true
    for _ in $(seq 1 20); do
      if ! kill -0 "$pid" 2>/dev/null; then
        break
      fi
      sleep 0.25
    done
    if kill -0 "$pid" 2>/dev/null; then
      echo "pid $pid still alive; sending SIGKILL"
      kill -9 "$pid" 2>/dev/null || true
    fi
  else
    echo "recorded pid $pid is not running"
  fi
  rm -f "$PID_FILE"
  echo "QEMU UI preview stopped (rootfs.ext2 unchanged; used -snapshot)"
}

case "${1:-}" in
  start)
    shift
    cmd_start "${1:-}"
    ;;
  status)
    print_status
    ;;
  stop)
    cmd_stop
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
