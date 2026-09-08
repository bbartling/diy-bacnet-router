#!/usr/bin/env bash
# M3 dual-B/IP netns route harness — Clause 6 forward between two B/IP nets.
# Evidence class: real-kernel. No MS/TP. Product G7/G8 BIP↔MS/TP remains OPEN.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

RUN_ID="${DBR_ROUTE_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$$}"
SUFF="${RUN_ID: -6}"
SUFF="${SUFF//[^a-zA-Z0-9]/x}"
NS_A="ra${SUFF}"
NS_DUT="rd${SUFF}"
NS_B="rb${SUFF}"
VA_A="vaa${SUFF}"
VA_D="vad${SUFF}"
VB_D="vbd${SUFF}"
VB_B="vbb${SUFF}"

NET_A=1000
NET_B=2000
DUT_A_IP="192.0.2.1"
PEER_A_IP="192.0.2.2"
DUT_B_IP="198.51.100.1"
PEER_B_IP="198.51.100.2"
BACNET_PORT=47808
MGMT_PORT=18082
MATRIX_N="${DBR_ROUTE_MATRIX_N:-4}"
BIN="${DBR_BIN:-$repo_root/target/debug/diy-bacnet-router}"
ORACLE="${DBR_ORACLE:-$repo_root/scripts/bip_bvll_oracle.py}"
EVIDENCE_DIR="${DBR_ROUTE_EVIDENCE_DIR:-/tmp/dbr-bip-bip-${RUN_ID}}"
CREATED_NS=()
CREATED_VETH=()
DUT_PID=""
RECV_A_PID=""
RECV_B_PID=""

log() { printf '[bip-bip] %s\n' "$*"; }
die() { printf '[bip-bip] ERROR: %s\n' "$*" >&2; exit 1; }
need_cmd() { command -v "$1" >/dev/null 2>&1 || die "missing $1"; }

cleanup() {
  local code=$?
  set +e
  [[ -n "${RECV_A_PID}" ]] && kill "$RECV_A_PID" 2>/dev/null
  [[ -n "${RECV_B_PID}" ]] && kill "$RECV_B_PID" 2>/dev/null
  [[ -n "${DUT_PID}" ]] && kill "$DUT_PID" 2>/dev/null
  wait "$RECV_A_PID" "$RECV_B_PID" "$DUT_PID" 2>/dev/null || true
  for link in "${CREATED_VETH[@]:-}"; do ip link delete "$link" 2>/dev/null || true; done
  for ns in "${CREATED_NS[@]:-}"; do ip netns delete "$ns" 2>/dev/null || true; done
  if [[ -d "$EVIDENCE_DIR" ]]; then
    log "evidence preserved at $EVIDENCE_DIR (exit=$code)"
  fi
  exit "$code"
}
trap cleanup EXIT

need_cmd ip; need_cmd curl; need_cmd python3; need_cmd sha256sum
[[ "$(id -u)" -eq 0 ]] || die "must run as root"
[[ -x "$BIN" ]] || die "binary not executable: $BIN"
python3 "$ORACLE" self-test

for ns in "$NS_A" "$NS_DUT" "$NS_B"; do
  if ip netns list 2>/dev/null | awk '{print $1}' | grep -qx "$ns"; then
    die "refusing pre-existing netns $ns"
  fi
done

mkdir -p "$EVIDENCE_DIR"
GIT_SHA="$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || echo unknown)"
UPSTREAM_SHA="$(awk -F'"' '/^revision = /{print $2; exit}' "$repo_root/config/upstream-lock.toml")"
cat >"$EVIDENCE_DIR/manifest.json" <<EOF
{
  "git_sha": "$GIT_SHA",
  "rusty_bacnet_sha": "$UPSTREAM_SHA",
  "run_id": "$RUN_ID",
  "mode": "dual_bip_netns",
  "net_a": $NET_A,
  "net_b": $NET_B,
  "matrix_n": $MATRIX_N,
  "kernel": "$(uname -r)"
}
EOF

ip netns add "$NS_A"; CREATED_NS+=("$NS_A")
ip netns add "$NS_DUT"; CREATED_NS+=("$NS_DUT")
ip netns add "$NS_B"; CREATED_NS+=("$NS_B")

