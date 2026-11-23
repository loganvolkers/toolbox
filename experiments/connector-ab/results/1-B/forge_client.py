"""Minimal Forge (Jira Cloud-shaped) HTTP client using stdlib only."""

from __future__ import annotations

import json
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Iterator
from typing import Any

PAGE_SIZE = 50


class ForgeClient:
    def __init__(self, base_url: str, token: str) -> None:
        self.base_url = base_url.rstrip("/")
        self.token = token

    def _get(self, path: str, params: dict[str, Any]) -> dict[str, Any]:
        query = urllib.parse.urlencode(params)
        url = f"{self.base_url}{path}?{query}"
        req = urllib.request.Request(
            url,
            headers={"Authorization": f"Bearer {self.token}", "Accept": "application/json"},
            method="GET",
        )
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                return json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as exc:
            body = exc.read().decode("utf-8", errors="replace")
            raise RuntimeError(f"Forge GET {url} -> {exc.code}: {body}") from exc

    def iter_projects(self) -> Iterator[list[dict[str, Any]]]:
        start_at = 0
        while True:
            payload = self._get(
                "/rest/api/3/project",
                {"startAt": start_at, "maxResults": PAGE_SIZE},
            )
            values = payload.get("values") or []
            if values:
                yield values
            if payload.get("isLast", True) or not values:
                break
            start_at += len(values)

    def iter_issues(self) -> Iterator[list[dict[str, Any]]]:
        start_at = 0
        while True:
            payload = self._get(
                "/rest/api/3/search",
                {"startAt": start_at, "maxResults": PAGE_SIZE},
            )
            issues = payload.get("issues") or []
            if issues:
                yield issues
            total = int(payload.get("total", 0))
            start_at += len(issues)
            if not issues or start_at >= total:
                break
