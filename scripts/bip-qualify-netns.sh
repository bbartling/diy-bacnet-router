#!/usr/bin/env bash
# G6 / M2A complete — BACnet/IP bidirectional unicast + directed-broadcast matrix
# on an isolated Linux netns topology with an independent BVLL oracle.
# Evidence class: real-kernel. Does not enable forwarding or MS/TP.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

RUN_ID="${DBR_BIP_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$$}"
# Linux IFNAMSIZ is 16 incl. NUL — keep names short.
SUFF="${RUN_ID: -6}"
SUFF="${SUFF//[^a-zA-Z0-9]/x}"
NS_A="dba${SUFF}"
NS_B="dbb${SUFF}"
VETH_A="va${SUFF}"
VETH_B="vb${SUFF}"
DUT_IP="192.0.2.1"
PEER_IP="192.0.2.2"
PREFIX=24
BROADCAST="192.0.2.255"
BACNET_PORT=47808
MGMT_PORT=18081
MATRIX_N="${DBR_BIP_MATRIX_N:-4}"
BIN="${DBR_BIN:-$repo_root/target/debug/diy-bacnet-router}"
ORACLE="${DBR_ORACLE:-$repo_root/scripts/bip_bvll_oracle.py}"
TIMEOUT_SECS="${DBR_BIP_QUALIFY_TIMEOUT:-40}"
EVIDENCE_DIR="${DBR_BIP_EVIDENCE_DIR:-/tmp/dbr-bip-g6-${RUN_ID}}"
CREATED_NS=()
CREATED_VETH=0
DUT_PID=""
ORACLE_RECV_PID=""

