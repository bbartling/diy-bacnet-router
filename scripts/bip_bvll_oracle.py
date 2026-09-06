#!/usr/bin/env python3
"""Independent BACnet/IP BVLL oracle (stdlib only — not rusty-bacnet).

Modes:
  encode-golden   print golden NPDU hex (for harness alignment)
  self-test       golden packet encode/decode asserts (no sockets)
  send            send N Original-Unicast or Original-Broadcast NPDUs
  recv            receive bounded datagrams; emit JSONL + summary JSON
"""
from __future__ import annotations

import argparse
import hashlib
import json
import socket
import struct
import sys
import time
from typing import Any

BVLC_TYPE = 0x81
FN_ORIGINAL_UNICAST = 0x0A
FN_ORIGINAL_BROADCAST = 0x0B
# Minimal NPDU used by DIY BACnet Router M2A/G6 qualify probes.
GOLDEN_NPDU = bytes([0x01, 0x00, 0x10])
MAX_RECORDS = 256
MAX_DATAGRAM = 2048


def npdu_digest(npdu: bytes) -> str:
    return hashlib.sha256(npdu).hexdigest()


def encode_bvll(function: int, npdu: bytes) -> bytes:
    if function not in (FN_ORIGINAL_UNICAST, FN_ORIGINAL_BROADCAST):
        raise ValueError(f"unsupported BVLC function 0x{function:02x}")
    total = 4 + len(npdu)
    if total > 0xFFFF:
        raise ValueError("BVLL frame too large")
    return bytes([BVLC_TYPE, function]) + struct.pack("!H", total) + npdu


def decode_bvll(data: bytes) -> dict[str, Any]:
    if len(data) < 4:
        raise ValueError("truncated BVLL header")
    bvlc_type, function, length = data[0], data[1], struct.unpack("!H", data[2:4])[0]
    if bvlc_type != BVLC_TYPE:
        raise ValueError(f"bad BVLC type 0x{bvlc_type:02x}")
    if length != len(data):
        # Accept declared length if payload matches; reject mismatch with wire size.
        if length > len(data):
            raise ValueError(f"BVLC length {length} exceeds datagram {len(data)}")
        data = data[:length]
    if function not in (FN_ORIGINAL_UNICAST, FN_ORIGINAL_BROADCAST):
        raise ValueError(f"unsupported BVLC function 0x{function:02x}")
    if length < 4:
        raise ValueError("BVLC length too small")
    npdu = data[4:]
    if 4 + len(npdu) != length:
        raise ValueError("BVLC length does not match NPDU")
    return {
        "bvlc_type": bvlc_type,
        "bvlc_function": function,
        "bvlc_length": length,
        "npdu_hex": npdu.hex(),
        "npdu_sha256": npdu_digest(npdu),
        "npdu_matches_golden": npdu == GOLDEN_NPDU,
    }


def self_test() -> None:
    for fn in (FN_ORIGINAL_UNICAST, FN_ORIGINAL_BROADCAST):
        frame = encode_bvll(fn, GOLDEN_NPDU)
        assert frame[0] == BVLC_TYPE
        assert frame[1] == fn
        assert struct.unpack("!H", frame[2:4])[0] == len(frame)
        decoded = decode_bvll(frame)
        assert decoded["bvlc_function"] == fn
        assert decoded["npdu_matches_golden"] is True
        assert decoded["npdu_sha256"] == npdu_digest(GOLDEN_NPDU)
    # Truncated header must fail
    try:
        decode_bvll(b"\x81\x0a")
        raise AssertionError("expected truncated failure")
    except ValueError:
        pass
    # Bad type must fail
    try:
        decode_bvll(b"\x80\x0a\x00\x07" + GOLDEN_NPDU)
        raise AssertionError("expected bad type failure")
    except ValueError:
        pass
    print("PASS: bip_bvll_oracle self-test")


