"""Read-only Haystack client wrapper (rusty_haystack SCRAM)."""

from __future__ import annotations

from typing import Any

READ_ONLY_OPS = frozenset({"about", "ops", "formats", "read", "nav", "his_read", "defs", "libs"})


class HaystackNotAllowedError(PermissionError):
    pass


class HaystackClientService:
    """Enforces read-only Haystack ops before calling rusty_haystack."""

    def __init__(self, base_url: str, username: str, password: str):
        self.base_url = base_url.rstrip("/")
        self.username = username
        self.password = password
        self._client = None

    def _connect(self):
        if self._client is not None:
            return self._client
        from rusty_haystack.client import HaystackClient

        self._client = HaystackClient.connect(self.base_url, self.username, self.password)
        return self._client

    def close(self) -> None:
        if self._client is not None:
            try:
                self._client.close()
            except Exception:
                pass
            self._client = None

    def _check_op(self, op: str) -> None:
        if op not in READ_ONLY_OPS:
            raise HaystackNotAllowedError(f"Haystack op '{op}' is not allowlisted (read-only gateway)")

    def about(self) -> Any:
        self._check_op("about")
        return self._connect().about()

    def read(self, filter_expr: str) -> Any:
        self._check_op("read")
        return self._connect().read(filter_expr)

    def nav(self, nav_id: str | None = None) -> Any:
        self._check_op("nav")
        if nav_id:
            return self._connect().nav(nav_id)
        return self._connect().nav()

    def his_read(
        self,
        ids: list[str],
        range_start: str | None = None,
        range_end: str | None = None,
    ) -> Any:
        self._check_op("his_read")
        client = self._connect()
        if range_start and range_end:
            return client.his_read(ids, range_start, range_end)
        return client.his_read(ids)
