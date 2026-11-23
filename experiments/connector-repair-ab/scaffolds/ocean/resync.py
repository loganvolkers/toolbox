#!/usr/bin/env python3
"""Run Forge resync without Port SaaS and write OUT/entities.ndjson."""

from __future__ import annotations

import asyncio
import json
import sys
from pathlib import Path
from typing import Any

_ROOT = Path(__file__).resolve().parent
if str(_ROOT) not in sys.path:
    sys.path.insert(0, str(_ROOT))

# Importing integration registers @ocean.on_resync handlers.
import integration  # noqa: F401
from integration import MAPPERS
from ocean_shim import registered_handlers


OUT_PATH = Path(__file__).resolve().parent / "OUT" / "entities.ndjson"
KINDS = ("project", "issue", "component")


async def collect_kind(kind: str) -> list[dict[str, Any]]:
    handlers = registered_handlers()
    handler = handlers.get(kind)
    if handler is None:
        raise RuntimeError(f"no @ocean.on_resync handler for kind={kind!r}")
    mapper = MAPPERS[kind]
    entities: list[dict[str, Any]] = []
    result = handler(kind)
    if hasattr(result, "__aiter__"):
        async for batch in result:
            for raw in batch:
                entities.append(mapper(raw))
    else:
        batches = await result
        for batch in batches:
            for raw in batch:
                entities.append(mapper(raw))
    return entities


async def run() -> list[dict[str, Any]]:
    all_entities: list[dict[str, Any]] = []
    for kind in KINDS:
        all_entities.extend(await collect_kind(kind))
    all_entities.sort(key=lambda e: (e["kind"], e["id"]))
    return all_entities


def main() -> int:
    entities = asyncio.run(run())
    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with OUT_PATH.open("w", encoding="utf-8") as f:
        for ent in entities:
            f.write(json.dumps(ent, separators=(",", ":"), ensure_ascii=False))
            f.write("\n")
    print(f"wrote {len(entities)} entities to {OUT_PATH}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
