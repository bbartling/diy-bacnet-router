#!/usr/bin/env bash
# M2A — qualify one BACnet/IP port on an isolated Linux netns topology.
# Evidence class: real-kernel (not unit). Does not enable forwarding or MS/TP.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

NS_A="dbr-bip-a"
NS_B="dbr-bip-b"
VETH_A="dbr-veth-a"
VETH_B="dbr-veth-b"
DUT_IP="192.0.2.1"
PEER_IP="192.0.2.2"
PREFIX=24
BROADCAST="192.0.2.255"
BACNET_PORT=47808
MGMT_PORT=18081
WORKDIR="${TMPDIR:-/tmp}/dbr-bip-qualify-$$"
BIN="${DBR_BIN:-$repo_root/target/debug/diy-bacnet-router}"
TIMEOUT_SECS="${DBR_BIP_QUALIFY_TIMEOUT:-45}"
CREATED_NS=()
CREATED_VETH=0

log() { printf '[bip-qualify] %s\n' "$*"; }
die() { printf '[bip-qualify] ERROR: %s\n' "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

refuse_existing() {
  if ip netns list | awk '{print $1}' | grep -qx "$1"; then
    die "refusing to reuse pre-existing netns '$1' (will not delete strangers)"
  fi
  if ip link show "$2" &>/dev/null; then
    die "refusing to reuse pre-existing link '$2' (will not delete strangers)"
  fi
}

cleanup() {
  local code=$?
  set +e
  if [[ -n "${DUT_PID:-}" ]]; then kill "$DUT_PID" 2>/dev/null; wait "$DUT_PID" 2>/dev/null; fi
  if [[ -n "${PEER_PID:-}" ]]; then kill "$PEER_PID" 2>/dev/null; wait "$PEER_PID" 2>/dev/null; fi
  if [[ "$CREATED_VETH" -eq 1 ]]; then
    ip link delete "$VETH_A" 2>/dev/null
  fi
  for ns in "${CREATED_NS[@]:-}"; do
    ip netns delete "$ns" 2>/dev/null
  done
  if [[ "$code" -ne 0 && -d "$WORKDIR" ]]; then
    log "preserving diagnostics in $WORKDIR"
  else
    rm -rf "$WORKDIR" 2>/dev/null
  fi
  exit "$code"
}
trap cleanup EXIT

need_cmd ip
need_cmd curl
need_cmd python3
[[ "$(id -u)" -eq 0 ]] || die "must run as root (netns/veth)"
[[ -x "$BIN" ]] || die "binary not executable: $BIN (set DBR_BIN or cargo build -p routerd)"

refuse_existing "$NS_A" "$VETH_A"
refuse_existing "$NS_B" "$VETH_B"

mkdir -p "$WORKDIR"
log "workdir $WORKDIR (bounded failure artifacts only)"

ip netns add "$NS_A"
CREATED_NS+=("$NS_A")
ip netns add "$NS_B"
CREATED_NS+=("$NS_B")
ip link add "$VETH_A" type veth peer name "$VETH_B"
CREATED_VETH=1
ip link set "$VETH_A" netns "$NS_A"
ip link set "$VETH_B" netns "$NS_B"

ip -n "$NS_A" addr add "${PEER_IP}/${PREFIX}" broadcast "$BROADCAST" dev "$VETH_A"
ip -n "$NS_B" addr add "${DUT_IP}/${PREFIX}" broadcast "$BROADCAST" dev "$VETH_B"
ip -n "$NS_A" link set lo up
ip -n "$NS_B" link set lo up
ip -n "$NS_A" link set "$VETH_A" up
ip -n "$NS_B" link set "$VETH_B" up

# Never touch host default route / DNS / physical NIC — only ns-local veths above.

cat >"$WORKDIR/dut.toml" <<EOF
[identity]
name = "dbr-bip-qualify-dut"
location = "netns-m2a"

[management]
bind = "127.0.0.1:${MGMT_PORT}"
web_root = "frontend/web/dist"
metrics_interval_ms = 250
max_ws_connections = 4

[router]
enabled = false

[bacnet_ip]
interface = "${VETH_B}"
bind_address = "${DUT_IP}"
broadcast_address = "${BROADCAST}"
udp_port = ${BACNET_PORT}
network = 1
bbmd_enabled = false
foreign_device_enabled = false

[mstp]
serial = "/dev/serial/by-id/REPLACE_ME"
adapter_profile = "waveshare-usb-to-rs485-c"
termination = "onboard-present"
baud = 38400
mac = 3
network = 2000
max_master = 127
max_info_frames = 1
EOF

cat >"$WORKDIR/peer.toml" <<EOF
[identity]
name = "dbr-bip-qualify-peer"
location = "netns-m2a"

[management]
bind = "127.0.0.1:9"
web_root = "frontend/web/dist"
metrics_interval_ms = 1000
max_ws_connections = 1

[router]
enabled = false

[bacnet_ip]
interface = "${VETH_A}"
bind_address = "${PEER_IP}"
broadcast_address = "${BROADCAST}"
udp_port = ${BACNET_PORT}
network = 1
bbmd_enabled = false
foreign_device_enabled = false

[mstp]
serial = "/dev/serial/by-id/REPLACE_ME"
adapter_profile = "waveshare-usb-to-rs485-c"
termination = "onboard-present"
baud = 38400
mac = 3
network = 2000
max_master = 127
max_info_frames = 1
EOF

log "starting DUT in $NS_B"
ip netns exec "$NS_B" "$BIN" --config "$WORKDIR/dut.toml" --bip-qualify --qualify-secs "$TIMEOUT_SECS" \
  >"$WORKDIR/dut.log" 2>&1 &
DUT_PID=$!

for _ in $(seq 1 60); do
  if ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/healthz" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done
ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/healthz" | tee "$WORKDIR/healthz.json"

log "sending peer probes from $NS_A"
ip netns exec "$NS_A" "$BIN" --config "$WORKDIR/peer.toml" --bip-qualify-peer \
  --probe-count 8 --peer-target "${DUT_IP}:${BACNET_PORT}" \
  >"$WORKDIR/peer.log" 2>&1
PEER_RC=$?
[[ "$PEER_RC" -eq 0 ]] || die "peer exited $PEER_RC (see $WORKDIR/peer.log)"

sleep 1
ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/api/metrics/snapshot" \
  | tee "$WORKDIR/metrics.json"
ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/healthz" \
  | tee "$WORKDIR/healthz-after.json"

python3 - <<'PY' "$WORKDIR/metrics.json" "$WORKDIR/healthz-after.json"
import json, sys
metrics = json.load(open(sys.argv[1]))
health = json.load(open(sys.argv[2]))
router = metrics["router"]
runtime = metrics["runtime"]
assert health.get("ready_to_route") is False, health
assert router["forwarded_bip_to_mstp"] == 0
assert router["forwarded_mstp_to_bip"] == 0
assert router["bip_rx_packets"] >= 1, router
assert runtime["bip_link"] == "operational", runtime
assert runtime["mstp_link"] == "disabled", runtime
assert metrics.get("bacnet_telemetry_available") is True, metrics
# MS/TP counters remain zero / not qualified — do not treat as observed wire data.
assert router["mstp_rx_packets"] == 0
assert router["tx_tokens"] == 0
print("PASS: M2A B/IP netns qualification checks")
PY

log "PASS (real-kernel netns evidence)"
