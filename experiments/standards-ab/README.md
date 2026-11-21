# Standards Lifecycle A/B: Writing, Sharing and Running a Standard

**Question** ([repo-value.md](../../docs/design/repo-value.md) R3, second round): over the life of a
standard (writing it, enforcing it with a ratchet, running a campaign on it, changing it, reusing it
in another repository, and letting another agent read its state), does Toolbox cost an agent fewer
tokens and less time than bespoke scripts, with at least the same correctness?

**Answer (2026-11-21, one run per phase and arm): no. R3 fails as pre-registered, but the round
did not test sharing, which is the mechanism Toolbox claims.**

- **Correctness was equal.** Both arms were exact in every phase: every violating file, every
  count, every one of 18 scripted gate cases, and all five reading questions.
- **Toolbox cost more in every phase, not only while it was being learnt.** Over the five phases
  it took 582k tokens against 445k (+31%), 125 tool calls against 57, and 41 minutes against 10.
  After phase 1 the gap narrows but stays: +13% tokens and 2.7 times the wall time.
- **Reuse (phase 4) was copy-and-edit in both arms.** Toolbox has no parameterised package for a
  file-size or regex-count standard, so arm B copied the Codex files and substituted about a dozen
  literals, just as arm A did. The ecosystem effect, where one standard is written once and read
  by many consumers, was not available to measure.
- **Most of arm B's extra cost is friction with a named fix:** wrapper scripts, `jq` pipelines over
  internal JSON, hand-frozen campaign lists, deleting the baseline to apply an intended change, and
  40-second runs on PostHog. The table under *Developer experience* classifies 18 such items.
- **A design error:** PostHog already enforces S1 in CI. It runs a semgrep rule with `--error`,
  and inline `nosemgrep` comments act as the baseline (65 of the 80 occurrences). Neither arm found
  this, and neither did the design. Both rebuilt an existing gate. What PostHog lacks is the
  campaign: a count of those suppressions going to zero.

The stop condition says that if R3 fails, Toolbox's single-repository value is its packaged
standards, and the plan ships standards as text. **Whether to accept that, or to fix the friction
and rerun arm B, is open** (see *Next*).

## Why these standards

Each is written down in a real repository's `AGENTS.md`, so a team already decided it matters:

| # | Repository (fork) | The written standard | Measured (ground truth) |
|---|---|---|---|
| S1 | PostHog | "Never add new direct ORM queries like `Person.objects.filter(...)` … use the existing routed helpers" | 80 occurrences in 28 non-test files, after exempting the routed helpers (`posthog/models/person/util.py`, `group_type_mapping.py`). **Already enforced** by `.semgrep/rules/no-direct-persons-db-orm.yaml` in `ci-security.yaml`; 65 of the 80 carry `# nosemgrep`, and the rest are in paths that semgrep excludes and S1 does not (for example `migrations/`) |
| S2 | Codex | "Target Rust modules under 500 LoC, excluding tests. If a file exceeds roughly 800 LoC, add new functionality in a new module" | 191 `codex-rs` files over 800 lines, excluding `tests/`, `tests.rs` and `*_tests.rs`. Not enforced |
| S3 | OpenFGA | S2, reused for Go: target 600, violation over 1,000 lines, `_test.go` exempt | 15 files |

**Other bespoke ratchets in PostHog:** `.github/scripts/check-idor-model-coverage.py` (740 lines,
with a committed `baseline_unmigrated.txt` and `--regenerate-baseline`) and its own workflow linter
(`tools/hogli-commands/.../workflow_lint`, 737 lines). Its `job_timeouts` and `pr_concurrency`
checks duplicate two rules in `packages/github-actions`.

## Method

| | Arm A: bespoke | Arm B: Toolbox |
|---|---|---|
| Given | The standard's text, Python, the shell, git | The same, plus the `toolbox` binaries and pointers to its docs, packages and starter |
| Budget | 40 tool calls per phase; no network or installs | The same |

Every phase starts a fresh general-purpose agent on the same model. It sees only the repository and
what earlier phases committed there, as a different person or agent would meet the standard later.
The prompts are in [prompts/](prompts/); the arms' diffs are in [patches/](patches/) (arm B's
generated `results.json` is left out); [harness/](harness/) scores them:

| File | What it does |
|---|---|
| [truth.py](harness/truth.py) | Independent count per standard, written before any arm ran |
| [score_report.py](harness/score_report.py) | Precision, recall and exact counts of an arm's report against the truth |
| [gate_test.py](harness/gate_test.py) | Phase 2: applies each scripted change on top of the arm's commit, runs the arm's own gate command (from its STANDARD.md), and resets |
| [phase5-key.md](harness/phase5-key.md) | The answers to phase 5 |

Tokens, tool calls and duration are the agent runtime's own counts ([usage.tsv](usage.tsv)).

