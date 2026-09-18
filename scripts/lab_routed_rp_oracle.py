#!/usr/bin/env python3
"""Routed ReadProperty oracle via DIY BACnet router (bacpypes3 --route-aware).

Bind on a DIFFERENT IP than the router BIP socket (e.g. router .11, client .12).

Example (FEC on @38400):
  python scripts/lab_routed_rp_oracle.py \\
    --address 192.168.204.12/24 --router 192.168.204.11 \\
    --include-fec

Example (FEC off, minis only):
  python scripts/lab_routed_rp_oracle.py \\
    --address 192.168.204.12/24 --router 192.168.204.11
"""
from __future__ import annotations

import argparse
import asyncio
import json
import sys
import time
from typing import Any


DEFAULT_READS: list[tuple[str, str, str, str]] = [
    ("mini2", "2001:2", "device:123102", "object-name"),
    ("mini2", "2001:2", "analog-input:1", "present-value"),
    ("mini3", "2001:3", "device:123103", "object-name"),
    ("mini3", "2001:3", "analog-input:1", "present-value"),
]

FEC_READS: list[tuple[str, str, str, str]] = [
    ("fec", "2001:7", "device:5007", "object-name"),
    ("fec", "2001:7", "analog-input:1173", "present-value"),
]


async def run(args: argparse.Namespace) -> dict[str, Any]:
    from bacpypes3.argparse import SimpleArgumentParser
    from bacpypes3.app import Application

    sys.argv = [
        "lab_routed_rp_oracle",
        "--route-aware",
        "--name",
        args.name,
        "--address",
        args.address,
        "--instance",
        str(args.instance),
        "--network",
        str(args.network),
    ]
    app = Application.from_args(SimpleArgumentParser().parse_args())
    out: dict[str, Any] = {"router": args.router, "address": args.address, "reads": []}

    try:
        routers = await asyncio.wait_for(
            app.nse.who_is_router_to_network(network=2001), timeout=5
        )
        out["whois_router"] = [str(r) for r in (routers or [])]
    except Exception as exc:  # noqa: BLE001
        out["whois_router"] = f"ERR {type(exc).__name__}:{exc}"

    probes = list(DEFAULT_READS)
    if args.include_fec:
        probes.extend(FEC_READS)

    for label, net_mac, obj, prop in probes:
        target = f"{net_mac}@{args.router}" if args.explicit_router else net_mac
        t0 = time.monotonic()
        try:
            val = await asyncio.wait_for(
                app.read_property(target, obj, prop), timeout=args.timeout
            )
            out["reads"].append(
                {
                    "label": label,
                    "target": target,
                    "obj": f"{obj}.{prop}",
                    "ok": True,
                    "ms": round((time.monotonic() - t0) * 1000, 1),
                    "val": str(val),
                }
            )
        except Exception as exc:  # noqa: BLE001
            out["reads"].append(
                {
                    "label": label,
                    "target": target,
                    "obj": f"{obj}.{prop}",
                    "ok": False,
                    "ms": round((time.monotonic() - t0) * 1000, 1),
                    "err": f"{type(exc).__name__}:{exc}",
                }
            )

    app.close()
    out["ok_count"] = sum(1 for r in out["reads"] if r["ok"])
    out["fail_count"] = sum(1 for r in out["reads"] if not r["ok"])
    out["all_ok"] = out["fail_count"] == 0
    return out


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--address", default="192.168.204.12/24")
    p.add_argument("--router", default="192.168.204.11")
    p.add_argument("--network", type=int, default=1)
    p.add_argument("--instance", type=int, default=599012)
    p.add_argument("--name", default="LabRoutedOracle")
    p.add_argument("--timeout", type=float, default=12.0)
    p.add_argument("--include-fec", action="store_true")
    p.add_argument(
        "--explicit-router",
        action="store_true",
        help="Use DNET:MAC@router form instead of relying on Who-Is-Router cache",
    )
    p.add_argument("-o", "--output", help="Write JSON to path")
    args = p.parse_args()
    result = asyncio.run(run(args))
    text = json.dumps(result, indent=2)
    print(text)
    if args.output:
        with open(args.output, "w", encoding="utf-8") as fh:
            fh.write(text + "\n")
    return 0 if result.get("all_ok") else 1


if __name__ == "__main__":
    raise SystemExit(main())
