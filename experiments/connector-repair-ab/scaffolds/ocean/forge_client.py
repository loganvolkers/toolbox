"""Minimal Forge (Jira Cloud-shaped) HTTP client using the stdlib.

Forge endpoints (FORGE.md) differ slightly from live Jira Cloud Ocean uses
(`/rest/api/3/project` and `/rest/api/3/search`, Bearer auth).
"""

from __future__ import annotations

import json
import urllib.error
import urllib.parse
import urllib.request
from typing import Any, AsyncIterator, Iterator


PAGE_SIZE = 50


class ForgeClient:
    def __init__(self, base_url: str, token: str) -> None:
        self.base_url = base_url.rstrip("/")
        self.token = token

    def _get_json(self, path: str, params: dict[str, Any] | None = None) -> Any:
        if params:
            query = urllib.parse.urlencode(params)
            url = f"{self.base_url}{path}?{query}"
        else:
            url = f"{self.base_url}{path}"
        req = urllib.request.Request(
            url,
            headers={
                "Authorization": f"Bearer {self.token}",
                "Accept": "application/json",
            },
            method="GET",
        )
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                return json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            body = e.read().decode("utf-8", errors="replace")
            raise RuntimeError(f"Forge GET {path} failed: {e.code} {body}") from e

    def _get(self, path: str, params: dict[str, Any]) -> dict[str, Any]:
        data = self._get_json(path, params)
        if not isinstance(data, dict):
            raise RuntimeError(f"Forge GET {path}: expected object, got {type(data).__name__}")
        return data

    def iter_projects(self) -> Iterator[list[dict[str, Any]]]:
        start_at = 0
        while True:
            data = self._get(
                "/rest/api/3/project",
                {"startAt": start_at, "maxResults": PAGE_SIZE},
            )
            values = data.get("values") or []
            if values:
                yield values
            if data.get("isLast", True) or not values:
                break
            start_at = int(data.get("startAt", start_at)) + len(values)

    def iter_issues(self) -> Iterator[list[dict[str, Any]]]:
        start_at = 0
        while True:
            data = self._get(
                "/rest/api/3/search",
                {"startAt": start_at, "maxResults": PAGE_SIZE},
            )
            issues = data.get("issues") or []
            if issues:
                yield issues
            total = int(data.get("total", 0))
            start_at = int(data.get("startAt", start_at)) + len(issues)
            if start_at >= total or not issues:
                break

    def get_project_components(self, project_key: str) -> list[dict[str, Any]]:
        """Forge: GET /rest/api/3/project/{key}/components — bare JSON array."""
        data = self._get_json(f"/rest/api/3/project/{project_key}/components")
        if not isinstance(data, list):
            raise RuntimeError(
                f"Forge GET /project/{project_key}/components: "
                f"expected array, got {type(data).__name__}"
            )
        enriched: list[dict[str, Any]] = []
        for item in data:
            if not isinstance(item, dict):
                continue
            row = dict(item)
            row["project"] = project_key
            enriched.append(row)
        return enriched

    def iter_components(self) -> Iterator[list[dict[str, Any]]]:
        for project_batch in self.iter_projects():
            for project in project_batch:
                key = project.get("key")
                if not key:
                    continue
                comps = self.get_project_components(str(key))
                if comps:
                    yield comps

    async def get_paginated_projects(self) -> AsyncIterator[list[dict[str, Any]]]:
        for batch in self.iter_projects():
            yield batch

    async def get_paginated_issues(self) -> AsyncIterator[list[dict[str, Any]]]:
        for batch in self.iter_issues():
            yield batch

    async def get_paginated_components(self) -> AsyncIterator[list[dict[str, Any]]]:
        for batch in self.iter_components():
            yield batch
