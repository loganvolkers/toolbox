#!/usr/bin/env python3
"""Tool calls whose arguments name another run's workspace, or the round directory that holds
them all (a listing of the other arms). Every run can read `/tmp/actions-ab/r<n>/`; this checks
that none did.

    isolation.py <transcript.json> <round> <run>

Prints `{"crossWorkspace": [...], "roundListing": [...]}`, each entry the call's index (1-based),
tool name and the matching path.
"""
import json
import re
import sys


def main() -> None:
    transcript, rnd, run = sys.argv[1:]
    calls = [m for m in json.load(open(transcript))["messages"] if m.get("role") == "tool"]
    base = "/tmp/actions-ab/"
    other = re.compile(re.escape(base) + r"(r\w+)(?:/([\w.-]+))?")
    cross, listing = [], []
    for i, m in enumerate(calls, 1):
        args = m.get("tool_args")
        text = args if isinstance(args, str) else json.dumps(args)
        for hit in other.finditer(text):
            r, name = hit.group(1), hit.group(2)
            if r == f"r{rnd}" and name and name.split(".")[0] == run:
                continue
            if r == "src":
                continue
            entry = {"call": i, "tool": m.get("tool_name"), "path": hit.group(0)}
            (cross if name else listing).append(entry)
    print(json.dumps({"crossWorkspace": cross, "roundListing": listing}, indent=1))


if __name__ == "__main__":
    main()
