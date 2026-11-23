"""Thin Ocean-compatible decorator shim.

Full `port_ocean` needs fastapi and other deps not present in this arm; the
scaffold allows vendoring the `@ocean.on_resync` pattern so handlers stay in
integration.py and resync.py can invoke them without Port SaaS.
"""

from __future__ import annotations

from collections.abc import Awaitable, Callable
from typing import Any

ResyncHandler = Callable[[str], Any]


class _OceanShim:
    def __init__(self) -> None:
        self.resync_handlers: dict[str, ResyncHandler] = {}
        self.integration_config: dict[str, Any] = {}

    def on_resync(
        self, kind: str | None = None
    ) -> Callable[[ResyncHandler], ResyncHandler]:
        def decorator(fn: ResyncHandler) -> ResyncHandler:
            key = kind if kind is not None else "*"
            self.resync_handlers[key] = fn
            return fn

        return decorator

    def on_start(self) -> Callable[[Callable[[], Awaitable[None]]], Callable[[], Awaitable[None]]]:
        def decorator(fn: Callable[[], Awaitable[None]]) -> Callable[[], Awaitable[None]]:
            return fn

        return decorator


ocean = _OceanShim()
