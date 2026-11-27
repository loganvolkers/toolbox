#!/usr/bin/env python3
"""Plan and run the maintainer PR assessment pilot with a caller-owned harness."""

import argparse
from contextlib import closing, contextmanager
import fnmatch
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time


POLICY = "portal/assessment-pr.json"
PACKET_BYTES = 48_000
OUTPUT_BYTES = 1_048_576
DEADLINE = 300
MAX_TOKENS = 32_000
MAX_TURNS = 6
ACTIVE_LEASE = DEADLINE + 15
IDENTIFIER = re.compile(r"[a-z0-9][a-z0-9-]*")


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=True, separators=(",", ":")).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(repo, *args, limit=OUTPUT_BYTES):
    with subprocess.Popen(["git", "-C", str(repo), *args], stdout=subprocess.PIPE) as process:
        data = process.stdout.read(limit + 1)
        if len(data) > limit:
            process.kill()
            raise ValueError("Git output exceeds the assessment input limit")
        if process.wait():
            raise ValueError(f"git {args[0]} failed")
        return data


def commit(repo, ref):
    return git(repo, "rev-parse", "--verify", "--end-of-options", f"{ref}^{{commit}}").decode().strip()


def policy_at(repo, base):
    entry = git(repo, "ls-tree", base, "--", POLICY)
    if not entry:
        return {"routes": [], "checks": {}}, None
    if not entry.startswith((b"100644 blob ", b"100755 blob ")):
        raise ValueError("assessment policy must be a regular file")
    raw = git(repo, "show", f"{base}:{POLICY}")
    policy = json.loads(raw)
    if policy.get("format") != "seikika-pr-policy/v1":
        raise ValueError("unsupported assessment policy version")
    names = [route["id"] for route in policy["routes"]]
    if len(set(names)) != len(names) or any(not IDENTIFIER.fullmatch(name) for name in names):
        raise ValueError("route IDs must be unique lowercase identifiers")
    for name, command in policy["checks"].items():
        if not IDENTIFIER.fullmatch(name) or not isinstance(command, list) or not command or not all(
            isinstance(arg, str) for arg in command
        ):
            raise ValueError("checks require an identifier and command argument array")
    for route in policy["routes"]:
        if set(route["checks"]) - policy["checks"].keys():
            raise ValueError(f"route {route['id']} names an undefined check")
        if any(not IDENTIFIER.fullmatch(name) for name in route["criteria"]):
            raise ValueError("criterion IDs must be lowercase identifiers")
    return policy, digest(raw)


def matches(path, pattern):
    # Path.match is suffix-based and Python's glob treatment of ** varies by version.
    return fnmatch.fnmatchcase(path, pattern)


def build_plan(repo, base, head):
    base, head = commit(repo, base), commit(repo, head)
    merge_base = git(repo, "merge-base", base, head).decode().strip()
    policy, policy_hash = policy_at(repo, base)
    raw = git(repo, "diff", "--no-ext-diff", "--find-renames", "--name-status", "-z", merge_base, head, "--")
    entries = iter(raw.decode(errors="surrogateescape").split("\0")[:-1])
    changes = []
    for status in entries:
        paths = [next(entries)]
        if status.startswith(("R", "C")):
            paths.append(next(entries))
        changes.append({"status": status, "paths": paths})
    all_paths = {path for change in changes for path in change["paths"]}
    selected, skipped = [], []
    for route in policy["routes"]:
        paths = sorted(path for path in all_paths if any(matches(path, p) for p in route["paths"]))
        if paths:
            selected.append({key: route[key] for key in ("id", "subject", "checks", "criteria", "risk")} | {
                "paths": paths,
                "design": route.get("design", False),
                "owner": route["owner"],
                "reason": "changed path matches route policy",
            })
        else:
            skipped.append({"id": route["id"], "reason": "no matching changed path"})
    known = {path for route in selected for path in route["paths"]}
    plan = {
        "format": "seikika-pr-assessment-plan/v1",
        "base": base, "mergeBase": merge_base, "head": head,
        "policyDigest": policy_hash, "changes": changes,
        "selected": selected, "skipped": skipped, "unmappedPaths": sorted(all_paths - known),
        "scopeOwner": policy.get("owner", "maintainers"),
        "scope": "scope_unknown" if policy_hash is None or all_paths - known else "scoped",
        "checks": sorted({check for route in selected for check in route["checks"]}),
    }
    return plan | {"digest": digest(canonical(plan))}, policy["checks"]


def write_json(value, path=None):
    text = json.dumps(value, indent=2, ensure_ascii=True, sort_keys=True) + "\n"
    if path is None:
        sys.stdout.write(text)
    else:
        path.write_text(text, encoding="utf-8")