ip link add "$VA_A" type veth peer name "$VA_D"
CREATED_VETH+=("$VA_A")
ip link add "$VB_B" type veth peer name "$VB_D"
CREATED_VETH+=("$VB_B")

ip link set "$VA_A" netns "$NS_A"
ip link set "$VA_D" netns "$NS_DUT"
ip link set "$VB_B" netns "$NS_B"
ip link set "$VB_D" netns "$NS_DUT"

ip -n "$NS_A" addr add "${PEER_A_IP}/24" dev "$VA_A"
ip -n "$NS_A" link set "$VA_A" up
ip -n "$NS_A" link set lo up

ip -n "$NS_DUT" addr add "${DUT_A_IP}/24" dev "$VA_D"
ip -n "$NS_DUT" addr add "${DUT_B_IP}/24" dev "$VB_D"
ip -n "$NS_DUT" link set "$VA_D" up
ip -n "$NS_DUT" link set "$VB_D" up
ip -n "$NS_DUT" link set lo up

ip -n "$NS_B" addr add "${PEER_B_IP}/24" dev "$VB_B"
ip -n "$NS_B" link set "$VB_B" up
ip -n "$NS_B" link set lo up

ip -n "$NS_A" -br addr >"$EVIDENCE_DIR/ns_a_addrs.txt"
ip -n "$NS_DUT" -br addr >"$EVIDENCE_DIR/ns_dut_addrs.txt"
ip -n "$NS_B" -br addr >"$EVIDENCE_DIR/ns_b_addrs.txt"

CFG="$EVIDENCE_DIR/router.toml"
cat >"$CFG" <<EOF
[identity]
name = "dual-bip-dut"
location = "ci-netns"

[management]
bind = "127.0.0.1:${MGMT_PORT}"
web_root = "frontend/web/dist"
metrics_interval_ms = 1000
max_ws_connections = 4

[router]
enabled = false

[bacnet_ip]
interface = "${VA_D}"
bind_address = "${DUT_A_IP}"
broadcast_address = "192.0.2.255"
udp_port = ${BACNET_PORT}
network = ${NET_A}
bbmd_enabled = false
foreign_device_enabled = false

[mstp]
serial = "/dev/serial/by-id/usb-placeholder-ci"
adapter_profile = "waveshare-usb-to-rs485-c"
baud = 38400
mac = 1
max_master = 2
max_info_frames = 1
network = 3000
termination = "unknown"
EOF

# BIP MAC for peer B: IP + port BE
# 198.51.100.2:47808 -> 198,51,100,2,186,192
DMAC_B="198,51,100,2,186,192"
DMAC_A="192,0,2,2,186,192"

# Recv must finish its window so summary JSON is written (do not kill early).
RECV_SECS=20
QUALIFY_SECS=15
ip netns exec "$NS_B" python3 "$ORACLE" recv \
  --bind "$PEER_B_IP" --port "$BACNET_PORT" --seconds "$RECV_SECS" \
  --out "$EVIDENCE_DIR/recv_b.json" \
  >"$EVIDENCE_DIR/recv_b.log" 2>&1 &
RECV_B_PID=$!
ip netns exec "$NS_A" python3 "$ORACLE" recv \
  --bind "$PEER_A_IP" --port "$BACNET_PORT" --seconds "$RECV_SECS" \
  --out "$EVIDENCE_DIR/recv_a.json" \
  >"$EVIDENCE_DIR/recv_a.log" 2>&1 &
RECV_A_PID=$!
sleep 0.5

ip netns exec "$NS_DUT" env \
  DBR_BIP2_BIND="$DUT_B_IP" \
  DBR_BIP2_BROADCAST="198.51.100.255" \
  DBR_BIP2_IFACE="$VB_D" \
  DBR_BIP2_NETWORK="$NET_B" \
  DBR_BIP2_PORT="$BACNET_PORT" \
  "$BIN" --config "$CFG" --route-bip-bip --qualify-secs "$QUALIFY_SECS" \
  --route-report "$EVIDENCE_DIR/route_report.json" \
  >"$EVIDENCE_DIR/dut.log" 2>&1 &
DUT_PID=$!

for _ in $(seq 1 40); do
  if ip netns exec "$NS_DUT" curl -fsS "http://127.0.0.1:${MGMT_PORT}/healthz" >"$EVIDENCE_DIR/health_pre.json" 2>/dev/null; then
    break
  fi
  sleep 0.25
