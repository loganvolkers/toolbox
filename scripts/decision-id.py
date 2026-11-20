#!/usr/bin/env python3
"""Mint a decision id: ``D`` plus a ULID (ulid/spec).

The id is time-sortable and needs no shared counter, so two agents can each
add a decision without claiming the same number. ``--self-test`` checks the
encoding against oklog/ulid's published vector. Backfilled ids use a zero
random part; a new id uses 80 bits from ``secrets``.
"""

from __future__ import annotations

import argparse
import secrets
import sys
import time

ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
_INDEX = {c: i for i, c in enumerate(ALPHABET)}

# oklog/ulid TestAlizainCompatibility: unix ms 1469918176385, zero entropy.
VECTOR_MS = 1469918176385
VECTOR = "01ARYZ6S410000000000000000"


def encode(timestamp_ms: int, randomness: bytes) -> str:
    """26-character Crockford ULID. ``randomness`` is 10 bytes (80 bits)."""
    if timestamp_ms < 0 or timestamp_ms >= 2**48:
        raise ValueError(f"timestamp out of range: {timestamp_ms}")
    if len(randomness) != 10:
        raise ValueError("randomness must be 10 bytes")
    value = (timestamp_ms << 80) | int.from_bytes(randomness, "big")
    chars = [ALPHABET[(value >> (shift * 5)) & 31] for shift in range(25, -1, -1)]
    return "".join(chars)


def decode(text: str) -> tuple[int, bytes]:
    """Return ``(timestamp_ms, randomness)`` for a 26-character ULID."""
    raw = text.strip().upper()
    if len(raw) != 26 or any(c not in _INDEX for c in raw):
        raise ValueError(f"not a ULID: {text!r}")
    value = 0
    for char in raw:
        value = (value << 5) | _INDEX[char]
    if value >= 2**128:
        raise ValueError(f"ULID overflows 128 bits: {text!r}")
    return value >> 80, (value & ((1 << 80) - 1)).to_bytes(10, "big")


def new_id(timestamp_ms: int | None = None) -> str:
    """A citation token: ``D`` plus a new ULID."""
    ms = int(time.time() * 1000) if timestamp_ms is None else timestamp_ms
    return "D" + encode(ms, secrets.token_bytes(10))


def backfill_id(unix_seconds: int, offset: int) -> str:
    """A reproducible id for a decision introduced at ``unix_seconds``.

    ``offset`` is the decision's place among those introduced in that same
    second (0 keeps it inside the second; 999 still does). The random part is
    zero, so the id is a pure function of that millisecond.
    """
    if not 0 <= offset < 1000:
        raise ValueError(f"offset must be 0..999, got {offset}")
    return "D" + encode(unix_seconds * 1000 + offset, bytes(10))


def self_test() -> None:
    got = encode(VECTOR_MS, bytes(10))
    if got != VECTOR:
        raise SystemExit(f"vector mismatch: {got} != {VECTOR}")
    ms, rand = decode(VECTOR)
    if ms != VECTOR_MS or rand != bytes(10):
        raise SystemExit(f"decode mismatch: {ms} {rand.hex()}")
    earlier = backfill_id(1_700_000_000, 0)
    later = backfill_id(1_700_000_000, 1)
    if not (earlier < later and earlier.endswith("0" * 16)):
        raise SystemExit(f"backfill order failed: {earlier} {later}")
    minted = new_id()
    body = minted[1:]
    if not (
        minted.startswith("D")
        and len(body) == 26
        and decode(body)[0] > VECTOR_MS
    ):
        raise SystemExit(f"mint failed: {minted}")
    # Two ids from the same millisecond still differ (80 bits, not a counter).
    a, b = new_id(VECTOR_MS), new_id(VECTOR_MS)
    if a == b or not (a.startswith("D01ARYZ6S41") and b.startswith("D01ARYZ6S41")):
        raise SystemExit(f"same-millisecond ids were not independent: {a} {b}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--self-test", action="store_true", help="check the ULID vector and exit"
    )
    args = parser.parse_args(argv)
    if args.self_test:
        self_test()
        print("ok")
        return 0
    print(new_id())
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