def terminate(process):
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    process.wait()


def execute(command, *, cwd, deadline, packet=None, env=None, cancelled=lambda: False):
    """Spool child output and supervise its process group."""
    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr, tempfile.TemporaryFile() as stdin:
        stdin.write(packet or b"")
        stdin.seek(0)
        with subprocess.Popen(command, cwd=cwd, stdin=stdin, stdout=stdout, stderr=stderr,
                              env=env, start_new_session=True) as process:
            stop = time.monotonic() + deadline
            try:
                while True:
                    if cancelled():
                        return "superseded", b"", b""
                    if max(os.fstat(f.fileno()).st_size for f in (stdout, stderr)) > OUTPUT_BYTES:
                        return "output-limit", b"", b""
                    if process.poll() is not None:
                        stdout.seek(0)
                        stderr.seek(0)
                        status = "pass" if process.returncode == 0 else "fail"
                        return status, stdout.read(OUTPUT_BYTES), stderr.read(OUTPUT_BYTES)
                    if time.monotonic() >= stop:
                        return "timeout", b"", b""
                    time.sleep(0.1)
            finally:
                terminate(process)


def run_checks(repo, plan, definitions, only, output):
    if commit(repo, "HEAD") != plan["head"] or git(repo, "status", "--porcelain", "--untracked-files=no"):
        raise ValueError("deterministic checks require a clean tracked checkout of --head")
    if any(not IDENTIFIER.fullmatch(name) for name in only):
        raise ValueError("check IDs in --only must be lowercase identifiers")
    output.mkdir(parents=True, exist_ok=True)
    results = {}
    for name in plan["checks"]:
        if not only or name in only:
            try:
                status, stdout, stderr = execute(definitions[name], cwd=repo, deadline=DEADLINE)
            except OSError as error:
                status, stdout, stderr = "unavailable", b"", str(error).encode()
            log = output / f"{name}.log"
            log.write_bytes(stdout + stderr)
            results[name] = {"status": status, "sha256": digest(log.read_bytes()), "artifact": log.name}
    evidence = {"format": "seikika-pr-evidence/v1", "head": plan["head"], "plan": plan["digest"],
                "checks": results, "notRun": sorted(set(plan["checks"]) - results.keys()),
                "obligations": obligations(plan, results)}
    write_json(evidence, output / "evidence.json")
    return evidence


def load_evidence(plan, files):
    evidence = {}
    for path in files:
        data = json.loads(path.read_bytes())
        if (data.get("format"), data.get("head"), data.get("plan")) != (
            "seikika-pr-evidence/v1", plan["head"], plan["digest"],
        ):
            raise ValueError("evidence belongs to another head or assessment plan")
        for name, result in data["checks"].items():
            if name not in plan["checks"] or name in evidence or result["artifact"] != f"{name}.log":
                raise ValueError("unexpected or duplicate evidence check")
            log = (path.parent / result["artifact"]).read_bytes()
            if digest(log) != result["sha256"]:
                raise ValueError(f"artifact digest mismatch for {name}")
            evidence[name] = result
    return evidence


def packet_for(repo, plan, evidence):
    paths = sorted({path for route in plan["selected"] if route["criteria"] for path in route["paths"]})
    diff = git(repo, "--literal-pathspecs", "diff", "--no-ext-diff", "--no-textconv",
               plan["mergeBase"], plan["head"], "--", *paths, limit=PACKET_BYTES)
    packet = canonical({"format": "seikika-pr-delegate/v1", "plan": plan, "checks": evidence,
                        "diff": diff.decode("utf-8", errors="replace"),
                        "limits": {"tokens": MAX_TOKENS, "turns": MAX_TURNS}})
    if len(packet) > PACKET_BYTES:
        raise ValueError("assessment packet exceeds the input byte budget")
    return packet


def evidence_reference(entry, repo, plan, route, checks):
    if "check" in entry:
        return set(entry) == {"check"} and entry["check"] in route["checks"] and entry["check"] in checks
    if set(entry) != {"path", "line", "side"} or entry["path"] not in route["paths"]:
        return False
    if entry["side"] not in ("base", "head") or type(entry["line"]) is not int or entry["line"] < 1:
        return False
    revision = plan["head" if entry["side"] == "head" else "mergeBase"]
    path = entry["path"]
    tree_entry = git(repo, "--literal-pathspecs", "ls-tree", revision, "--", path)
    if not tree_entry.startswith((b"100644 blob ", b"100755 blob ")):
        return False
    blob = git(repo, "show", f"{revision}:{path}")
    return b"\0" not in blob and entry["line"] <= len(blob.splitlines())