| Phase | Task |
|---|---|
| 1. Write | S1 on PostHog and S2 on Codex: report every violation per file, a CI gate that fails on a new one, and campaign progress |
| 2. Gate | Scripted, no agent, run after phases 1, 3 and 4: a new violation, a violator growing, a fix, a file crossing the threshold, an unrelated edit |
| 3. Change | Add `Group` and `GroupTypeMapping` to S1; exempt `posthog/models/person/person.py` with a reason; tighten S2 from 800 lines to 700 |
| 4. Reuse | S3: apply the Codex standard to OpenFGA, reusing the Codex artifacts |
| 5. Read | On top of two synthetic commits ([phase5-setup.patch](patches/phase5-setup.patch): one "fix" and one regression), answer five questions from the team's artifacts alone |

## Results

### Correctness

| Phase | Measure | Arm A | Arm B |
|---|---|---|---|
| 1 | S1 report (28 files) and S2 report (191 files) | Exact | Exact |
| 3 | S1 after the change (37 files, 102 occurrences) and S2 at 700 lines (229 files) | Exact | Exact |
| 4 | S3 report (15 files) | Exact | Exact |
| 2 | Gate verdicts: 7 cases after phase 1, 7 after phase 3, 4 after phase 4 | 18 of 18 | 18 of 18 |
| 5 | Violations left (99 in 35 files), top three, did HEAD regress (+3 in two files), campaign percentage (2.9%), how to fix one violation (`get_person_by_id`, and handle `None`) | 5 of 5 | 5 of 5 |

In phase 5, **both readers noticed that the synthetic "fix" was fake.** It renamed six calls to
names that don't exist. Both said the campaign's 2.9% was really a step backwards. The textual
standard can be gamed whichever tool runs it.

### Cost

| Phase | Arm A: tokens, calls, time | Arm B: tokens, calls, time |
|---|---|---|
| 1. Write S1 | 85k, 10, 127 s | 133k, 35, 1,020 s |
| 1. Write S2 | 85k, 8, 125 s | 139k, 39, 506 s |
| 3. Change | 99k, 17, 161 s | 112k, 22, 644 s |
| 4. Reuse | 90k, 12, 105 s | 106k, 17, 133 s |
| 5. Read | 85k, 10, 72 s | 93k, 12, 152 s |
| **Total** | **445k, 57, 590 s** | **582k, 125, 2,455 s** |
| Phases 3 to 5 | 275k, 39, 338 s | 310k, 51, 929 s |

Every agent starts with a fixed context. The cheapest run, arm A's phase 5 at 10 calls, took 85k
tokens, so token totals understate the relative difference; tool calls and time show it more
sharply. Phase 3's gap is mostly the baseline reset (below) plus PostHog runs of about 40 s each.

### What the team maintains afterwards

Lines of code and configuration, then documentation, then generated or recorded state:

| Repository | Arm A | Arm B |
|---|---|---|
| PostHog | 204 (script 180, CI 24); 85 docs; 164 baseline | 237 (`toolbox.yaml` 86, wrapper `s1.sh` 98, CI 53); 94 docs; `results.json` 26,960 lines (948 KB) plus a 153-line campaign start |
| Codex | 250 (script 210, config 6, CI 34); 118 docs; 236 baseline | 130 (`toolbox.yaml` 70, measuring script 25, CI 35) plus a 261-line hand-frozen campaign list; 128 docs; `results.json` 13,798 lines (444 KB) |
| OpenFGA | 250, of which about 49 lines changed from Codex; 113 docs; 22 baseline | 128 plus a 46-line campaign list, of which about 59 lines changed from Codex; 116 docs; `results.json` 2,854 lines |

Arm B's declarative part is smaller, but it still needed a script per standard: a regex-count
wrapper for PostHog, and a line-number lister for Rust and Go. Its baseline records every entity,
passing ones included, and is two orders of magnitude larger, which makes the pull-request diffs
that update it harder to review.

## Developer experience

The friction arm B's agents reported, classified by what would fix it. "P" is the phase.

| # | Friction | Phases | Kind |
|---|---|---|---|
| 1 | The ratchet compares pass or fail and evidence lists, not a number. To catch "a violating file grew by one line", arm B emitted one evidence item per line over the limit (`lineNumbers`) from its own script. That is also why `results.json` is so large | P1, P4 | Missing feature: ratchet on a declared numeric field |
| 2 | A new entity that fails is a "change", not a regression, unless `newEntitiesAtLeast` is set. Arm B's PostHog wrapper checks for it separately | P1, P3 | Missing feature (a default) |
| 3 | No flat "failing entities" output. Every report command is a `jq` pipeline over internal JSON, such as `.scores[…].evidence[…][0] \| split(" ")` | P1, P4 | Missing feature |
| 4 | Campaign status isn't in `report -o json`, and `toolbox campaign` needs a server. Arm B read `report.json` out of an `-o web` directory | P1, P4 | Missing feature |
| 5 | No frozen campaign start. Arm B wrote a start file from its wrapper, or pasted 229 paths into `appliesTo`. There is no freeze or restart command | P1, P3, P4 | Missing feature |
| 6 | No way to accept an intended change. `--update` refuses regressions, so tightening a standard meant deleting `results.json`, which also accepts any real regression mixed in. Arm B diffed old and new with `jq` by hand | P3 | Missing feature: accept with a summary of what changed |
| 7 | No portal parameters, and no size or regex-count package. Reuse means copying and substituting literals in YAML, scripts, `jq` and docs | P1, P4 | Missing feature |
| 8 | No `reason` field on an exemption. A `#` reason inside a folded CEL block would become CEL text | P3 | Missing feature |
| 9 | Scorecards reject `title` and `description` | P1 | Missing feature |
| 10 | The baseline records every entity: 948 KB for PostHog | P1 | Design |
| 11 | No reference for the workspace connector's configuration. Agents read `crates/toolbox-connector-workspace/src/main.rs` | P1, P4 | Docs |
| 12 | `examples/repo-portal-starter` has no README and no regex-count or file-size example | P1, P4 | Starter |
| 13 | `root:` and the report action's install path aren't documented | P1 | Docs |
| 14 | No guidance on changing a standard mid-campaign (keep the scope, or restart it) | P3 | Docs |
| 15 | `toolbox describe \| head` panics on a broken pipe | P1 | Bug |
| 16 | The campaign percentage rounds down | P1 | Bug |
| 17 | Whether `toolbox validate` catches a stray `#` in CEL wasn't tested | P3 | Unverified |
| 18 | Each PostHog run takes about 40 s: the workspace scan is 36 s, against under 2 s for arm A's script. Arm B's reader waited on four runs | P1, P3, P5 | Performance |

