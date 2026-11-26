# Standards Lifecycle A/B: Writing, Sharing and Running a Standard

**Question** ([repo-value.md](../../docs/design/repo-value.md) R3, second round): over the life of a
standard (writing it, enforcing it with a ratchet, running a campaign on it, changing it, reusing it
in another repository, and letting another agent read its state), does Seikika cost an agent fewer
tokens and less time than bespoke scripts, with at least the same correctness?

**Answer (2026-11-21, one run per phase and arm): no, and R3 is closed as failed
([D01M7Z2ZZ97VZQNTDT4QQHM3HQD](../../docs/decisions/D01M7Z2ZZ97VZQNTDT4QQHM3HQD.md)). After the
measured friction was fixed and the Seikika arm rerun (B′), Seikika cost 15% more tokens than
bespoke scripts over the whole lifecycle (510k against 445k), at equal correctness. The bar
set before the rerun was "fewer tokens".**

| | Arm A: bespoke scripts | Arm B: Seikika, round 2 | Arm B′: Seikika after the fixes |
|---|---|---|---|
| Correct (reports, 18 gate cases, 5 questions) | All | All | All |
| Tokens, all phases | **445k** | 582k | 510k |
| Tool calls | **57** | 125 | 96 |
| Wall time | **10 min** | 41 min | 17 min |
| Tokens, phases 3 to 5 (after writing) | **275k** | 310k | 284k |
| Code and configuration the team maintains, per repository | 204 to 250 lines | 130 to 237 lines | **77 to 92 lines**, plus a vendored package |

- **The fixes closed most of the gap:** 12% fewer tokens than round 2, 23% fewer tool calls, and
  59% less time. After the standard is written, Seikika is within 3% of the scripts on tokens.
  In the reuse phase, where B′ included the same package with OpenFGA's parameters, it used
  fewer calls (11 against 12) and less time (98 s against 105 s) than arm A copying its script.
- **What remains is learning the format.** Writing the first standard cost B′ 55k tokens more
  than writing a script (226k against 171k), with three times the tool calls. A frontier agent
  writes a 200-line counting script cheaply, and there is no later phase where a script costs much
  more.
- **Tokens are a blunt measure here.** Every agent starts with about 85k tokens of fixed context,
  so the phases after the first differ by a few thousand tokens. Calls and time separate the arms
  more clearly, and on those B′ is close to the scripts after phase 1, not ahead.
- **Two things Seikika did better are outside the bar:** about a third as much code to maintain,
  and reuse by `include` instead of copying a script. The first A/B's guardrail finding (no
  regressions, no suppressions) also stands. None of them was the pre-registered question.
- **A design error in round 2:** PostHog already enforces S1 in CI, with a semgrep rule and inline
  `nosemgrep` suppressions (65 of the 80 occurrences). Neither arm found it. The new
  [`semgrep` package](../../packages/semgrep/README.md) reads that rule: 104 suppressed findings in
  33 files, a campaign PostHog doesn't have.

The round 2 results and their friction table follow; the rerun is under *Rerun (B′)*.

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