def validate_result(repo, plan, checks, body):
    data = json.loads(body)
    if data.get("plan") != plan["digest"]:
        raise ValueError("delegate result did not echo the plan digest")
    expected = {(r["id"], c): r for r in plan["selected"] for c in r["criteria"]}
    seen = set()
    for item in data["verdicts"]:
        key = item["route"], item["criterion"]
        verdict = item["verdict"]
        if key not in expected or key in seen or verdict not in ("pass", "fail", "unsure"):
            raise ValueError("unexpected verdict, criterion, or duplicate")
        seen.add(key)
        if not isinstance(item["reason"], str) or not item["reason"].strip():
            raise ValueError("every verdict needs a reason")
        references = item["evidence"]
        if not isinstance(references, list):
            raise ValueError("evidence must be an array")
        if (verdict != "unsure" and not references) or any(
            not isinstance(entry, dict) or not evidence_reference(entry, repo, plan, expected[key], checks)
            for entry in references
        ):
            raise ValueError("verdict refers to missing or out-of-scope evidence")
        if verdict == "fail":
            if type(item.get("severity")) is not int or not 1 <= item["severity"] <= 4:
                raise ValueError("a failing verdict requires severity 1 through 4")
            if not isinstance(item.get("suggestion"), str) or not item["suggestion"].strip():
                raise ValueError("a failing verdict requires an actionable suggestion")
    if seen != expected.keys():
        raise ValueError("missing verdicts for planned review criteria")
    usage = data.get("usage")
    if usage is not None:
        for name in ("inputTokens", "outputTokens", "turns"):
            if type(usage.get(name)) is not int or usage[name] < 0:
                raise ValueError("usage counters must be nonnegative integers")
        if usage.get("cachedInputTokens") is not None and (
            type(usage["cachedInputTokens"]) is not int or not 0 <= usage["cachedInputTokens"] <= usage["inputTokens"]
        ):
            raise ValueError("cached input must be part of input tokens")
    return data


SCHEMA = """
CREATE TABLE IF NOT EXISTS current_plan (
    repository TEXT, pr INTEGER, generation INTEGER NOT NULL, plan TEXT NOT NULL,
    PRIMARY KEY (repository, pr)
);
CREATE TABLE IF NOT EXISTS attempts (
    id INTEGER PRIMARY KEY,
    repository TEXT NOT NULL, pr INTEGER NOT NULL, generation INTEGER NOT NULL,
    plan TEXT NOT NULL, head TEXT NOT NULL, started REAL NOT NULL, finished REAL,
    state TEXT NOT NULL, adapter TEXT NOT NULL, slot TEXT NOT NULL, override_reason TEXT, result TEXT
);
CREATE INDEX IF NOT EXISTS attempts_repo_time ON attempts(repository, started);
CREATE INDEX IF NOT EXISTS attempts_pr ON attempts(repository, pr, started);
"""


@contextmanager
def transaction(path):
    with closing(sqlite3.connect(path, timeout=10)) as conn, conn:
        conn.row_factory = sqlite3.Row
        conn.executescript(SCHEMA)
        conn.execute("BEGIN IMMEDIATE")
        yield conn


def activate(conn, key, generation, plan):
    previous = conn.execute(
        "SELECT generation, plan FROM current_plan WHERE repository = ? AND pr = ?", key,
    ).fetchone()
    if previous and (generation < previous["generation"] or
                     (generation == previous["generation"] and previous["plan"] != plan)):
        return False
    conn.execute("REPLACE INTO current_plan VALUES (?, ?, ?, ?)", (*key, generation, plan))
    return True


