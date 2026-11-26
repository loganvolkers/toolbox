#!/usr/bin/env python3
"""The measures, from the runtime's transcript of one run (P6): tool calls, characters read,
Graphify and Seikika calls and the characters of their results, wall time, and the paths outside
the workspace and the run's scratch directory (/tmp/graphify-monorepo-ab/scratch/r<round>/codex-<n>)
that the run touched (the contamination check, README §6). Adapted from graphify-ab.

    count.py <transcript.json> <workspace>

The transcript is what `batch-fetch-details` with `includeTranscripts` writes for a subagent's
bcId: `{"messages": [...]}`, with one `role: tool` message per tool call carrying `tool_name`,
`tool_args`, `tool_result`, `started_at_ms` and `completed_at_ms`.

A `read_file` whose result the transcript lacks is counted as that file in the workspace
(`readImputed`), built the way a recorded `read_file` result is, as gate-repair-ab's count.py
does from round 5. Nothing else is imputed.
"""
import collections
import json
import pathlib
import re
import sys

GRAPHIFY = re.compile(r"(^|[\s;&|(])graphify\s+(query|path|explain|affected|god-nodes|section|update|stats|[a-z][\w-]*)")
SEIKIKA = re.compile(r"(^|[\s;&|(])seikika\s+(search|query|get|graph|owner|explain|describe|score|scorecard"
                     r"|refresh|actions|validate|test|plan|derive|serve|report|doctor|assess|connector|mcp)\b")
ABSOLUTE = re.compile(r"(?<![\w.])/(?:tmp|agent|home|root|opt|cursor|workspace|usr/local/src)/[^\s'\"`)\]}:,]*")


def command(m: dict) -> str:
    a = m.get("tool_args")
    return str(a.get("command", "")) if isinstance(a, dict) else str(a or "")


def read_file_result(dest: pathlib.Path, args: dict) -> str | None:
    target = pathlib.Path(str(args.get("target_file", "")))
    path = target if target.is_absolute() else dest / target
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return None
    lines = text.splitlines(keepends=True)
    offset, limit = args.get("offset"), args.get("limit")
    if offset or limit:
        start = max(int(offset or 1), 1) - 1
        text = "".join(lines[start:start + int(limit)] if limit else lines[start:])
    return str({"resultType": "readFileResult", "value": {
        "contents": text, "didDowngradeToLineRange": False, "didShortenLineRange": False,
        "didSetDefaultLineRange": False, "relativeWorkspacePath": "", "didShortenCharRange": False,
        "totalLines": len(lines) + 1, "matchingCursorRules": []}})


def outside(calls: list, dest: str, scratch: str) -> list[str]:
    """Absolute paths in tool arguments that are not in the workspace, the run's scratch directory
    or the tools' own install."""
    allowed = (dest, scratch, "/home/ubuntu/.local/bin/graphify", "/home/ubuntu/.local/share/uv/tools/graphifyy",
               "/usr/local/bin/graphify", "/usr/local/bin/seikika")
    seen = []
    for m in calls:
        for p in ABSOLUTE.findall(json.dumps(m.get("tool_args", ""))):
            p = p.rstrip(".")
            if not p.startswith(allowed) and p not in seen:
                seen.append(p)
    return seen


def main() -> None:
    path, dest = sys.argv[1], sys.argv[2].rstrip("/")
    ws = pathlib.Path(dest)
    scratch = str(ws.parent.parent / "scratch" / ws.parent.name / ws.name)
    calls = [m for m in json.load(open(path))["messages"] if m.get("role") == "tool"]
    starts = [int(m["started_at_ms"]) for m in calls if m.get("started_at_ms")]
    ends = [int(m["completed_at_ms"]) for m in calls if m.get("completed_at_ms")]
    graphify = [m for m in calls if m.get("tool_name") == "run_terminal_cmd" and GRAPHIFY.search(command(m))]
    sub = collections.Counter(c.group(2) for m in graphify for c in GRAPHIFY.finditer(command(m)))
    seikika = [m for m in calls if m.get("tool_name") == "run_terminal_cmd" and SEIKIKA.search(command(m))]
    tsub = collections.Counter(c.group(2) for m in seikika for c in SEIKIKA.finditer(command(m)))
    missing = [m for m in calls if m.get("tool_result") in (None, "", {})]
    imputed = sum(len(read_file_result(pathlib.Path(dest), m["tool_args"]) or "") for m in missing
                  if m.get("tool_name") == "read_file" and isinstance(m.get("tool_args"), dict))
    recorded = sum(len(str(m.get("tool_result", ""))) for m in calls)
    print(json.dumps({
        "toolCalls": len(calls),
        "byTool": dict(collections.Counter(m.get("tool_name", "?") for m in calls).most_common()),
        "resultChars": recorded,
        "missingResults": [m.get("tool_name", "?") for m in missing],
        "readImputed": imputed,
        "resultCharsImputed": recorded + imputed,
        "graphifyCalls": len(graphify),
        "graphifyCommands": dict(sub.most_common()),
        "graphifyChars": sum(len(str(m.get("tool_result", ""))) for m in graphify),
        "seikikaCalls": len(seikika),
        "seikikaCommands": dict(tsub.most_common()),
        "seikikaChars": sum(len(str(m.get("tool_result", ""))) for m in seikika),
        "outsidePaths": outside(calls, dest, scratch),
        "wallSeconds": round((max(ends) - min(starts)) / 1000) if starts and ends else None,
    }))


if __name__ == "__main__":
    main()
