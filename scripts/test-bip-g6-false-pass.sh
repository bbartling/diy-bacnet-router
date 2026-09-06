#!/usr/bin/env bash
# False-PASS regressions for G6 result checker (no root/netns required).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

checker='
import json, pathlib, sys
ev = pathlib.Path(sys.argv[1])
N = int(sys.argv[2])
def load(n): return json.loads((ev/n).read_text())
pre, after_u, after_b, post = map(load, [
  "metrics-pre.json","metrics-after-peer-unicast.json",
  "metrics-after-peer-broadcast.json","metrics-post.json"])
oracle_recv = load("oracle-recv.json")
def rx(m): return m["router"]["bip_rx_packets"]
def tx(m): return m["router"]["bip_tx_packets"]
u_delta = rx(after_u)-rx(pre)
b_delta = rx(after_b)-rx(after_u)
tx_delta = tx(post)-tx(pre)
assert u_delta == N
assert b_delta == N
assert tx_delta == 2*N
by = oracle_recv.get("by_bvlc_function", {})
assert by.get("0x0a",0) >= N
assert by.get("0x0b",0) >= N
print("checker_ok")
'

write_base() {
  python3 - <<'PY' "$tmpdir"
import json, pathlib, sys
ev = pathlib.Path(sys.argv[1])
def w(name, obj): (ev/name).write_text(json.dumps(obj)+"\n")
base_r = dict(bip_rx_packets=0,bip_tx_packets=0,mstp_rx_packets=0,mstp_tx_packets=0,
  forwarded_bip_to_mstp=0,forwarded_mstp_to_bip=0,tx_tokens=0,rx_tokens=0,
  dropped_packets=0,invalid_frames=0,header_crc_errors=0,data_crc_errors=0,
  apdu_timeouts=0,serial_reconnects=0,tx_poll_for_master=0,rx_poll_for_master=0,event_count=0)
def metrics(rx, tx, seq=1):
  return {"sequence": seq, "bacnet_telemetry_available": True, "router": {**base_r, "bip_rx_packets": rx, "bip_tx_packets": tx}}
w("metrics-pre.json", metrics(0,0,1))
w("metrics-after-peer-unicast.json", metrics(4,0,2))
w("metrics-after-peer-broadcast.json", metrics(8,0,3))
w("metrics-post.json", metrics(8,8,5))
w("oracle-recv.json", {"by_bvlc_function":{"0x0a":4,"0x0b":4},"golden_npdu_ok":8})
print("base written")
PY
}

fail_case() {
  local name="$1"
  shift
  write_base
  "$@"
  if python3 -c "$checker" "$tmpdir" 4 >/dev/null 2>&1; then
    echo "FALSE PASS: $name should have failed" >&2
    exit 1
  fi
  echo "PASS: caught false scenario $name"
}

write_base
python3 -c "$checker" "$tmpdir" 4 >/dev/null
echo "PASS: happy path checker"

# zero RX
fail_case zero_rx python3 - <<'PY' "$tmpdir"
import json, pathlib, sys
p=pathlib.Path(sys.argv[1])/"metrics-after-peer-unicast.json"
m=json.loads(p.read_text()); m["router"]["bip_rx_packets"]=0; p.write_text(json.dumps(m))
PY

# no TX
fail_case no_tx python3 - <<'PY' "$tmpdir"
import json, pathlib, sys
p=pathlib.Path(sys.argv[1])/"metrics-post.json"
m=json.loads(p.read_text()); m["router"]["bip_tx_packets"]=0; p.write_text(json.dumps(m))
PY

# only unicast mode (broadcast delta 0)
fail_case single_mode python3 - <<'PY' "$tmpdir"
import json, pathlib, sys
p=pathlib.Path(sys.argv[1])/"metrics-after-peer-broadcast.json"
m=json.loads(p.read_text()); m["router"]["bip_rx_packets"]=4; p.write_text(json.dumps(m))
PY

# stale oracle (missing broadcast RX)
fail_case stale_oracle python3 - <<'PY' "$tmpdir"
import json, pathlib, sys
p=pathlib.Path(sys.argv[1])/"oracle-recv.json"
p.write_text(json.dumps({"by_bvlc_function":{"0x0a":4,"0x0b":0},"golden_npdu_ok":4}))
PY

echo "PASS: G6 false-PASS regressions"
