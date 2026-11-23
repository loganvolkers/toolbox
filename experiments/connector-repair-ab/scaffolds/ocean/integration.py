"""Ocean-style Forge integration: project, issue, and component resync handlers.

Uses a local ocean_shim because the pinned Port Ocean package needs fastapi
and other deps that are not installed in this harness. Handler shape matches
integrations/jira/main.py and integrations/fake-integration/main.py
(@ocean.on_resync per kind, including component).
"""

from __future__ import annotations

import os
from typing import Any, AsyncIterator

from forge_client import ForgeClient
from ocean_shim import ocean

# Config: env overrides, else tenant A defaults from FORGE.md
DEFAULT_BASE_URL = "http://127.0.0.1:9743"
DEFAULT_TOKEN = "tenant-a-token"


def get_client() -> ForgeClient:
    base_url = os.environ.get("FORGE_BASE_URL") or os.environ.get(
        "OCEAN__INTEGRATION__CONFIG__BASEURL", DEFAULT_BASE_URL
    )
    token = os.environ.get("FORGE_TOKEN") or os.environ.get(
        "OCEAN__INTEGRATION__CONFIG__TOKEN", DEFAULT_TOKEN
    )
    return ForgeClient(base_url, token)


@ocean.on_resync("project")
async def on_resync_projects(kind: str) -> AsyncIterator[list[dict[str, Any]]]:
    client = get_client()
    async for batch in client.get_paginated_projects():
        yield batch


@ocean.on_resync("issue")
async def on_resync_issues(kind: str) -> AsyncIterator[list[dict[str, Any]]]:
    client = get_client()
    async for batch in client.get_paginated_issues():
        yield batch


@ocean.on_resync("component")
async def on_resync_components(kind: str) -> AsyncIterator[list[dict[str, Any]]]:
    """Mirror jira/main.py on_resync_components: per-project component fetch."""
    client = get_client()
    async for batch in client.get_paginated_components():
        yield batch


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
    fields = raw.get("fields") or {}
    key = raw["key"]
    status_obj = fields.get("status") or {}
    project_obj = fields.get("project") or {}
    assignee_obj = fields.get("assignee")
    assignee = None
    if isinstance(assignee_obj, dict):
        assignee = assignee_obj.get("accountId")
    elif isinstance(assignee_obj, str):
        assignee = assignee_obj
    return {
        "kind": "issue",
        "id": key,
        "data": {
            "key": key,
            "summary": fields.get("summary", ""),
            "status": status_obj.get("name", "") if isinstance(status_obj, dict) else str(status_obj or ""),
            "project": project_obj.get("key", "") if isinstance(project_obj, dict) else str(project_obj or ""),
            "assignee": None,
        },
    }


def map_component(raw: dict[str, Any]) -> dict[str, Any]:
    # Catalog id is {project}:{name}; Forge numeric id lives only in data.id.
    forge_id = str(raw["id"])
    name = raw.get("name", "")
    project = raw.get("project") or ""
    if isinstance(project, dict):
        project = project.get("key", "")
    project = str(project or "")
    return {
        "kind": "component",
        "id": name,
        "data": {
            "id": forge_id,
            "name": name,
            "project": project,
            "description": raw.get("description") or "",
        },
    }


MAPPERS = {
    "project": map_project,
    "issue": map_issue,
    "component": map_component,
}
