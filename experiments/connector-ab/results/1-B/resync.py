#!/usr/bin/env python3
"""Run Forge resync handlers and write OUT/entities.ndjson (no Port SaaS)."""

from __future__ import annotations

import asyncio
import json
import os
import sys
from pathlib import Path
from typing import Any

from ocean_shim import ocean

# Registers @ocean.on_resync handlers.
import integration  # noqa: F401

OUT_PATH = Path(__file__).resolve().parent / "OUT" / "entities.ndjson"


def map_project(raw: dict[str, Any]) -> dict[str, Any]:
    key = raw["key"]
    return {
        "kind": "project",
        "id": key,
        "data": {
            "key": key,
            "name": raw.get("name", ""),
            "type": raw.get("projectTypeKey") or raw.get("type") or "",
        },
    }


def map_issue(raw: dict[str, Any]) -> dict[str, Any]:
    key = raw["key"]
    fields = raw.get("fields") or {}
    status_obj = fields.get("status") or {}
    status = status_obj.get("name") if isinstance(status_obj, dict) else status_obj
    project_obj = fields.get("project") or {}
    project_key = (
        project_obj.get("key") if isinstance(project_obj, dict) else project_obj
    )
    assignee_obj = fields.get("assignee")
    assignee = None
    if isinstance(assignee_obj, dict):
        assignee = assignee_obj.get("accountId")
    return {
        "kind": "issue",
        "id": key,
        "data": {
            "key": key,
            "summary": fields.get("summary", ""),
            "status": status or "",
            "project": project_key or "",
            "assignee": assignee,
        },
    }


async def _collect(kind: str, mapper) -> list[dict[str, Any]]:
    handler = ocean.resync_handlers.get(kind)
    if handler is None:
        raise RuntimeError(f"no @ocean.on_resync handler registered for kind={kind!r}")
    entities: list[dict[str, Any]] = []
    agen = handler(kind)
    async for batch in agen:
        for raw in batch:
            entities.append(mapper(raw))
    return entities


_KIND_ORDER = {"project": 0, "issue": 1, "component": 2}


async def run_resync() -> list[dict[str, Any]]:
    ocean.integration_config["baseUrl"] = os.environ.get(
        "FORGE_BASE_URL", "http://127.0.0.1:9743"
    )
    ocean.integration_config["token"] = os.environ.get("FORGE_TOKEN", "tenant-a-token")
    projects = await _collect("project", map_project)
    issues = await _collect("issue", map_issue)
    entities = projects + issues
    # Kind order matches the experiment schema example: project, issue, component.
    entities.sort(key=lambda e: (_KIND_ORDER.get(e["kind"], 99), e["id"]))
    return entities


def main() -> int:
    entities = asyncio.run(run_resync())
    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with OUT_PATH.open("w", encoding="utf-8") as fh:
        for entity in entities:
            fh.write(json.dumps(entity, separators=(",", ":"), ensure_ascii=False))
            fh.write("\n")
    print(f"wrote {len(entities)} entities to {OUT_PATH}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
