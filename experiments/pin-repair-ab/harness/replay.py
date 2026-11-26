#!/usr/bin/env python3
"""Rebuild the results of terminal commands that a run's transcript lacks, by replaying the run (P6).

    replay.py <arm: seikika|reviewdog> <dest> <transcript.json> <openfga-src> <seikika-checkout> <seikika-target-dir>

The runtime sometimes records no result for a call. This moves the run's workspace <dest> aside,
sets up a fresh one at the same path with setup.sh, and runs the run's `run_terminal_cmd` calls in
order, then puts the run's workspace back.

- Replay stops at the first call that may change files and that it can't redo: any tool other than
  `run_terminal_cmd`, `read_file`, `grep` and `glob_file_search` (the transcript keeps no arguments
  for `search_replace`), or a background command. Missing results after that are not rebuilt.
- A command's rebuilt result is the runtime's result object with the command's stdout (L17: the
  transcript keeps stdout only), as `count.py` measures a recorded one.
- The replay is kept only if every recorded `run_terminal_cmd` result it reached has the same
  length when rebuilt. Otherwise it rebuilds nothing, and says which calls differed.

Prints `{"replayed": {"<call index>": <characters>}, "valid": bool, "checked": n, "mismatches":
[...], "stoppedAt": index or null}`, with call indexes counted over the transcript's tool calls.
"""
import json
import pathlib
import shutil
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
READ_ONLY = {"read_file", "grep", "glob_file_search"}


def missing(result) -> bool:
    return result in (None, "", {})


def rebuilt(command: str) -> int:
    done = subprocess.run(["bash", "-c", command], capture_output=True, text=True)
    return len(str({"resultType": "runTerminalCommandV2Result", "value": {
        "output": done.stdout, "exitCode": done.returncode, "poppedOutIntoBackground": False,
        "isRunningInBackground": False, "notInterrupted": False, "resultingWorkingDirectory": "",
        "didUserChange": False, "endedReason": "RUN_TERMINAL_COMMAND_ENDED_REASON_UNSPECIFIED",
        "outputRaw": ""}}))


def main() -> None:
    arm, dest, transcript, src, checkout, target = sys.argv[1:7]
    calls = [m for m in json.load(open(transcript))["messages"] if m.get("role") == "tool"]
    report = {"replayed": {}, "valid": True, "checked": 0, "mismatches": [], "stoppedAt": None}
    wanted = [i for i, m in enumerate(calls) if m.get("tool_name") == "run_terminal_cmd" and missing(m.get("tool_result"))]
    if not wanted:
        print(json.dumps(report))
        return
    kept = pathlib.Path(dest + ".replay-kept")
    shutil.move(dest, kept)
    try:
        subprocess.run([HERE / "setup.sh", src, arm, dest, checkout, target], check=True, capture_output=True)
        for i, call in enumerate(calls):
            name, args = call.get("tool_name"), call.get("tool_args")
            if name in READ_ONLY:
                continue
            if name != "run_terminal_cmd" or not isinstance(args, dict) or args.get("is_background"):
                report["stoppedAt"] = i
                break
            length = rebuilt(str(args.get("command", "")))
            if missing(call.get("tool_result")):
                report["replayed"][str(i)] = length
            else:
                report["checked"] += 1
                recorded = len(str(call["tool_result"]))
                if recorded != length:
                    report["mismatches"].append({"call": i, "recorded": recorded, "rebuilt": length})
    finally:
        shutil.rmtree(dest, ignore_errors=True)
        shutil.move(kept, dest)
    if report["mismatches"]:
        report["valid"], report["replayed"] = False, {}
    print(json.dumps(report))


if __name__ == "__main__":
    main()