def reserve(conn, key, generation, plan, adapter, override, slot="review"):
    now = time.time()
    if not activate(conn, key, generation, plan["digest"]):
        return "superseded", None
    conn.execute(
        """UPDATE attempts SET state = 'abandoned', finished = ?
           WHERE repository = ? AND pr = ? AND state = 'running' AND started <= ?""",
        (now, *key, now - ACTIVE_LEASE),
    )
    rows = conn.execute(
        "SELECT * FROM attempts WHERE repository = ? AND pr = ? ORDER BY id DESC", key,
    ).fetchall()
    same = [row for row in rows if row["plan"] == plan["digest"] and row["slot"] == slot]
    if any(row["state"] == "running" for row in rows):
        return "busy", None
    if same and (len(same) >= 2 or same[0]["state"] not in ("adapter-error", "timeout", "abandoned")):
        return "duplicate", same[0]["id"]
    if sum(row["started"] > now - 3600 for row in rows) >= 2:
        return "pr-hourly-limit", None
    if conn.execute("SELECT count(*) FROM attempts WHERE repository = ? AND started > ?",
                    (key[0], now - 3600)).fetchone()[0] >= 8:
        return "repository-hourly-limit", None
    if len(rows) >= 3 and not override:
        return "cycle-limit", None
    if not override and plan["selected"] and all(route["design"] for route in plan["selected"]):
        recent = [json.loads(row["result"]) for row in rows if row["state"] == "reviewed"][:2]
        if len(recent) == 2 and all(r.get("designOnly") and not r["newFindings"] for r in recent):
            return "design-plateau", None
    attempt = conn.execute(
        """INSERT INTO attempts(repository, pr, generation, plan, head, started, state, adapter, slot, override_reason)
           VALUES (?, ?, ?, ?, ?, ?, 'running', ?, ?, ?)""",
        (*key, generation, plan["digest"], plan["head"], now, adapter, slot, override),
    )
    return "running", attempt.lastrowid


def is_current(conn, key, plan):
    row = conn.execute("SELECT plan FROM current_plan WHERE repository = ? AND pr = ?", key).fetchone()
    return row is not None and row[0] == plan


def settle(conn, key, plan, attempt, state, result):
    if not is_current(conn, key, plan["digest"]):
        state, result = "superseded", {}
    if state == "reviewed":
        previous = set()
        for row in conn.execute(
            "SELECT result FROM attempts WHERE repository = ? AND pr = ? AND state = 'reviewed'", key,
        ):
            previous.update(json.loads(row["result"])["findings"])
        # Criterion and evidence paths survive source line moves. Historical evidence remains in the ledger.
        findings = set()
        for item in result["verdicts"]:
            if item["verdict"] == "fail":
                paths = sorted({e.get("path", e.get("check")) for e in item["evidence"]})
                findings.add(digest(canonical([item["route"], item["criterion"], paths])))
        result["findings"] = sorted(findings)
        result["newFindings"] = sorted(findings - previous)
        result["designOnly"] = all(route["design"] for route in plan["selected"])
    updated = conn.execute(
        "UPDATE attempts SET state = ?, result = ?, finished = ? WHERE id = ? AND state = 'running'",
        (state, canonical(result).decode(), time.time(), attempt),
    )
    return state if updated.rowcount else "abandoned"


def history(conn, key):
    active = conn.execute(
        "SELECT plan FROM current_plan WHERE repository = ? AND pr = ?", key,
    ).fetchone()
    current = active[0] if active else None
    attempts = []
    for row in conn.execute("SELECT * FROM attempts WHERE repository = ? AND pr = ? ORDER BY id", key):
        attempt = dict(row)
        attempt["result"] = json.loads(row["result"]) if row["result"] else None
        attempt["current"] = current == row["plan"]
        attempts.append(attempt)
    return {"activePlan": current, "attempts": attempts}


def obligations(plan, checks, result=None):
    outcomes = {(r["route"], r["criterion"]): r["verdict"] for r in (result or {}).get("verdicts", [])}
    return {
        route["id"]: (
            "incomplete" if any(checks.get(c, {}).get("status") != "pass" for c in route["checks"])
            or any(outcomes.get((route["id"], c)) in (None, "unsure") for c in route["criteria"])
            else "needs_owner_decision" if route["criteria"] else "satisfied"
        )
        for route in plan["selected"] if route["risk"] == "high"
    }