done
[[ -f "$EVIDENCE_DIR/health_pre.json" ]] || die "DUT healthz not ready"
python3 - "$EVIDENCE_DIR/health_pre.json" <<'PY'
import json,sys
h=json.load(open(sys.argv[1]))
assert h["ready_to_route"] is False
assert h["status"]=="ok"
PY

# Who-Is-Router from A (best-effort network-message observation on B or A)
ip netns exec "$NS_A" python3 "$ORACLE" send \
  --mode who-is-router --bind "$PEER_A_IP" --dest "$DUT_A_IP" --port "$BACNET_PORT" \
  --count 2 --out "$EVIDENCE_DIR/send_whois.json"

# A → B routed unicast
ip netns exec "$NS_A" python3 "$ORACLE" send \
  --mode routed-unicast --bind "$PEER_A_IP" --dest "$DUT_A_IP" --port "$BACNET_PORT" \
  --dnet "$NET_B" --dmac "$DMAC_B" --count "$MATRIX_N" \
  --out "$EVIDENCE_DIR/send_a_to_b.json"

# B → A routed unicast
ip netns exec "$NS_B" python3 "$ORACLE" send \
  --mode routed-unicast --bind "$PEER_B_IP" --dest "$DUT_B_IP" --port "$BACNET_PORT" \
  --dnet "$NET_A" --dmac "$DMAC_A" --count "$MATRIX_N" \
  --out "$EVIDENCE_DIR/send_b_to_a.json"

# Allow forwards to land, then SIGTERM DUT so graceful shutdown writes --route-report.
# (Management plane keeps running after the router session timeout; do not wait forever.)
sleep 2
kill -TERM "$DUT_PID" 2>/dev/null || true
wait "$DUT_PID" 2>/dev/null || true
DUT_PID=""

# Recv must finish its window so summary JSON is written (do not kill early).
wait "$RECV_A_PID" || true
wait "$RECV_B_PID" || true
RECV_A_PID=""; RECV_B_PID=""
[[ -f "$EVIDENCE_DIR/recv_a.json" ]] || die "recv_a.json missing"
[[ -f "$EVIDENCE_DIR/recv_b.json" ]] || die "recv_b.json missing"

python3 - "$EVIDENCE_DIR" "$MATRIX_N" <<'PY'
import json, pathlib, sys
ev = pathlib.Path(sys.argv[1])
n = int(sys.argv[2])
recv_b = json.loads((ev / "recv_b.json").read_text(encoding="utf-8"))
recv_a = json.loads((ev / "recv_a.json").read_text(encoding="utf-8"))
a_to_b = recv_b.get("routed_payload_ok", 0)
b_to_a = recv_a.get("routed_payload_ok", 0)
net_msgs = recv_a.get("network_message_ok", 0) + recv_b.get("network_message_ok", 0)
report = {}
rp = ev / "route_report.json"
if rp.exists():
    report = json.loads(rp.read_text(encoding="utf-8"))
subgates = {
    "U_a_to_b": {"expected": n, "observed": a_to_b, "pass": a_to_b >= n},
    "U_b_to_a": {"expected": n, "observed": b_to_a, "pass": b_to_a >= n},
    "WhoIs_Router_netmsg": {
        "expected": ">=0 observed",
        "observed": net_msgs,
        "pass": True,
        "note": "I-Am/Who-Is observation best-effort; not a product G8 PASS",
    },
    "ready_to_route_false": {"pass": True},
    "mstp_absent": {
        "pass": (not rp.exists()) or report.get("mstp") is False,
        "note": "dual-B/IP path must not claim MS/TP; report optional if DUT aborted",
    },
}
status = "PASS" if all(s.get("pass") for s in subgates.values()) else "FAIL"
result = {
    "status": status,
    "mode": "dual_bip_netns",
    "matrix_n": n,
    "subgates": subgates,
    "product_g7_g8_bip_mstp": "OPEN",
    "upstream_gaps": [
        "No public BACnetRouter forward counters at pin; packet observation via independent BVLL oracle only"
    ],
}
(ev / "result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, indent=2))
if status != "PASS":
    raise SystemExit(1)
PY

(
  cd "$EVIDENCE_DIR"
  find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum >SHA256SUMS
)

log "PASS dual-B/IP netns matrix N=$MATRIX_N"
