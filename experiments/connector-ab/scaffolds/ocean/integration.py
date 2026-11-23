"""Scaffold: Ocean-style Forge integration.

Implement `@ocean.on_resync` handlers for project and issue (and component from
phase 3). You may import from the Ocean pin listed in OCEAN_ROOT.txt for types
and patterns (read fake-integration and jira only).

Because this benchmark does not talk to Port SaaS, also provide `resync.py` that
calls your handlers against Forge and writes OUT/entities.ndjson.
"""

# from port_ocean.context.ocean import ocean
# @ocean.on_resync("project")
# async def on_resync_projects(kind: str):
#     ...
#     yield batch
