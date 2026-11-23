#!/usr/bin/env python3
"""The measure, from the runtime's transcript of one run (P6): tool calls, characters read, gate runs
and the characters of their results, wall time.

    count.py <arm: toolbox|reviewdog> <transcript.json> [<repo>]

The transcript is what `batch-fetch-details` with `includeTranscripts` writes for a subagent's bcId:
`{"messages": [...]}`, with one `role: tool` message per tool call carrying `tool_name`,
`tool_args`, `tool_result`, `started_at_ms` and `completed_at_ms`.

`missingResults` names each call whose result the transcript doesn't have. With <repo>, the run's
checkout, a `read_file` among them is counted as the runtime's result for that file on branch `pr`
(`readImputed`, `resultCharsImputed`), built the way a recorded `read_file` result is (round 4,
threat 13). Nothing else is imputed.
"""
import collections
import json
import re
import subprocess
import sys

GATE = {
    "toolbox": re.compile(r"toolbox report \S+.*--baseline"),
    "reviewdog": re.compile(r"(?<!cat )(?<!less )(?<!head )\bscripts/quality-gate\.sh"),
}


def read_file_result(repo: str, args: dict) -> str | None:
    """A recorded `read_file` result for the whole file as it is on `pr`, or a line range of it."""
    target = str(args.get("target_file", ""))
    rel = target.split("/openfga/", 1)[-1] if "/openfga/" in target else target
    shown = subprocess.run(["git", "show", f"pr:{rel}"], cwd=repo, capture_output=True, text=True)
    if shown.returncode != 0:
        return None
    text = shown.stdout
    lines = text.splitlines(keepends=True)
    offset, limit = args.get("offset"), args.get("limit")
    if offset or limit:
        start = max(int(offset or 1), 1) - 1
        text = "".join(lines[start:start + int(limit)] if limit else lines[start:])
    return str({"resultType": "readFileResult", "value": {
        "contents": text, "didDowngradeToLineRange": False, "didShortenLineRange": False,
        "didSetDefaultLineRange": False, "relativeWorkspacePath": "", "didShortenCharRange": False,
        "totalLines": len(lines) + 1, "matchingCursorRules": []}})


def main() -> None:
    arm, path = sys.argv[1], sys.argv[2]
    repo = sys.argv[3] if len(sys.argv) > 3 else None
    calls = [m for m in json.load(open(path))["messages"] if m.get("role") == "tool"]
    starts = [int(m["started_at_ms"]) for m in calls if m.get("started_at_ms")]
    ends = [int(m["completed_at_ms"]) for m in calls if m.get("completed_at_ms")]
    gate = [m for m in calls if m.get("tool_name") == "run_terminal_cmd"
            and GATE[arm].search(str(m.get("tool_args", "")))]
    missing = [m for m in calls if m.get("tool_result") in (None, "", {})]
    imputed = 0
    for m in missing:
        if repo and m.get("tool_name") == "read_file" and isinstance(m.get("tool_args"), dict):
            imputed += len(read_file_result(repo, m["tool_args"]) or "")
    recorded = sum(len(str(m.get("tool_result", ""))) for m in calls)
    print(json.dumps({
        "toolCalls": len(calls),
        "byTool": dict(collections.Counter(m.get("tool_name", "?") for m in calls).most_common()),
        "gateRuns": len(gate),
        "resultChars": recorded,
        "missingResults": [m.get("tool_name", "?") for m in missing],
        "readImputed": imputed,
        "resultCharsImputed": recorded + imputed,
        "gateChars": sum(len(str(m.get("tool_result", ""))) for m in gate),
        "wallSeconds": round((max(ends) - min(starts)) / 1000) if starts and ends else None,
    }))


if __name__ == "__main__":
    main()
