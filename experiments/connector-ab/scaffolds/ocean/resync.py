#!/usr/bin/env python3
"""Entry point the harness / agent uses without Port SaaS.

Call your resync handlers against Forge (see FORGE.md) and write mapped entities
to OUT/entities.ndjson in the common schema from the experiment README.
"""

from __future__ import annotations

import sys


def main() -> int:
    print("scaffold: implement resync against Forge → OUT/entities.ndjson", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
