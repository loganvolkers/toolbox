#!/usr/bin/env python3
"""Report ShellCheck and shfmt results through Seikika's NDJSON runner slot."""

import argparse
import json
from pathlib import Path
import shlex
import subprocess
import sys


def capture(argv, acceptable=(0,)):
    result = subprocess.run(argv, capture_output=True, check=False)
    if result.returncode not in acceptable or result.stderr:
        detail = result.stderr.decode(errors="replace").strip()
        raise RuntimeError(f"{shlex.join(argv)} exited {result.returncode}: {detail}")
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pathspec", nargs="+", help="Git pathspecs of shell sources")
    args = parser.parse_args()

    paths = capture([
        "git", "ls-files", "-z", "--cached", "--others", "--exclude-standard",
        "--", *args.pathspec,
    ])
    files = sorted({
        path.decode("utf-8") for path in paths.split(b"\0")
        if path and Path(path.decode("utf-8")).is_file()
    })
    if not files:
        raise RuntimeError("no shell sources match the configured Git pathspecs")

    for path in files:
        lint = ["shellcheck", "--severity=warning", "--", path]
        # A status of 1 reports findings. Other statuses indicate a failed tool invocation.
        diagnostics = json.loads(capture(
            [lint[0], "--format=json1", *lint[1:]], acceptable=(0, 1)
        ))["comments"]
        findings = [
            f"line {item['line']}: SC{item['code']} {item['message']}"
            for item in diagnostics
        ]
        format_style = ["shfmt", "-i", "2", "-ci"]
        format_command = [*format_style, "--", path]
        formatted = capture(format_command) == Path(path).read_bytes()
        print(json.dumps({
            "type": "record",
            "kind": "file",
            "externalId": f"file:{path}",
            "data": {
                "path": path,
                "findings": findings,
                "formatted": formatted,
                "lintCommand": shlex.join(lint),
                "formatCommand": shlex.join([*format_style, "-w", "--", path]),
            },
        }))


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError, ValueError) as error:
        sys.exit(f"shell standard: {error}")
