#!/usr/bin/env bash
# FEC-off dual-mini baud smoke: passive + routed RP for each allowed baud.
# Topology: bensbench MAC1 + pi2 MAC2 + pi1 MAC3. Oracle on bosspi (.12).
# Appliance runs in privileged Docker (host sandbox cannot open /dev/ttyUSB*).
# Restores minis to 38400 / Max_Master=7 at end. Does NOT restart openfdd-fieldbus.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STAMP="${STAMP:-$(date -u +%Y%m%dT%H%M%SZ)}"
EV="${EV_DIR:-$ROOT/docs/evidence/CLAUSE9_BAUD_MATRIX_FULL_FEC_OFF_${STAMP}}"
mkdir -p "$EV"

ROUTERD="${ROUTERD:-$ROOT/target/release/diy-bacnet-router}"
CFG=/tmp/router.lab-baud-matrix.toml
SERIAL=/dev/serial/by-id/usb-FTDI_FT232R_USB_UART_BH001FQ0-if00-port0
PI1=ben@192.168.204.59
PI2=ben@192.168.204.60
ORACLE=ben@192.168.204.12
BAUDS=(38400 9600 19200 57600 76800 115200)
DOCKER_IMAGE="${DOCKER_IMAGE:-dbr-lab:ip}"

test -x "$ROUTERD" || { echo "missing diy-bacnet-router binary: $ROUTERD" >&2; exit 1; }

git -C "$ROOT" rev-parse HEAD >"$EV/dbr-sha.txt"
echo 9e5168c5ac10bf06f3422f66fc2b0b9983a6ac5c >"$EV/rusty-bacnet-pin.txt"
cat /sys/bus/usb-serial/devices/ttyUSB0/latency_timer >"$EV/latency_timer.txt" 2>/dev/null || echo unknown >"$EV/latency_timer.txt"
: >"$EV/summary.jsonl"

prep_serial() {
  docker run --rm --privileged --network host alpine sh -c \
    'chmod 666 /dev/ttyUSB0 2>/dev/null || true; echo 1 >/sys/bus/usb-serial/devices/ttyUSB0/latency_timer 2>/dev/null || true' \
    >/dev/null 2>&1 || true
}

kill_routerd() {
  docker rm -f dbr-route dbr-passive >/dev/null 2>&1 || true
  sleep 1
}

write_cfg() {
  local baud=$1 mm=$2
  cat >"$CFG" <<EOF
[identity]
name = "bensbench-diy-router"
location = "FEC-off dual-mini baud matrix @${baud}"

[management]
bind = "127.0.0.1:18080"
metrics_interval_ms = 1000
max_ws_connections = 8

[router]
enabled = false

[bacnet_ip]
interface = "eno1"
bind_address = "192.168.204.11"
broadcast_address = "192.168.204.255"
udp_port = 47808
network = 1
bbmd_enabled = false
foreign_device_enabled = false

[mstp]
serial = "$SERIAL"
adapter_profile = "waveshare-usb-to-rs485-c"
termination = "onboard-present"
baud = $baud
mac = 1
network = 2001
max_master = $mm
max_info_frames = 1
EOF
}

restart_minis() {
  local baud=$1 mm=$2
  ssh "$PI1" 'pids=$(pgrep -f "[m]stp-mini-device" || true); [ -n "$pids" ] && kill $pids || true'
  ssh "$PI2" 'pids=$(pgrep -f "[m]stp-mini-device" || true); [ -n "$pids" ] && kill $pids || true'
  sleep 1
  ssh "$PI1" "nohup /home/ben/vibe13-build/target/release/mstp-mini-device \
    --serial /dev/serial/by-id/usb-FTDI_FT232R_USB_UART_BH002I9S-if00-port0 \
    --baud $baud --mac 3 --max-master $mm --max-info-frames 1 \
    --device-instance 123103 --name Mini3 --vendor-id 999 \
    >/tmp/mini3.log 2>&1 </dev/null & echo \$!"
  ssh "$PI2" "nohup /home/ben/src/py-bacnet-stacks-playground/vibe_code_apps_13/target/release/mstp-mini-device \
    --serial /dev/serial/by-id/usb-1a86_USB_Single_Serial_5A98075745-if00 \
    --baud $baud --mac 2 --max-master $mm --max-info-frames 1 \
    --device-instance 123102 --name Mini2 --vendor-id 999 \
    >/tmp/mini2.log 2>&1 </dev/null & echo \$!"
  sleep 5
  ssh "$PI1" 'pgrep -f "[m]stp-mini-device" >/dev/null'
  ssh "$PI2" 'pgrep -f "[m]stp-mini-device" >/dev/null'
}

