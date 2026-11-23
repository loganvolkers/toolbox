"""Thin local stand-in for port_ocean.context.ocean when the full framework
cannot be imported (missing fastapi and other deps in this harness).

Keeps the same `@ocean.on_resync(kind)` registration shape used by the
Ocean jira and fake-integration examples.
"""

from __future__ import annotations

from typing import Any, Callable

ResyncHandler = Callable[[str], Any]

_handlers: dict[str, ResyncHandler] = {}


class _Ocean:
    def on_resync(
        self, kind: str | None = None
    ) -> Callable[[ResyncHandler], ResyncHandler]:
        def wrapper(function: ResyncHandler) -> ResyncHandler:
            if kind is not None:
                _handlers[kind] = function
            return function

        return wrapper


ocean = _Ocean()


def registered_handlers() -> dict[str, ResyncHandler]:
    return dict(_handlers)
