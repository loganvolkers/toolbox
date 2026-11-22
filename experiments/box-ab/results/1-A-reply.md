# Phase 1, arm A: the agent's reply (DX feedback in §3)

1. Built: TOOLS.md; .github/scripts/quality_gate.py (~200 lines, stdlib: gate, accept, progress, campaign); .github/quality-gate/baseline.json (per tool, per file); .github/workflows/ci-quality-gate.yml (pip-installs semgrep 1.163.0 and ruff 0.14.11, actionlint via the pinned download script). Gate exit 0; tested with three planted regressions (exit 1, names file, counts and rules).
   Baseline: 212 findings in 65 files: semgrep 209 in 63 (no-direct-persons-db-orm 104 in 33, prefer-codegen-api 105 in 30, all suppressed), ruff 0, actionlint 3 in 2 (ci-backend.yml:674, ci-rust.yml:202, :382).
   semgrep command uses `--disable-nosem --json` so suppressed findings count.
2. Couldn't: run the workflow on GitHub (pip/curl steps unverified). Per-file counts mean a finding moving between files fails the gate.
3. DX feedback:
   1. semgrep's JSON with --disable-nosem doesn't mark which findings were suppressed (no extra.is_ignored); without it they vanish. Counting new suppressions means counting all findings.
   2. ruff bans print (T201) in the new gate script: the first `accept` baselined the script's own 11 ruff findings; rewrote to sys.stdout.write and re-accepted. A ratchet should be checked on its own files before the first accept.
   3. semgrep scans only git-tracked files ("Scan was limited to files tracked by git"): new untracked files are invisible until added.
   4. actionlint's results depend on the machine: it uses shellcheck when on PATH; GitHub runners have it, this environment doesn't, so the gate disables it; the existing ci-actionlint.yml runs with shellcheck, so counts differ.
   5. Exit codes differ per tool (semgrep 0 with findings unless --error; ruff and actionlint 1): the gate needs a per-tool success table.
   6. Minor: ruff warns on an invalid noqa at redshift.py:258; semgrep reports 4 parse errors with exit 0.
