#!/usr/bin/env python3
"""Phase-1 Workbench-proxy: poll all five mini objects concurrently through the DIY router.

Objects on 2001:2 / device 123102 (object-list):
  device:123102 object-name
  analog-input:1 present-value
  analog-value:2 present-value
  binary-input:1 present-value
  binary-value:2 present-value

Example:
  /home/ben/env/bin/python scripts/phase1_five_point_soak.py --duration-secs 600
"""
from __future__ import annotations

import argparse
import asyncio
import json
import statistics
import sys
import time
from typing import Any


POINTS: list[tuple[str, str]] = [
    ("device:123102", "object-name"),
    ("analog-input:1", "present-value"),
    ("analog-value:2", "present-value"),
    ("binary-input:1", "present-value"),
    ("binary-value:2", "present-value"),
]


async def run(args: argparse.Namespace) -> dict[str, Any]:
    from bacpypes3.argparse import SimpleArgumentParser
    from bacpypes3.app import Application

    sys.argv = [
        "phase1_five_point_soak",
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

    per_point: dict[str, list[dict[str, Any]]] = {f"{o}.{p}": [] for o, p in POINTS}
    t_end = time.monotonic() + args.duration_secs
    round_i = 0

    async def one(obj: str, prop: str) -> dict[str, Any]:
        t0 = time.monotonic()
        key = f"{obj}.{prop}"
        try:
            val = await asyncio.wait_for(
                app.read_property(args.target, obj, prop),
                timeout=args.timeout,
            )
            row = {
                "ok": True,
                "ms": round((time.monotonic() - t0) * 1000, 1),
                "val": str(val)[:80],
            }
        except Exception as exc:  # noqa: BLE001
            row = {
                "ok": False,
                "ms": round((time.monotonic() - t0) * 1000, 1),
                "err": f"{type(exc).__name__}: {exc}",
            }
        per_point[key].append(row)
        return row

    while time.monotonic() < t_end:
        round_i += 1
        await asyncio.gather(*[one(o, p) for o, p in POINTS])
        await asyncio.sleep(args.interval)

    def summarize(rows: list[dict[str, Any]]) -> dict[str, Any]:
        oks = [r for r in rows if r["ok"]]
        fails = [r for r in rows if not r["ok"]]
        ms = sorted(r["ms"] for r in oks)
        return {
            "n": len(rows),
            "ok": len(oks),
            "fail": len(fails),
            "p50": ms[len(ms) // 2] if ms else None,
            "p95": ms[min(len(ms) - 1, int(len(ms) * 0.95))] if ms else None,
            "p99": ms[min(len(ms) - 1, int(len(ms) * 0.99))] if ms else None,
            "max": max(ms) if ms else None,
            "mean": round(statistics.fmean(ms), 1) if ms else None,
        }

    summary = {k: summarize(v) for k, v in per_point.items()}
    all_ms = sorted(
        r["ms"] for rows in per_point.values() for r in rows if r["ok"]
    )
    overall = {
        "rounds": round_i,
        "duration_secs": args.duration_secs,
        "interval": args.interval,
        "ok": sum(s["ok"] for s in summary.values()),
        "fail": sum(s["fail"] for s in summary.values()),
        "p50": all_ms[len(all_ms) // 2] if all_ms else None,
        "p95": all_ms[min(len(all_ms) - 1, int(len(all_ms) * 0.95))]
        if all_ms
        else None,
        "p99": all_ms[min(len(all_ms) - 1, int(len(all_ms) * 0.99))]
        if all_ms
        else None,
        "max": max(all_ms) if all_ms else None,
    }
    app.close()
    return {
        "target": args.target,
        "points": [f"{o}.{p}" for o, p in POINTS],
        "per_point": summary,
        "overall": overall,
        "slo": {
            "p95_ms_max": args.slo_p95,
            "max_ms_max": args.slo_max,
            "zero_fails": True,
            "pass": overall["fail"] == 0
            and (overall["p95"] is not None and overall["p95"] <= args.slo_p95)
            and (overall["max"] is not None and overall["max"] <= args.slo_max),
        },
    }


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--address", default="192.168.204.11/24")
    p.add_argument("--network", type=int, default=1)
    p.add_argument("--instance", type=int, default=599993)
    p.add_argument("--name", default="Phase1FivePoint")
    p.add_argument("--target", default="2001:2")
    p.add_argument("--duration-secs", type=float, default=600.0)
    p.add_argument("--interval", type=float, default=1.0)
    p.add_argument("--timeout", type=float, default=10.0)
    p.add_argument("--slo-p95", type=float, default=250.0)
    p.add_argument("--slo-max", type=float, default=2000.0)
    p.add_argument("--json-out", default="")
    args = p.parse_args()
    out = asyncio.run(run(args))
    text = json.dumps(out, indent=2)
    if args.json_out:
        open(args.json_out, "w", encoding="utf-8").write(text + "\n")
    print(text)
    print("OVERALL", out["overall"], "SLO", out["slo"], file=sys.stderr)
    return 0 if out["slo"]["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
