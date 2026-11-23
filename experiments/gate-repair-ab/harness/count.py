#!/usr/bin/env python3
"""The measure, from the runtime's transcript of one run (P6): tool calls, characters read, gate runs
and the characters of their results, wall time.

    count.py <arm: toolbox|reviewdog> <transcript.json>

The transcript is what `batch-fetch-details` with `includeTranscripts` writes for a subagent's bcId:
`{"messages": [...]}`, with one `role: tool` message per tool call carrying `tool_name`,
`tool_args`, `tool_result`, `started_at_ms` and `completed_at_ms`.
"""
import collections
import json
import re
import sys

GATE = {
    "toolbox": re.compile(r"toolbox report \S+.*--baseline"),
    "reviewdog": re.compile(r"(?<!cat )(?<!less )(?<!head )\bscripts/quality-gate\.sh"),
}


def main() -> None:
    arm, path = sys.argv[1], sys.argv[2]
    calls = [m for m in json.load(open(path))["messages"] if m.get("role") == "tool"]
    starts = [int(m["started_at_ms"]) for m in calls if m.get("started_at_ms")]
    ends = [int(m["completed_at_ms"]) for m in calls if m.get("completed_at_ms")]
    gate = [m for m in calls if m.get("tool_name") == "run_terminal_cmd"
            and GATE[arm].search(str(m.get("tool_args", "")))]
    print(json.dumps({
        "toolCalls": len(calls),
        "byTool": dict(collections.Counter(m.get("tool_name", "?") for m in calls).most_common()),
        "gateRuns": len(gate),
        "resultChars": sum(len(str(m.get("tool_result", ""))) for m in calls),
        "gateChars": sum(len(str(m.get("tool_result", ""))) for m in gate),
        "wallSeconds": round((max(ends) - min(starts)) / 1000) if starts and ends else None,
    }))


if __name__ == "__main__":
    main()
