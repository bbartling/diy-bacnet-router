#!/usr/bin/env python3
"""Issue #66 controlled BIP ReadProperty soak through DIY router → DNET 2001/MAC 2.

Requires bacpypes3. Example:
  /path/to/.venv-bacpypes3/bin/python scripts/issue66_rp_soak.py \\
    --address 192.168.204.11/24 --target 2001:2 --count 30 --burst 5
"""
from __future__ import annotations

import argparse
import asyncio
import json
import sys
import time
from typing import Any


async def run(args: argparse.Namespace) -> dict[str, Any]:
    from bacpypes3.argparse import SimpleArgumentParser
    from bacpypes3.app import Application

    sys.argv = [
        "issue66_rp_soak",
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

    async def one(i: int) -> dict[str, Any]:
        t0 = time.monotonic()
        try:
            val = await asyncio.wait_for(
                app.read_property(args.target, args.object, args.property),
                timeout=args.timeout,
            )
            return {
                "i": i,
                "ok": True,
                "ms": round((time.monotonic() - t0) * 1000, 1),
                "val": str(val),
            }
        except Exception as exc:  # noqa: BLE001 — soak oracle
            return {
                "i": i,
                "ok": False,
                "ms": round((time.monotonic() - t0) * 1000, 1),
                "err": f"{type(exc).__name__}: {exc}",
            }

    probe = await one(-1)
    burst = await asyncio.gather(*[one(i) for i in range(args.burst)])
    sequential = [await one(i) for i in range(args.count)]
    ms = [r["ms"] for r in sequential if r["ok"]]
    ms_sorted = sorted(ms)
    summary = {
        "count": args.count,
        "ok": sum(1 for r in sequential if r["ok"]),
        "fail": sum(1 for r in sequential if not r["ok"]),
        "p50": ms_sorted[len(ms_sorted) // 2] if ms_sorted else None,
        "p95": ms_sorted[min(len(ms_sorted) - 1, int(len(ms_sorted) * 0.95))]
        if ms_sorted
        else None,
        "max": max(ms_sorted) if ms_sorted else None,
    }
    app.close()
    return {
        "target": args.target,
        "object": args.object,
        "property": args.property,
        "probe": probe,
        "burst": burst,
        "sequential": sequential,
        "summary": summary,
    }


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--address", default="192.168.204.11/24")
    p.add_argument("--network", type=int, default=1)
    p.add_argument("--instance", type=int, default=599991)
    p.add_argument("--name", default="Issue66Soak")
    p.add_argument("--target", default="2001:2")
    p.add_argument("--object", default="analog-input:1")
    p.add_argument("--property", default="present-value")
    p.add_argument("--count", type=int, default=30)
    p.add_argument("--burst", type=int, default=5)
    p.add_argument("--timeout", type=float, default=10.0)
    p.add_argument("--json-out", default="")
    args = p.parse_args()
    out = asyncio.run(run(args))
    text = json.dumps(out, indent=2)
    if args.json_out:
        open(args.json_out, "w", encoding="utf-8").write(text + "\n")
    print(text)
    print("SUMMARY", out["summary"], file=sys.stderr)
    return 0 if out["summary"]["fail"] == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
