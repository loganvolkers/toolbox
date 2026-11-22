# Phase 6, arm B: the agent's reply (scored against 6-B-key.json; DX feedback at the end)

**Score against the key: all correct.** Tools that failed: semgrep and zizmor. New findings: semgrep posthog/api/person.py:1566 no-direct-persons-db-orm (a new nosemgrep-suppressed Person.objects; suppressed findings count; file 2 → 3); zizmor .github/workflows/ci-python.yml:29 unpinned-uses and :29 artipacked (the new `uses: actions/checkout@v4`; the old checkout's artipacked shifted from 29 to 32 and is not new; file 4 → 6). Reproduce: semgrep and zizmor commands from TOOLS.md and the log's `reproduce:` lines, plus narrowed one-file variants (unrun). Campaigns: semgrep-zero 209 → 200, paid 9 (4 files); zizmor-zero 336 → 338 (−2); ruff-zero 28 → 28; actionlint-zero 3 → 3; job-timeouts-zero 0 → 0.

Sources: last-pr-gate.log, git show HEAD, TOOLS.md, .toolbox/results.json, the rule file. Changed nothing.

DX feedback:
1. The gate's "worse" block doesn't mark which findings are new: it lists all 6 for ci-python.yml and all 3 for person.py, and the baseline holds only counts, so the diff had to be read. Mark new findings, or store a fingerprint per finding in results.json.
2. "no-findings" wording confuses: "no-findings evidence grew 4 → 6" and bullets labelled `no-findings:` read as if there were none; the tool's rule (unpinned-uses, no-direct-persons-db-orm) is buried in the message.
3. The `reproduce:` line always reruns the whole tool over every path, never the one file or rule.
4. The campaign line mixes the PR's regression into "now" (200 with it, 199 accepted); nothing says which "now" means.
5. `start: null` in results.json campaigns is unclear next to `measure` and `recorded`.
