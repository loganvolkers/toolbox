#!/usr/bin/env python3
"""The runtime's measures for one run (P6): tool calls, by kind (L18), characters of tool results,
results the transcript dropped (L20), and wall time.

    count.py <transcript.json>

The transcript is what `batch-fetch-details` with `includeTranscripts` writes for a subagent's bcId:
`{"messages": [...]}`, with one `role: tool` message per tool call carrying `tool_name`,
`tool_args`, `tool_result`, `started_at_ms` and `completed_at_ms`.
"""
import collections
import json
import sys

KIND = {
    "edit": {"edit_file", "edit_file_v2", "search_replace", "write", "apply_patch", "str_replace",
             "StrReplace", "Write", "multi_edit", "delete_file", "Delete"},
    "read": {"read_file", "read_file_v2", "Read", "list_dir", "list_dir_v2", "glob_file_search",
             "Glob", "file_search"},
    "search": {"grep", "grep_search", "Grep", "codebase_search", "semantic_search_full", "rg"},
    "shell": {"run_terminal_cmd", "run_terminal_command_v2", "Shell", "await_shell", "AwaitShell"},
}


def kind(name: str) -> str:
    for k, names in KIND.items():
        if name in names:
            return k
    return "other"


def main() -> None:
    calls = [m for m in json.load(open(sys.argv[1]))["messages"] if m.get("role") == "tool"]
    starts = [int(m["started_at_ms"]) for m in calls if m.get("started_at_ms")]
    ends = [int(m["completed_at_ms"]) for m in calls if m.get("completed_at_ms")]
    missing = [m.get("tool_name", "?") for m in calls if m.get("tool_result") in (None, "", {})]
    print(json.dumps({
        "toolCalls": len(calls),
        "byKind": dict(collections.Counter(kind(m.get("tool_name", "?")) for m in calls).most_common()),
        "byTool": dict(collections.Counter(m.get("tool_name", "?") for m in calls).most_common()),
        "resultChars": sum(len(str(m.get("tool_result", ""))) for m in calls),
        "missingResults": missing,
        "wallSeconds": round((max(ends) - min(starts)) / 1000) if starts and ends else None,
    }))


if __name__ == "__main__":
    main()
