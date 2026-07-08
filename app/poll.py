"""Background BACnet poll engine.

Periodically reads present-value of every enabled configured field point and
keeps a rolling in-memory sample buffer plus a last-value cache. This is the
commissioning/soak workhorse for an Open-FDD field-bus sidecar: the sidecar
owns all polling of the field bus so the FDD app only ever talks JSON.

Exposed via /bacnet/poll/status and /bacnet/poll/once.
"""

from __future__ import annotations

import asyncio
import logging
import time
from collections import deque
from typing import Any

from app.bacnet_client import BacnetClientService
from app.config import Settings

logger = logging.getLogger(__name__)


class PollEngine:
    def __init__(self, settings: Settings, client: BacnetClientService):
        self.settings = settings
        self.client = client
        self._task: asyncio.Task | None = None
        self._stop = asyncio.Event()
        self._samples: deque[dict[str, Any]] = deque(maxlen=settings.poll.max_samples)
        self._last: dict[str, dict[str, Any]] = {}
        self._cycles = 0
        self._errors = 0
        self._last_cycle_ts: float | None = None
        self._last_cycle_duration: float | None = None
        self._last_error: str | None = None

    # ---- lifecycle -------------------------------------------------------

    def start(self) -> None:
        if not self.settings.poll.enabled:
            logger.info("poll engine disabled (poll.enabled=false)")
            return
        if self._task is not None:
            return
        self._stop.clear()
        self._task = asyncio.create_task(self._run(), name="bacnet-poll")
        logger.info(
            "poll engine started (interval=%ss)", self.settings.poll.interval_secs
        )

    async def stop(self) -> None:
        self._stop.set()
        if self._task is not None:
            self._task.cancel()
            try:
                await self._task
            except (asyncio.CancelledError, Exception):
                pass
            self._task = None

    async def _run(self) -> None:
        try:
            await asyncio.sleep(self.settings.poll.startup_delay_secs)
        except asyncio.CancelledError:
            return
        while not self._stop.is_set():
            try:
                await self.poll_once()
            except asyncio.CancelledError:
                return
            except Exception as e:  # keep the loop alive across transient failures
                self._errors += 1
                self._last_error = str(e)
                logger.warning("poll cycle error: %s", e)
            try:
                await asyncio.wait_for(
                    self._stop.wait(), timeout=self.settings.poll.interval_secs
                )
            except asyncio.TimeoutError:
                pass

    # ---- one cycle -------------------------------------------------------

    async def poll_once(self) -> dict[str, Any]:
        started = time.time()
        rows = await self.client.poll_points()
        duration = time.time() - started
        self._cycles += 1
        self._last_cycle_ts = started
        self._last_cycle_duration = duration
        errored = 0
        for row in rows:
            key = f"{row['device_instance']}:{row['object_type']},{row['object_instance']}"
            self._last[key] = row
            self._samples.append(row)
            if row.get("error"):
                errored += 1
        return {
            "points_polled": len(rows),
            "points_errored": errored,
            "duration_secs": round(duration, 3),
            "cycle": self._cycles,
        }

    # ---- status ----------------------------------------------------------

    def status(self) -> dict[str, Any]:
        last_values = list(self._last.values())
        healthy = sum(1 for v in last_values if not v.get("error"))
        return {
            "enabled": self.settings.poll.enabled,
            "running": self._task is not None and not self._task.done(),
            "interval_secs": self.settings.poll.interval_secs,
            "cycles_completed": self._cycles,
            "cycle_errors": self._errors,
            "last_error": self._last_error,
            "last_cycle_ts": self._last_cycle_ts,
            "last_cycle_duration_secs": (
                round(self._last_cycle_duration, 3)
                if self._last_cycle_duration is not None
                else None
            ),
            "points_tracked": len(last_values),
            "points_healthy": healthy,
            "samples_buffered": len(self._samples),
            "last_values": last_values,
        }
