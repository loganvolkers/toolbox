# Standards Lifecycle A/B: Writing, Sharing and Running a Standard

**Question** ([repo-value.md](../../docs/design/repo-value.md) R3, second round): over the life of a
standard (writing it, enforcing it with a ratchet, running a campaign on it, changing it, reusing it
in another repository, and letting another agent read its state), does Toolbox cost an agent fewer
tokens and less time than bespoke scripts, with at least the same correctness?

**Status: designed, not run.** The first A/B ([../out-of-box/ab](../out-of-box/ab/README.md)) asked an
agent to move findings of linters it could run directly, and found no difference. That measures the
linter, not Toolbox. Toolbox's claim is about everything around the check: one format for rules,
evidence, baselines and campaigns, which other repositories and other agents can use without reading
anyone's code. MCP and LSP make the same kind of claim: neither does anything a custom tool couldn't,
but both save work because every consumer speaks them.

## Why these standards

Each is written down in a real repository's `AGENTS.md`, so a team already decided it matters, and
none is fully enforced:

| # | Repository (fork) | The written standard | Today |
|---|---|---|---|
| S1 | PostHog | "Never add new direct ORM queries like `Person.objects.filter(...)` … use the existing routed helpers" (`AGENTS.md`) | 86 occurrences in non-test Python. The routed helpers themselves (`posthog/models/person/util.py`, `person.py`) are allowed, so the standard needs exemptions. Nothing in CI enforces it |
| S2 | Codex | "Target Rust modules under 500 LoC, excluding tests. If a file exceeds roughly 800 LoC, add new functionality in a new module" (`AGENTS.md`) | About 237 Rust files over 800 lines. No enforcement |
| S3 | OpenFGA | S2, reused for Go with different thresholds (600 and 1,000 lines) | 16 Go files over 1,000 lines |

**The bespoke baseline exists:** PostHog built its own ratchet for a similar campaign,
`.github/scripts/check-idor-model-coverage.py` (740 lines), with a committed
`baseline_unmigrated.txt`, a `--regenerate-baseline` flag, and removals reported "as opportunities
to shrink the baseline". It also has its own workflow linter (`tools/hogli-commands/.../workflow_lint`,
737 lines), whose `job_timeouts` and `pr_concurrency` checks duplicate two rules in
`packages/github-actions`. That is what arm A is expected to rebuild, and what Toolbox claims to
make unnecessary.

## Arms

| | Arm A: bespoke | Arm B: Toolbox |
|---|---|---|
| Given | The standard's text, Python, the shell, git | The same, plus `toolbox` and pointers to its manifest, campaign and package docs |
| Writes | Whatever it chooses: a script, a baseline file, a CI step | A portal (`toolbox.yaml`), a baseline (`--baseline`), a campaign, and a package for S3 |
| Budget | 40 tool calls per phase; no network | The same |

Every phase starts a fresh agent (same model) with only the repository and the artifacts earlier
phases left, the way a different person or agent meets a standard later. Arm B's first phase includes
the cost of learning Toolbox.

## Phases

| Phase | Task | Scored by |
|---|---|---|
| 1. Write | S1 on PostHog and S2 on Codex: report every violation per file, fail CI on a new one, and report campaign progress (baseline, current, percentage to target) | Precision and recall against an independent count |
| 2. Gate | Scripted, no agent: on top of each arm's result, (a) add a new violation, (b) fix an existing one, (c) make an unrelated edit, and run the arm's gate on each | Right verdict: fail, pass and report the improvement, pass |
| 3. Change | "Exempt this file, with a reason. Add `Group.objects` to S1. Tighten S2's 800 lines to 700" | Correct after the change; cost |
| 4. Reuse | Apply S2 to OpenFGA's Go code with thresholds of 600 and 1,000 lines (S3) | Correct counts; how much of phase 1 was reused |
| 5. Read | Answer five questions from the artifacts alone: violations left, top three files, did the last change regress, campaign percentage, how to fix one violation | Accuracy |

**Measured in every phase:** tokens, tool calls and wall time (from the agent's run), correctness,
and the lines of code and configuration the team would then maintain.

## Result that would stop us

Toolbox should cost more in phase 1 (learning) and less in phases 3 to 5. **If arm B's total tokens
across all five phases aren't lower than arm A's, at equal or better correctness, R3 fails:**
Toolbox's value for a single repository is its packaged standards, not the tool, and the plan ships
standards as text.

## Limits

- One run per phase and arm, on two standards: indicative, not significant.
- Arm B may hit gaps in Toolbox. For example, mapping each file to its CODEOWNERS owner isn't
  built. A gap counts against arm B, and is also a finding.

## Sources

- The forks' `AGENTS.md` files: PostHog (lines on person data access and on `team_id`), Codex
  (module size)
- PostHog's `.github/scripts/check-idor-model-coverage.py` and `tools/hogli-commands/hogli_commands/workflow_lint/`
- [../out-of-box/ab](../out-of-box/ab/README.md): the first A/B, with its token and time costs