def cmd_send(args: argparse.Namespace) -> int:
    fn = FN_ORIGINAL_UNICAST if args.mode == "unicast" else FN_ORIGINAL_BROADCAST
    frame = encode_bvll(fn, GOLDEN_NPDU)
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        if args.mode == "broadcast":
            sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
        sock.bind((args.bind, args.bind_port))
        dest = (args.dest, args.port)
        for i in range(args.count):
            sock.sendto(frame, dest)
            time.sleep(args.interval_ms / 1000.0)
        summary = {
            "mode": "send",
            "bvlc_function": fn,
            "count": args.count,
            "dest": list(dest),
            "bind": [args.bind, args.bind_port],
            "npdu_sha256": npdu_digest(GOLDEN_NPDU),
            "frame_len": len(frame),
        }
        path = args.out or "-"
        text = json.dumps(summary, indent=2) + "\n"
        if path == "-":
            sys.stdout.write(text)
        else:
            with open(path, "w", encoding="utf-8") as fh:
                fh.write(text)
        return 0
    finally:
        sock.close()


def cmd_recv(args: argparse.Namespace) -> int:
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    records: list[dict[str, Any]] = []
    try:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
        sock.bind((args.bind, args.port))
        sock.settimeout(0.25)
        deadline = time.monotonic() + args.seconds
        while time.monotonic() < deadline and len(records) < MAX_RECORDS:
            try:
                data, addr = sock.recvfrom(MAX_DATAGRAM)
            except socket.timeout:
                continue
            except OSError as exc:
                records.append({"error": f"recv_oserror:{exc}"})
                break
            try:
                decoded = decode_bvll(data)
                decoded.update(
                    {
                        "src_ip": addr[0],
                        "src_port": addr[1],
                        "dst_bind": args.bind,
                        "dst_port": args.port,
                        "wire_len": len(data),
                    }
                )
                records.append(decoded)
            except ValueError as exc:
                records.append(
                    {
                        "error": f"decode:{exc}",
                        "src_ip": addr[0],
                        "src_port": addr[1],
                        "wire_len": len(data),
                        "wire_hex_prefix": data[:32].hex(),
                    }
                )
        by_fn: dict[str, int] = {}
        golden_ok = 0
        for rec in records:
            if "bvlc_function" in rec:
                key = f"0x{rec['bvlc_function']:02x}"
                by_fn[key] = by_fn.get(key, 0) + 1
                if rec.get("npdu_matches_golden"):
                    golden_ok += 1
        summary = {
            "mode": "recv",
            "bind": [args.bind, args.port],
            "seconds": args.seconds,
            "received": len(records),
            "by_bvlc_function": by_fn,
            "golden_npdu_ok": golden_ok,
            "npdu_sha256_expected": npdu_digest(GOLDEN_NPDU),
            "records": records[:MAX_RECORDS],
        }
        path = args.out or "-"
        text = json.dumps(summary, indent=2) + "\n"
        if path == "-":
            sys.stdout.write(text)
        else:
            with open(path, "w", encoding="utf-8") as fh:
                fh.write(text)
        return 0
    finally:
        sock.close()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_self = sub.add_parser("self-test")
    p_self.set_defaults(func=lambda _a: (self_test() or 0))

    p_enc = sub.add_parser("encode-golden")
    p_enc.set_defaults(
        func=lambda _a: (
            print(GOLDEN_NPDU.hex()),
            print(npdu_digest(GOLDEN_NPDU)),
            0,
        )[-1]
    )

    p_send = sub.add_parser("send")
    p_send.add_argument("--mode", choices=("unicast", "broadcast"), required=True)
    p_send.add_argument("--bind", default="0.0.0.0")
    p_send.add_argument("--bind-port", type=int, default=0)
    p_send.add_argument("--dest", required=True)
    p_send.add_argument("--port", type=int, default=47808)
    p_send.add_argument("--count", type=int, default=4)
    p_send.add_argument("--interval-ms", type=float, default=50.0)
    p_send.add_argument("--out", default="-")
    p_send.set_defaults(func=cmd_send)

    p_recv = sub.add_parser("recv")
    p_recv.add_argument("--bind", default="0.0.0.0")
    p_recv.add_argument("--port", type=int, default=47808)
    p_recv.add_argument("--seconds", type=float, default=5.0)
    p_recv.add_argument("--out", default="-")
    p_recv.set_defaults(func=cmd_recv)

    args = parser.parse_args()
    return int(args.func(args) or 0)


if __name__ == "__main__":
    sys.exit(main())