def delegate(args, repo, plan):
    key = args.repository, args.pr
    with transaction(args.ledger) as conn:
        current = activate(conn, key, args.generation, plan["digest"])
    if not current:
        return {"state": "superseded", "plan": plan["digest"]}
    checks = load_evidence(plan, args.evidence)
    outcome = {"plan": plan["digest"], "scope": plan["scope"], "obligations": obligations(plan, checks)}
    if not any(route["criteria"] for route in plan["selected"]):
        return outcome | {"state": "no-review-planned"}
    if any(checks.get(name, {}).get("status") != "pass" for name in plan["checks"]):
        return outcome | {"state": "incomplete", "reason": "planned checks must have passing evidence"}
    try:
        packet = packet_for(repo, plan, checks)
    except ValueError as error:
        return outcome | {"state": "incomplete", "reason": str(error)}
    with transaction(args.ledger) as conn:
        state, attempt = reserve(conn, key, args.generation, plan, args.adapter, args.override_reason, args.slot)
    if state != "running":
        return outcome | {"state": state, "attempt": attempt}

    def cancelled():
        with closing(sqlite3.connect(args.ledger, timeout=10)) as conn:
            return not is_current(conn, key, plan["digest"])

    started = time.monotonic()
    outcome_body = {}
    state = "adapter-error"
    try:
        # Adapter arguments are selected by the runner. Run away from the repository and omit
        # environment credentials unless explicitly forwarded by the runner.
        env = {name: os.environ[name] for name in ("PATH", "HOME", "LANG", *args.forward_env) if name in os.environ}
        env.update(SEIKIKA_MAX_TOKENS=str(MAX_TOKENS), SEIKIKA_MAX_TURNS=str(MAX_TURNS))
        with tempfile.TemporaryDirectory(prefix="seikika-delegate-") as directory:
            state, stdout, _ = execute(args.command, cwd=directory, deadline=DEADLINE, packet=packet,
                                       env=env, cancelled=cancelled)
        if state == "pass":
            state = "invalid-result"
            outcome_body = validate_result(repo, plan, checks, stdout)
            usage = outcome_body.get("usage")
            outcome_body["usageSource"] = "adapter-reported" if usage is not None else "unknown"
            state = "budget-exceeded" if usage is not None and (
                usage["inputTokens"] + usage["outputTokens"] > MAX_TOKENS or usage["turns"] > MAX_TURNS
            ) else "reviewed"
        elif state == "fail":
            state = "adapter-error"
    except (ValueError, KeyError, TypeError, AttributeError, OSError) as error:
        outcome_body = {"error": str(error)}
    except KeyboardInterrupt:
        state = "cancelled"
        raise
    finally:
        outcome_body["durationMs"] = round((time.monotonic() - started) * 1000)
        with transaction(args.ledger) as conn:
            state = settle(conn, key, plan, attempt, state, outcome_body)
    if state == "reviewed":
        outcome["obligations"] = obligations(plan, checks, outcome_body)
    return outcome | {"state": state, "attempt": attempt,
                      "result": outcome_body if state != "superseded" else None}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    commands = parser.add_subparsers(dest="action", required=True)
    for action in ("plan", "check", "activate", "delegate", "status"):
        cmd = commands.add_parser(action)
        if action != "status":
            cmd.add_argument("--base", required=True)
            cmd.add_argument("--head", default="HEAD")
        if action in ("plan", "check"):
            cmd.add_argument("--out", type=Path)
        if action == "check":
            cmd.add_argument("--only", action="append", default=[])
            cmd.add_argument("--evidence-dir", type=Path, required=True)
        if action in ("activate", "delegate", "status"):
            cmd.add_argument("--ledger", type=Path, required=True)
            cmd.add_argument("--repository", required=True)
            cmd.add_argument("--pr", type=int, required=True)
        if action in ("activate", "delegate"):
            cmd.add_argument("--generation", type=int, required=True)
        if action == "delegate":
            cmd.add_argument("--adapter", required=True)
            cmd.add_argument("--slot", choices=("review", "adjudication"), default="review")
            cmd.add_argument("--evidence", type=Path, action="append", default=[])
            cmd.add_argument("--forward-env", action="append", default=[])
            cmd.add_argument("--override-reason")
            cmd.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    try:
        if hasattr(args, "pr") and args.pr < 1:
            raise ValueError("PR must be positive")
        if args.action == "status":
            with transaction(args.ledger) as conn:
                write_json(history(conn, (args.repository, args.pr)))
            return 0
        repo = Path(git(args.repo.resolve(), "rev-parse", "--show-toplevel").decode().strip())
        plan, checks = build_plan(repo, args.base, args.head)
        if args.action == "plan":
            write_json(plan, args.out)
        elif args.action == "check":
            result = run_checks(repo, plan, checks, args.only, args.evidence_dir)
            write_json(result, args.out)
            return int(any(r["status"] != "pass" for r in result["checks"].values()))
        else:
            if args.generation < 1:
                raise ValueError("generation must be positive")
            args.ledger = args.ledger.resolve()
            if args.action == "activate":
                with transaction(args.ledger) as conn:
                    active = activate(conn, (args.repository, args.pr), args.generation, plan["digest"])
                write_json({"state": "active" if active else "superseded", "plan": plan["digest"]})
            else:
                args.command = args.command[1:] if args.command[:1] == ["--"] else args.command
                if not args.command:
                    raise ValueError("delegate needs a caller-owned adapter command after --")
                if args.slot == "adjudication" and not (args.override_reason or "").strip():
                    raise ValueError("an adjudication requires an override reason")
                write_json(delegate(args, repo, plan))
    except (ValueError, KeyError, TypeError, OSError, sqlite3.Error) as error:
        print(f"assessment: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    def interrupt(*_):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, interrupt)
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit(130)
