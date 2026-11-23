"""Ocean-style Forge integration: project and issue resync handlers.

Uses a local `ocean_shim` because the Ocean pin's `port_ocean` package needs
fastapi and other deps that are not installed in this arm. Handlers still use
the `@ocean.on_resync` shape from integrations/jira and fake-integration.
"""

from __future__ import annotations

import os
from collections.abc import AsyncIterator
from typing import Any

from forge_client import ForgeClient
from ocean_shim import ocean

KindBatch = AsyncIterator[list[dict[str, Any]]]


def _client() -> ForgeClient:
    base_url = (
        ocean.integration_config.get("baseUrl")
        or os.environ.get("FORGE_BASE_URL")
        or "http://127.0.0.1:9743"
    )
    token = (
        ocean.integration_config.get("token")
        or os.environ.get("FORGE_TOKEN")
        or "tenant-a-token"
    )
    return ForgeClient(base_url, token)


@ocean.on_resync("project")
async def on_resync_projects(kind: str) -> KindBatch:
    client = _client()
    for batch in client.iter_projects():
        yield batch


@ocean.on_resync("issue")
async def on_resync_issues(kind: str) -> KindBatch:
    client = _client()
    for batch in client.iter_issues():
        yield batch