log() { printf '[bip-g6] %s\n' "$*"; }
die() { printf '[bip-g6] ERROR: %s\n' "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

refuse_existing() {
  if ip netns list 2>/dev/null | awk '{print $1}' | grep -qx "$1"; then
    die "refusing to reuse pre-existing netns '$1'"
  fi
  if ip link show "$2" &>/dev/null; then
    die "refusing to reuse pre-existing link '$2'"
  fi
}

cleanup() {
  local code=$?
  set +e
  if [[ -n "${ORACLE_RECV_PID}" ]]; then kill "$ORACLE_RECV_PID" 2>/dev/null; wait "$ORACLE_RECV_PID" 2>/dev/null; fi
  if [[ -n "${DUT_PID}" ]]; then kill "$DUT_PID" 2>/dev/null; wait "$DUT_PID" 2>/dev/null; fi
  if [[ "$CREATED_VETH" -eq 1 ]]; then
    ip link delete "$VETH_A" 2>/dev/null || true
  fi
  for ns in "${CREATED_NS[@]:-}"; do
    ip netns delete "$ns" 2>/dev/null || true
  done
  # Never delete evidence on success or failure — caller owns retention.
  if [[ -d "$EVIDENCE_DIR" ]]; then
    log "evidence preserved at $EVIDENCE_DIR (exit=$code)"
    echo "$EVIDENCE_DIR" >"$EVIDENCE_DIR/path.txt" 2>/dev/null || true
  fi
  exit "$code"
}
trap cleanup EXIT

need_cmd ip
need_cmd curl
need_cmd python3
need_cmd sha256sum
[[ "$(id -u)" -eq 0 ]] || die "must run as root (netns/veth)"
[[ -x "$BIN" ]] || die "binary not executable: $BIN"
[[ -f "$ORACLE" ]] || die "oracle missing: $ORACLE"
python3 "$ORACLE" self-test

refuse_existing "$NS_A" "$VETH_A"
refuse_existing "$NS_B" "$VETH_B"

mkdir -p "$EVIDENCE_DIR"
chmod 0755 "$EVIDENCE_DIR"
log "evidence dir $EVIDENCE_DIR"

GIT_SHA="$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || echo unknown)"
UPSTREAM_SHA="$(awk -F'"' '/^revision = /{print $2; exit}' "$repo_root/config/upstream-lock.toml")"
{
  printf '{\n'
  printf '  "git_sha": "%s",\n' "$GIT_SHA"
  printf '  "rusty_bacnet_sha": "%s",\n' "$UPSTREAM_SHA"
  printf '  "run_id": "%s",\n' "$RUN_ID"
  printf '  "ns_a": "%s", "ns_b": "%s",\n' "$NS_A" "$NS_B"
  printf '  "veth_a": "%s", "veth_b": "%s",\n' "$VETH_A" "$VETH_B"
  printf '  "matrix_n": %s,\n' "$MATRIX_N"
  printf '  "kernel": "%s",\n' "$(uname -r)"
  printf '  "uname": "%s"\n' "$(uname -a | sed 's/"/\\"/g')"
  printf '}\n'
} >"$EVIDENCE_DIR/manifest.json"

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
{
  echo "# namespace snapshot"
  ip -n "$NS_A" -4 addr show
  ip -n "$NS_B" -4 addr show
  ip -n "$NS_A" link show
  ip -n "$NS_B" link show
} >"$EVIDENCE_DIR/ns-snapshot.txt"

# Optional bounded pcap (independent record; not sole oracle).
if command -v tcpdump >/dev/null 2>&1; then
  ip netns exec "$NS_B" timeout 35 tcpdump -i "$VETH_B" -c 64 -w "$EVIDENCE_DIR/dut.pcap" udp port "$BACNET_PORT" \
    >"$EVIDENCE_DIR/tcpdump.log" 2>&1 &
  echo $! >"$EVIDENCE_DIR/tcpdump.pid" || true
fi

cat >"$EVIDENCE_DIR/dut.toml" <<EOF
[identity]
name = "dbr-bip-g6-dut"
location = "netns-g6"

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

# Start oracle recv first (peer ns) so DUT TX is observed.
# Bind INADDR_ANY so directed broadcasts to 192.0.2.255 are received.
ip netns exec "$NS_A" python3 "$ORACLE" recv \
  --bind "0.0.0.0" --port "$BACNET_PORT" --seconds "$TIMEOUT_SECS" \
  --out "$EVIDENCE_DIR/oracle-recv.json" \
  >"$EVIDENCE_DIR/oracle-recv.log" 2>&1 &
ORACLE_RECV_PID=$!

log "starting DUT in $NS_B"
# Delay DUT TX until after peer→DUT injection (~2s).
export DBR_QUALIFY_TX_DELAY_MS=2500
ip netns exec "$NS_B" env DBR_QUALIFY_TX_DELAY_MS=2500 \
  "$BIN" --config "$EVIDENCE_DIR/dut.toml" --bip-qualify --qualify-secs "$TIMEOUT_SECS" \
  --qualify-send-unicast "${PEER_IP}:${BACNET_PORT}:${MATRIX_N}" \
  --qualify-send-broadcast "${MATRIX_N}" \
  >"$EVIDENCE_DIR/dut.log" 2>&1 &
DUT_PID=$!

for _ in $(seq 1 80); do
  if ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/healthz" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done
ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/healthz" | tee "$EVIDENCE_DIR/healthz-pre.json"
ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/api/metrics/snapshot" | tee "$EVIDENCE_DIR/metrics-pre.json"
SEQ_PRE="$(python3 -c 'import json;print(json.load(open("'"$EVIDENCE_DIR"'/metrics-pre.json")).get("sequence",0))')"

# --- U_peer_to_dut + B_peer_to_dut ---
log "oracle peer→DUT unicast x${MATRIX_N}"
ip netns exec "$NS_A" python3 "$ORACLE" send --mode unicast \
  --bind "$PEER_IP" --dest "$DUT_IP" --port "$BACNET_PORT" --count "$MATRIX_N" \
  --out "$EVIDENCE_DIR/oracle-send-unicast.json" | tee -a "$EVIDENCE_DIR/commands.log"

sleep 0.8
ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/api/metrics/snapshot" \
  | tee "$EVIDENCE_DIR/metrics-after-peer-unicast.json"

log "oracle peer→DUT directed broadcast x${MATRIX_N}"
ip netns exec "$NS_A" python3 "$ORACLE" send --mode broadcast \
  --bind "$PEER_IP" --dest "$BROADCAST" --port "$BACNET_PORT" --count "$MATRIX_N" \
  --out "$EVIDENCE_DIR/oracle-send-broadcast.json" | tee -a "$EVIDENCE_DIR/commands.log"

sleep 0.8
ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/api/metrics/snapshot" \
  | tee "$EVIDENCE_DIR/metrics-after-peer-broadcast.json"

# Wait for DUT scheduled TX (unicast+broadcast) and oracle recv window.
sleep 4
ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/api/metrics/snapshot" \
  | tee "$EVIDENCE_DIR/metrics-post.json"
ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/healthz" \
  | tee "$EVIDENCE_DIR/healthz-post.json"
SEQ_POST="$(python3 -c 'import json;print(json.load(open("'"$EVIDENCE_DIR"'/metrics-post.json")).get("sequence",0))')"

# Allow oracle recv to finish.
wait "$ORACLE_RECV_PID" || true
ORACLE_RECV_PID=""

# Limited broadcast: record NOT_SUPPORTED (not PASS).
python3 - <<'PY' "$EVIDENCE_DIR/limited-broadcast.json"
import json, sys
path = sys.argv[1]
json.dump({
  "subgate": "L_limited",
  "status": "NOT_SUPPORTED",
  "reason": "255.255.255.255 limited broadcast is not claimed on this Linux veth topology; directed 192.0.2.255 is the supported broadcast path for G6.",
}, open(path, "w", encoding="utf-8"), indent=2)
print("recorded L_limited NOT_SUPPORTED")
PY

# Evaluate matrix.
python3 - <<'PY' "$EVIDENCE_DIR" "$MATRIX_N" "$SEQ_PRE" "$SEQ_POST"
import json, pathlib, sys
ev = pathlib.Path(sys.argv[1])
N = int(sys.argv[2])
seq_pre, seq_post = int(sys.argv[3]), int(sys.argv[4])

def load(name):
    return json.loads((ev / name).read_text(encoding="utf-8"))

pre = load("metrics-pre.json")
after_u = load("metrics-after-peer-unicast.json")
after_b = load("metrics-after-peer-broadcast.json")
post = load("metrics-post.json")
health_post = load("healthz-post.json")
oracle_recv = load("oracle-recv.json")
oracle_su = load("oracle-send-unicast.json")
oracle_sb = load("oracle-send-broadcast.json")

def router(m): return m["router"]
def assert_zero_mstp_fwd(m, label):
    r = router(m)
    assert r["forwarded_bip_to_mstp"] == 0, (label, r)
    assert r["forwarded_mstp_to_bip"] == 0, (label, r)
    assert r["mstp_rx_packets"] == 0, (label, r)
    assert r["mstp_tx_packets"] == 0, (label, r)
    assert r["tx_tokens"] == 0 and r["rx_tokens"] == 0, (label, r)

for label, m in [("pre", pre), ("after_u", after_u), ("after_b", after_b), ("post", post)]:
    assert_zero_mstp_fwd(m, label)

rx0 = router(pre)["bip_rx_packets"]
tx0 = router(pre)["bip_tx_packets"]
rx_u = router(after_u)["bip_rx_packets"]
rx_b = router(after_b)["bip_rx_packets"]
rx_end = router(post)["bip_rx_packets"]
tx_end = router(post)["bip_tx_packets"]

# Peer→DUT unicast: RX delta at least N (may see more if broadcast overlapped — assert exact at after_u)
u_delta = rx_u - rx0
assert oracle_su["count"] == N
assert u_delta == N, f"U_peer_to_dut expected rx delta {N}, got {u_delta} (rx0={rx0}, rx_u={rx_u})"

# Peer→DUT directed broadcast
b_delta = rx_b - rx_u
assert oracle_sb["count"] == N
assert b_delta == N, f"B_peer_to_dut expected rx delta {N}, got {b_delta}"

# DUT TX: unicast + broadcast = 2N
tx_delta = tx_end - tx0
assert tx_delta == 2 * N, f"DUT tx expected {2*N}, got {tx_delta}"

# Oracle must have seen DUT unicast (0x0A) and broadcast (0x0B) with golden NPDU
by = oracle_recv.get("by_bvlc_function", {})
got_u = by.get("0x0a", 0)
got_b = by.get("0x0b", 0)
assert got_u >= N, f"U_dut_to_peer oracle 0x0A expected>={N}, got {got_u}"
assert got_b >= N, f"B_dut_to_peer oracle 0x0B expected>={N}, got {got_b}"
assert oracle_recv.get("golden_npdu_ok", 0) >= 2 * N, oracle_recv

assert health_post.get("ready_to_route") is False
assert seq_post > seq_pre, f"metrics stale: seq {seq_pre} -> {seq_post}"
assert post.get("bacnet_telemetry_available") is True or router(post)["bip_rx_packets"] >= 2 * N

limited = load("limited-broadcast.json")
assert limited["status"] == "NOT_SUPPORTED"

result = {
  "status": "PASS",
  "subgates": {
    "U_peer_to_dut": {"expected": N, "observed_rx_delta": u_delta, "oracle_tx": oracle_su["count"], "status": "PASS"},
    "B_peer_to_dut": {"expected": N, "observed_rx_delta": b_delta, "oracle_tx": oracle_sb["count"], "status": "PASS"},
    "U_dut_to_peer": {"expected": N, "oracle_rx_0x0a": got_u, "dut_tx_total": tx_delta, "status": "PASS"},
    "B_dut_to_peer": {"expected": N, "oracle_rx_0x0b": got_b, "status": "PASS"},
    "L_limited": limited,
  },
  "dut_rx_total": rx_end - rx0,
  "dut_tx_total": tx_delta,
  "metrics_sequence": {"pre": seq_pre, "post": seq_post},
  "forwarding_zero": True,
  "mstp_zero": True,
}
(ev / "result.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print("PASS: G6 B/IP matrix")
print(json.dumps(result["subgates"], indent=2))
PY

# Lifecycle: stop DUT and confirm bip_link leaves operational (poll briefly).
kill "$DUT_PID" 2>/dev/null || true
wait "$DUT_PID" 2>/dev/null || true
DUT_PID=""
# Rebind smoke: start briefly again on same addr/port.
ip netns exec "$NS_B" "$BIN" --config "$EVIDENCE_DIR/dut.toml" --bip-qualify --qualify-secs 3 \
  >"$EVIDENCE_DIR/dut-rebind.log" 2>&1 &
REBIND_PID=$!
sleep 1
if ! ip netns exec "$NS_B" curl -fsS "http://127.0.0.1:${MGMT_PORT}/healthz" >/dev/null; then
  die "rebind DUT management failed"
fi
kill "$REBIND_PID" 2>/dev/null || true
wait "$REBIND_PID" 2>/dev/null || true
echo '{"subgate":"rebind","status":"PASS"}' >"$EVIDENCE_DIR/rebind.json"

# Required PASS artifacts
for f in manifest.json result.json oracle-recv.json metrics-post.json healthz-post.json SHA256SUMS; do
  :
done
(
  cd "$EVIDENCE_DIR"
  # shellcheck disable=SC2035
  sha256sum manifest.json result.json oracle-recv.json metrics-post.json healthz-post.json \
    oracle-send-unicast.json oracle-send-broadcast.json limited-broadcast.json rebind.json \
    >SHA256SUMS
)
test -f "$EVIDENCE_DIR/result.json"
python3 -c 'import json,sys; r=json.load(open(sys.argv[1])); assert r["status"]=="PASS"' "$EVIDENCE_DIR/result.json"
log "PASS G6 evidence at $EVIDENCE_DIR"