| | Arm A: bespoke | Arm B: Seikika |
|---|---|---|
| Given | The standard's text, Python, the shell, git | The same, plus the `seikika` binaries and pointers to its docs, packages and starter |
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
| PostHog | 204 (script 180, CI 24); 85 docs; 164 baseline | 237 (`seikika.yaml` 86, wrapper `s1.sh` 98, CI 53); 94 docs; `results.json` 26,960 lines (948 KB) plus a 153-line campaign start |
| Codex | 250 (script 210, config 6, CI 34); 118 docs; 236 baseline | 130 (`seikika.yaml` 70, measuring script 25, CI 35) plus a 261-line hand-frozen campaign list; 128 docs; `results.json` 13,798 lines (444 KB) |
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
| 4 | Campaign status isn't in `report -o json`, and `seikika campaign` needs a server. Arm B read `report.json` out of an `-o web` directory | P1, P4 | Missing feature |
| 5 | No frozen campaign start. Arm B wrote a start file from its wrapper, or pasted 229 paths into `appliesTo`. There is no freeze or restart command | P1, P3, P4 | Missing feature |
| 6 | No way to accept an intended change. `--update` refuses regressions, so tightening a standard meant deleting `results.json`, which also accepts any real regression mixed in. Arm B diffed old and new with `jq` by hand | P3 | Missing feature: accept with a summary of what changed |
| 7 | No portal parameters, and no size or regex-count package. Reuse means copying and substituting literals in YAML, scripts, `jq` and docs | P1, P4 | Missing feature |
| 8 | No `reason` field on an exemption. A `#` reason inside a folded CEL block would become CEL text | P3 | Missing feature |
| 9 | Scorecards reject `title` and `description` | P1 | Missing feature |
| 10 | The baseline records every entity: 948 KB for PostHog | P1 | Design |
| 11 | No reference for the workspace connector's configuration. Agents read `crates/seikika-connector-workspace/src/main.rs` | P1, P4 | Docs |
| 12 | `examples/repo-portal-starter` has no README and no regex-count or file-size example | P1, P4 | Starter |
| 13 | `root:` and the report action's install path aren't documented | P1 | Docs |
| 14 | No guidance on changing a standard mid-campaign (keep the scope, or restart it) | P3 | Docs |
| 15 | `seikika describe \| head` panics on a broken pipe | P1 | Bug |
| 16 | The campaign percentage rounds down | P1 | Bug |
| 17 | Whether `seikika validate` catches a stray `#` in CEL wasn't tested | P3 | Unverified |
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
  Seikika's, and the reader did no better with Seikika's format.
- **Sharing was not tested.** Phase 4 was copy against copy, because the package mechanism can't
  yet express "a file-size standard with these thresholds" or "count this regex outside these
  paths". So this round can't separate the strategy from the implementation on reuse. On writing,
  changing and reading, it did measure the implementation, and the implementation lost.
- **The semgrep finding is the most direct evidence for the strategy's opportunity, and it
  changes the target.** Teams already encode standards in tools (semgrep, linters, custom
  scripts), and they keep the baseline as inline suppressions or bespoke files. What they don't
  have is the counted, per-file, over-time campaign. Seikika's job may be to ingest those tools'
  output and suppressions as the evidence and baseline, rather than to re-express the rule. A
  runner parser for semgrep JSON would make PostHog's existing rule the source, and its 34 files of
  suppressions the campaign.

## Rerun (B′)

**What changed between the rounds** ([D01M7Z2ZZ97VZQNTDT4QQHM3HQD](../../docs/decisions/D01M7Z2ZZ97VZQNTDT4QQHM3HQD.md)):
round 2's friction items 1 to 9, 11, 12, 14 to 16 and 18 were fixed in Seikika. That includes a
rule's `measure`, `-o failures`, campaign paydown from a recorded start, `--accept`, and the
`file-size`, `pattern-count` and `semgrep` packages, with a connector reference and starter docs.
Before the rerun, the package and starter examples were made generic, because they had been
written after seeing these standards.

**How it ran:** the same prompts as arm B ([rerun/prompts/](rerun/prompts/)), on fresh copies at
the same commits, with the same truth, gate harness and phase 5 setup, which was checked byte for
byte. Two changes:
- The agents could read only Seikika's `README.md`, `docs/v1-spec.md`, `docs/design/`,
  `docs/reference/`, `examples/`, `packages/`, `crates/`, `schema/` and `plugins/`. This repository
  now holds the ground truth and the answer key, under `experiments/` and in the research log.
- The first phase 1 launch was cut off by an account usage limit partway through. Its partial
  work was discarded, and phase 1 was relaunched on clean copies.

The diffs are in [rerun/patches/](rerun/patches/) (`results.json` left out), and usage in
[rerun/usage.tsv](rerun/usage.tsv).

| Phase | Arm A | Arm B | Arm B′ |
|---|---|---|---|
| 1. Write S1 and S2 | 171k, 18 calls, 252 s | 272k, 74, 1,526 s | 226k, 51, 568 s |
| 3. Change | 99k, 17, 161 s | 112k, 22, 644 s | 103k, 23, 239 s |
| 4. Reuse on OpenFGA | 90k, 12, 105 s | 106k, 17, 133 s | 92k, 11, 98 s |
| 5. Read | 85k, 10, 72 s | 93k, 12, 152 s | 89k, 11, 102 s |
| **Total** | **445k, 57, 590 s** | 582k, 125, 2,455 s | 510k, 96, 1,007 s |

