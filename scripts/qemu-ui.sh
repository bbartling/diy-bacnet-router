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
#   ssh -N -o ExitOnForwardFailure=yes -L 127.0.0.1:18080:127.0.0.1:18080 ubuntu2-buildroot
#   http://127.0.0.1:18080
set -euo pipefail

STATE_DIR="${DBR_QEMU_UI_DIR:-/tmp/dbr-qemu-ui}"
PID_FILE="$STATE_DIR/qemu.pid"
META_FILE="$STATE_DIR/qemu.meta"
LOG_FILE="$STATE_DIR/qemu.log"
LOCK_FILE="$STATE_DIR/qemu.lock"
IMAGES_FILE="$STATE_DIR/images-dir"
HOST_PORT="${DBR_QEMU_UI_PORT:-18080}"
DASHBOARD_URL="http://127.0.0.1:${HOST_PORT}"
START_TIMEOUT_SEC="${DBR_QEMU_UI_START_TIMEOUT:-90}"

usage() {
  cat <<EOF
usage: $0 start <images-dir>
       $0 status
       $0 stop
EOF
}

ensure_state_dir() {
  mkdir -p "$STATE_DIR"
  chmod 700 "$STATE_DIR" 2>/dev/null || true
}

with_lock() {
  ensure_state_dir
  exec 9>"$LOCK_FILE"
  if ! flock -n 9; then
    echo "another qemu-ui.sh invocation holds the lock ($LOCK_FILE)" >&2
    exit 1
  fi
}

cmdline_of() {
  local pid="$1"
  if [[ -r "/proc/$pid/cmdline" ]]; then
    tr '\0' ' ' <"/proc/$pid/cmdline"
    return 0
  fi
  ps -p "$pid" -o args= 2>/dev/null || true
}

is_our_qemu() {
  local pid="$1"
  local cmdline
  cmdline="$(cmdline_of "$pid")"
  [[ "$cmdline" == *qemu-system-x86_64* ]] || return 1
  [[ "$cmdline" == *dbr.bind=0.0.0.0:8080* ]] || return 1
  [[ "$cmdline" == *hostfwd=tcp:127.0.0.1:${HOST_PORT}-:8080* ]] || return 1
  return 0
}

read_meta_pid() {
  if [[ -f "$PID_FILE" ]]; then
    tr -d '[:space:]' <"$PID_FILE"
  fi
}

is_running() {
  local pid
  pid="$(read_meta_pid)"
  [[ -n "$pid" ]] || return 1
  kill -0 "$pid" 2>/dev/null || return 1
  is_our_qemu "$pid"
}

port_in_use() {
  if command -v ss >/dev/null 2>&1; then
    ss -ltn "( sport = :$HOST_PORT )" 2>/dev/null | grep -q ":$HOST_PORT"
    return $?
  fi
  if command -v lsof >/dev/null 2>&1; then
    lsof -iTCP:"$HOST_PORT" -sTCP:LISTEN >/dev/null 2>&1
    return $?
  fi
  return 1
}

print_status() {
  if is_running; then
    local pid images started
    pid="$(read_meta_pid)"
    images="$(cat "$IMAGES_FILE" 2>/dev/null || echo '(unknown)')"
    started="$(awk -F= '/^started_unix=/{print $2}' "$META_FILE" 2>/dev/null || echo unknown)"
    echo "QEMU UI preview: RUNNING"
    echo "  pid:       $pid"
    echo "  images:    $images"
    echo "  port:      $HOST_PORT"
    echo "  started:   $started"
    echo "  log:       $LOG_FILE"
    echo "  health:    ${DASHBOARD_URL}/healthz"
    echo "  dashboard: ${DASHBOARD_URL}"
    echo "  stop:      $0 stop"
    if curl --fail --silent --max-time 2 "${DASHBOARD_URL}/healthz" >/dev/null 2>&1; then
      echo "  healthz:   OK"
    else
      echo "  healthz:   not ready yet (see log)"
    fi
    return 0
  fi
  echo "QEMU UI preview: STOPPED"
  echo "  log: $LOG_FILE (if present)"
  return 1
}