run_passive() {
  local baud=$1
  kill_routerd
  write_cfg "$baud" 3
  prep_serial
  set +e
  timeout 30 docker run --rm --privileged --network host --name dbr-passive \
    -v "$ROUTERD:/usr/local/bin/diy-bacnet-router:ro" \
    -v "$CFG:$CFG:ro" \
    -v "$EV:$EV" \
    -v /tmp:/tmp \
    -v /dev:/dev \
    "$DOCKER_IMAGE" \
    /usr/local/bin/diy-bacnet-router --config "$CFG" --mstp-passive --expect-source 2 \
      --qualify-secs 12 --mstp-report "$EV/passive-${baud}.json" \
    >"$EV/passive-${baud}.log" 2>&1
  local rc=$?
  set -e
  echo "passive baud=$baud rc=$rc"
  if [[ -f "$EV/passive-${baud}.json" ]]; then
    python3 - <<PY
import json
from pathlib import Path
d=json.loads(Path("$EV/passive-${baud}.json").read_text())
print("  passive ok=", d.get("ok"), "valid_ratio=", d.get("valid_ratio"), "tokens=", d.get("tokens"), "sources=", d.get("sources_seen"), "fail=", d.get("failure_reason"))
PY
  fi
}

run_route_rp() {
  local baud=$1
  kill_routerd
  write_cfg "$baud" 3
  prep_serial
  docker run --rm --privileged --network host --name dbr-route \
    -v "$ROUTERD:/usr/local/bin/diy-bacnet-router:ro" \
    -v "$CFG:$CFG:ro" \
    -v "$EV:$EV" \
    -v /tmp:/tmp \
    -v /dev:/dev \
    "$DOCKER_IMAGE" \
    /usr/local/bin/diy-bacnet-router --config "$CFG" --route-enable --qualify-secs 0 \
      --route-report "$EV/route-${baud}.json" >"$EV/route-${baud}.log" 2>&1 &
  sleep 10
  set +e
  ssh "$ORACLE" "HOME=/home/ben /home/ben/lab-venv/bin/python /home/ben/lab_routed_rp_oracle.py \
    --address 192.168.204.12/24 --router 192.168.204.11 --explicit-router \
    --timeout 12 -o /tmp/rp-${baud}.json" >"$EV/rp-${baud}.console.txt" 2>&1
  local rc=$?
  set -e
  scp -q "$ORACLE:/tmp/rp-${baud}.json" "$EV/rp-${baud}.json" || echo '{"error":"scp_failed"}' >"$EV/rp-${baud}.json"
  kill_routerd
  echo "rp baud=$baud oracle_rc=$rc"
  python3 - <<PY
import json
from pathlib import Path
d=json.loads(Path("$EV/rp-${baud}.json").read_text())
print("  ok", d.get("ok_count"), "fail", d.get("fail_count"), "all_ok", d.get("all_ok"))
for r in d.get("reads", []):
    print(" ", r.get("label"), r.get("obj"), "ok="+str(r.get("ok")), (r.get("err") or str(r.get("val","")))[:60])
PY
}

echo "EV=$EV"
python3 - <<'PY'
import socket
s=socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
try:
    s.bind(("192.168.204.11", 47808))
    print("47808 FREE")
except OSError as e:
    raise SystemExit(f"47808 busy — stop openfdd-fieldbus first: {e}")
finally:
    s.close()
PY

for baud in "${BAUDS[@]}"; do
  echo "======== BAUD $baud ========"
  restart_minis "$baud" 3
  run_passive "$baud"
  run_route_rp "$baud"
  python3 - <<PY >>"$EV/summary.jsonl"
import json
from pathlib import Path
baud = $baud
ev = Path("$EV")
row = {"baud": baud}
pp = ev / f"passive-{baud}.json"
rp = ev / f"rp-{baud}.json"
if pp.exists():
    try:
        d = json.loads(pp.read_text())
        row["passive_ok"] = d.get("ok")
        row["valid_ratio"] = d.get("valid_ratio")
        row["tokens"] = d.get("tokens")
        row["sources_seen"] = d.get("sources_seen")
        row["failure_reason"] = d.get("failure_reason")
    except Exception as e:
        row["passive_err"] = str(e)
if rp.exists():
    try:
        d = json.loads(rp.read_text())
        row["rp_ok_count"] = d.get("ok_count")
        row["rp_fail_count"] = d.get("fail_count")
        row["rp_all_ok"] = d.get("all_ok")
        row["reads"] = d.get("reads")
    except Exception as e:
        row["rp_err"] = str(e)
print(json.dumps(row))
PY
done

echo "======== RESTORE 38400 Max_Master=7 ========"
restart_minis 38400 7
kill_routerd
echo "DONE EV=$EV"