**Correctness:** B′ was exact everywhere:
- S1: 28 of 28 files, 37 of 37 after the change;
- S2: 191 of 191, 229 of 229 at 700 lines;
- S3: 15 of 15;
- all 18 gate cases;
- 5 of 5 answers.

Its reader also caught the synthetic fake fix, and said the campaign's 2.9% wasn't real.

**How B′ worked:**
- Both phase 1 agents included a package (`pattern-count`, `file-size`) and wrote 25 to 30 lines
  of `seikika.yaml`.
- Phase 3 changed one parameter in each repository and ran `--update --accept <scorecard>`, which
  restarted the campaigns.
- Phase 4 copied Codex's portal, changed 13 lines of `seikika.yaml`, and reused the package
  unchanged.

**Friction B′ still reported:**

| Friction | Phases | Kind |
|---|---|---|
| Including a package offline means copying it into the repository. `seikika install` fetches `github:` packages over the network, and there is no vendoring command | P1, P4 | Missing feature |
| The default Markdown lists every passing entity (6,331 files on PostHog), which overflows an agent's tool output. A quiet gate needs `--out /dev/null` | P1 | Design |
| `-o failures` needs a `jq` reshape for a given output shape | P1, P4 | Missing feature (field selection) |
| Campaign progress is under `.campaigns[…].sinceStart` in `-o json`; `seikika campaign` needs a server. The top-level `percent` read 100 while 28 files failed (6,331 of 6,359, rounded) | P1 | Missing feature, and a bug this change introduced: rounding must not reach 100 before every entity meets the goal |
| No copy-paste CI workflow that installs Seikika. The report action's README is under `actions/`, which the rerun's read list left out | P1, P4 | Experiment's read list, and docs |
| `--ci` fails on an unrecorded improvement, while the task said improvements pass | P1 | Docs (two modes, two uses) |
| The baseline records every clean file, so a pull request that adds a clean file makes it stale | P1 | Design (round 2's item 10) |
| `tighten: lower` isn't enforced for a parameter without a default, so raising a required limit passes | P1 | Bug |
| An exempted file is reported as `✓ better … now passes`, and the campaign counts it as fixed | P1 | Design |
| Whether `**/` matches zero directories wasn't documented | P1 | Docs |
| Guidance on keeping or restarting a campaign is in `campaigns.md`, not in the package README | P3 | Docs |
| `--accept` lists 229 changes without separating newly failing files from files the new limit shifted | P3 | Missing feature |
| STANDARD.md and the workflow were hand-copied between repositories | P4 | Starter |

**Friction neither arm escaped:**
- `AGENTS.md` doesn't point to the tooling.
- The gate compares against the baseline, not the previous commit.
- A textual pattern can be gamed by renaming.

## Outcome

R3 is closed as failed. With its friction fixed, Seikika matched bespoke scripts after a standard
was written, and on reuse, but didn't beat them, and learning the format costs more than writing
a script. For a single agent applying simple textual standards to one repository, Seikika saves
no tokens. The stop condition's consequence: the single-repository value claim rests on the
packaged standards, not on agent efficiency.

**What the evidence still supports**, none of it R3's question:
- packages that turn a written standard into a ratchet and a campaign in about 30 lines;
- about a third as much code to maintain;
- reuse by `include`;
- the guardrail from the first A/B (R4 tests that on real pull requests).

## Limits

- One run per phase and arm, on three standards: indicative, not significant. Per-phase token
  differences of a few thousand are within what one rerun of the same agent would vary by.
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
- [prompts/](prompts/), [patches/](patches/), [harness/](harness/), [usage.tsv](usage.tsv): round 2
- [rerun/prompts/](rerun/prompts/), [rerun/patches/](rerun/patches/), [rerun/usage.tsv](rerun/usage.tsv): the rerun (B′)
- [../out-of-box/ab](../out-of-box/ab/README.md): the first A/B, with its token and time costs