cmd_start() {
  local images="${1:-}"
  with_lock
  if [[ -z "$images" ]]; then
    usage >&2
    exit 2
  fi
  images="$(cd "$images" && pwd)"
  if [[ ! -f "$images/bzImage" || ! -f "$images/rootfs.ext2" ]]; then
    echo "missing bzImage or rootfs.ext2 under $images" >&2
    exit 1
  fi
  if is_running; then
    echo "refusing to start: QEMU UI already running (pid $(read_meta_pid))" >&2
    print_status >&2 || true
    exit 1
  fi
  # Stale PID file for a dead/unrelated process — clear it; do not signal strangers.
  if [[ -f "$PID_FILE" ]]; then
    local stale
    stale="$(read_meta_pid)"
    if [[ -n "$stale" ]] && kill -0 "$stale" 2>/dev/null && ! is_our_qemu "$stale"; then
      echo "refusing to start: PID $stale is alive but is not our QEMU instance" >&2
      echo "  cmdline: $(cmdline_of "$stale")" >&2
      exit 1
    fi
    rm -f "$PID_FILE" "$META_FILE"
  fi
  if port_in_use; then
    echo "refusing to start: TCP 127.0.0.1:$HOST_PORT already has a listener" >&2
    echo "  not killing the occupant; free the port or set DBR_QEMU_UI_PORT" >&2
    exit 1
  fi

  : >"$LOG_FILE"
  printf '%s\n' "$images" >"$IMAGES_FILE"

  qemu-system-x86_64 \
    -M pc -m 512 -smp 2 \
    -snapshot \
    -kernel "$images/bzImage" \
    -drive "file=$images/rootfs.ext2,if=virtio,format=raw" \
    -append "root=/dev/vda console=ttyS0 noapic dbr.bind=0.0.0.0:8080" \
    -nic "user,model=virtio-net-pci,hostfwd=tcp:127.0.0.1:${HOST_PORT}-:8080" \
    -nographic -no-reboot >"$LOG_FILE" 2>&1 &
  local pid=$!
  local started
  started="$(date +%s)"
  echo "$pid" >"$PID_FILE"
  cat >"$META_FILE" <<EOF
pid=$pid
started_unix=$started
port=$HOST_PORT
images=$images
cmdline_marker=qemu-system-x86_64 dbr.bind=0.0.0.0:8080 hostfwd=tcp:127.0.0.1:${HOST_PORT}-:8080
EOF

  echo "Started QEMU UI preview (pid $pid)"
  echo "  dashboard: ${DASHBOARD_URL}"
  echo "  health:    ${DASHBOARD_URL}/healthz"
  echo "  log:       $LOG_FILE"
  echo "  stop:      $0 stop"
  echo
  echo "Windows tunnel (from host):"
  echo "  ssh -N -o ExitOnForwardFailure=yes -L 127.0.0.1:${HOST_PORT}:127.0.0.1:${HOST_PORT} ubuntu2-buildroot"
  echo "  then open ${DASHBOARD_URL}"

  local i
  for i in $(seq 1 "$START_TIMEOUT_SEC"); do
    if ! kill -0 "$pid" 2>/dev/null; then
      echo "QEMU exited during startup; see $LOG_FILE" >&2
      tail -n 80 "$LOG_FILE" >&2 || true
      rm -f "$PID_FILE" "$META_FILE"
      exit 1
    fi
    if curl --fail --silent --max-time 2 "${DASHBOARD_URL}/healthz" >/dev/null 2>&1; then
      echo "  healthz:   OK after ${i}s"
      return 0
    fi
    sleep 1
  done
  echo "timed out waiting for healthz after ${START_TIMEOUT_SEC}s; leaving QEMU running for diagnosis" >&2
  tail -n 80 "$LOG_FILE" >&2 || true
  exit 1
}

cmd_stop() {
  with_lock
  if [[ ! -f "$PID_FILE" ]]; then
    echo "no PID file; nothing to stop"
    exit 0
  fi
  local pid
  pid="$(read_meta_pid)"
  if [[ -z "$pid" ]]; then
    rm -f "$PID_FILE" "$META_FILE"
    echo "empty PID file removed"
    exit 0
  fi
  if ! kill -0 "$pid" 2>/dev/null; then
    echo "recorded pid $pid is not running"
    rm -f "$PID_FILE" "$META_FILE"
    exit 0
  fi
  if ! is_our_qemu "$pid"; then
    echo "refusing to signal pid $pid: identity mismatch (not our QEMU)" >&2
    echo "  cmdline: $(cmdline_of "$pid")" >&2
    exit 1
  fi
  echo "stopping QEMU pid $pid"
  kill "$pid" 2>/dev/null || true
  local _
  for _ in $(seq 1 40); do
    if ! kill -0 "$pid" 2>/dev/null; then
      break
    fi
    sleep 0.25
  done
  if kill -0 "$pid" 2>/dev/null; then
    if is_our_qemu "$pid"; then
      echo "pid $pid still alive; sending SIGKILL"
      kill -9 "$pid" 2>/dev/null || true
    else
      echo "pid $pid changed identity during stop; not sending SIGKILL" >&2
      exit 1
    fi
  fi
  rm -f "$PID_FILE" "$META_FILE"
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