**Friction both arms hit:**
- **`AGENTS.md` doesn't point to the tooling.** Both readers found it through `git log`.
- **No per-commit comparison.** The gate compares against the baseline. Both readers worked out
  what the last commit did from its diff; arm B's reader also gated an exported `HEAD~1`.
- **Neither found PostHog's semgrep rule.** Arm A's phase 4 also asked whether OpenFGA's generated
  mock and test helpers should be exempt, a decision the prompt didn't make.

## Reading it

- **This round measured one consumer per artifact: an agent with a shell.** That is where a
  bespoke script is cheapest. MCP's and LSP's value appears with many consumers: editors, CI, UIs
  and other repositories speaking one protocol. Here the readers read the STANDARD.md and a JSON
  report command in both arms. A bespoke script's output turned out to be just as readable as
  Toolbox's, and the reader did no better with Toolbox's format.
- **Sharing was not tested.** Phase 4 was copy against copy, because the package mechanism can't
  yet express "a file-size standard with these thresholds" or "count this regex outside these
  paths". So this round can't separate the strategy from the implementation on reuse. On writing,
  changing and reading, it did measure the implementation, and the implementation lost.
- **The semgrep finding is the most direct evidence for the strategy's opportunity, and it
  changes the target.** Teams already encode standards in tools (semgrep, linters, custom
  scripts), and they keep the baseline as inline suppressions or bespoke files. What they don't
  have is the counted, per-file, over-time campaign. Toolbox's job may be to ingest those tools'
  output and suppressions as the evidence and baseline, rather than to re-express the rule. A
  runner parser for semgrep JSON would make PostHog's existing rule the source, and its 34 files of
  suppressions the campaign.

## Next

**For decision:**

1. **Accept the stop.** Ship standards as text, keep `toolbox report` as a convenience, and
   take repo-level value as not shown.
2. **Fix and rerun arm B (B′), with the bar set now.** Fix friction items 1 to 7, 11 and 12, add
   semgrep JSON to the runner connector, and rerun arm B's phases 1, 3 and 4 against the same
   truth and gate harness. Then add one phase that applies a shared package to all three
   repositories. **B′ passes only if its total tokens are lower than arm A's at equal correctness.
   If not, take option 1.**

Option 2 is recommended. Phase 4 did not test the mechanism Toolbox claims, and most of the
measured cost is listed friction rather than the format. Committing to the bar before building
keeps the rerun honest.

## Limits

- One run per phase and arm, on three standards: indicative, not significant.
- The standards are textual (a regex, a line count), the kind a bespoke script handles well. A
  standard that joins several sources, such as CODEOWNERS with file size, wasn't tested.
- The S3 truth first left out `tests/` directories, copying Codex's rule; the S3 prompt exempts
  only `_test.go`. That was corrected from 11 files to 15 after arm A's phase 4 report showed it,
  and before either arm was scored on S3.
- The design's own figures were off before measurement: "86 occurrences" for S1 (80 after the
  helper exemptions), "about 237" for S2 (191 after the test exemptions), "16" for S3 (15), and
  "nothing in CI enforces" S1.
- The phase 5 commits are synthetic, and the "fix" breaks the code. Both readers said so.

## Sources

- The forks' `AGENTS.md` files: PostHog (person data access), Codex (module size)
- PostHog's `.semgrep/rules/no-direct-persons-db-orm.yaml`, `.github/workflows/ci-security.yaml`,
  `.github/scripts/check-idor-model-coverage.py` and `tools/hogli-commands/hogli_commands/workflow_lint/`
- [prompts/](prompts/), [patches/](patches/), [harness/](harness/), [usage.tsv](usage.tsv): this round
- [../out-of-box/ab](../out-of-box/ab/README.md): the first A/B, with its token and time costs
