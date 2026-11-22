# The Box A/B: Several Tools, One Box or Hand-Written Glue

**Question:** when a repository already runs several tools, does keeping them in Toolbox's box cost
a coding agent less than the CI glue teams write by hand? The tasks are adding, upgrading, swapping,
ratcheting and reading tools, and both arms must keep each tool usable on its own.

**Status: ready to run, not run.** The setup, harness, prompts and answer key are built and
dry-run (§8). Still open: whether to add the hypothesis to
[repo-value.md](../../docs/design/repo-value.md) (§5), and the budget (§7).

This tests the box, not tool building ([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md)).
The standards A/B ([../standards-ab](../standards-ab/README.md)) compared building a tool with
Toolbox's primitives against writing a Python script, and Toolbox lost. That was the wrong half.
Teams rarely lack a linter. What they lack is one place that holds all of them: each tool's pinned
version, one baseline, whether a pull request made anything worse, and the jobs left.

## 1. What using the box on Toolbox for Toolbox showed

These lessons come from T4T's own use. Rows marked *built since* describe features added after
this plan was first written. They favour the box in the phases named, and the write-up must say so.

| What happened | Lesson for the A/B |
|---|---|
| Putting ruff in the box by hand took a 90-line standard; the [`sarif` kit](../../packages/sarif/README.md) made it a 15-line `include` | Adding a tool is where the box should win most (phase 3) |
| *Built since:* one pin per tool, with `toolbox tools install` and `tools config` (D01M800DHD0Z1RTPH7CM46GZS3B). The pin used to live in four places | Phase 4 upgrades a pin. Arm B has one place to change it |
| take-it-out passed first time for zizmor, actionlint, ruff and semgrep | Cheap to require of both arms: the harness checks both the same way (§6) |
| Adding a tool whose files already have findings would have dropped T4T's repository level | Phase 3 adds zizmor, which has 335 findings on day one |
| *Built since:* root causes (D01M7ZZYZ1SXSTHDZQK8A1CZMMN). A regression names its chain down to a file, its evidence and a command that reproduces it | Phase 6 asks why a gate failed. This favours the box by design |
| *Built since:* `toolbox tools find` and the `find-tools` skill (D01M801A9QNVCSBBJATYV721TCS) | Phase 1 may start from it |
| A stricter package check broke an old vendored kit on upgrade | Phase 4 upgrades a tool. Kits stay at one version through the run |
| `--against` reruns every connector in a worktree, so a repository-relative binary failed there | Tools are on the `PATH` (`$AB_HOME/bin`), never built in the repository |
| semgrep's SARIF lists only files with findings, so a fixed file disappears | Phase 2's paydown removes findings; both arms must count that as progress |
| **Found preparing this run:** with the network closed, semgrep 1.163.0's startup version check stalls for about 190 s per run, and once for 9 minutes despite `SEMGREP_ENABLE_VERSION_CHECK=0`. `--disable-version-check` fixes it, but semgrep refuses that flag twice | Both arms get the same `semgrep` wrapper, so the stall is not a variable. The semgrep kit now passes the flag itself |

## 2. Setting

**The PostHog fork** at `153f1b8d` (2026-06-01), measured by
[`harness/ab.py truth`](harness/ab.py), which runs each tool directly:

| Tool | Version | How CI runs it | Findings | Files |
|---|---|---|---|---|
| semgrep | 1.163.0 (CI's image) | `.semgrep/rules/no-direct-persons-db-orm.yaml` and `prefer-codegen-api.yaml` over CI's paths | 209, all suppressed inline | 63 |
| ruff | 0.14.11 (the pyproject pin) | `ruff check .` | 0 | 0 |
| ruff | 0.15.0 (phase 4's upgrade) | the same | 28 (UP042) | 19 |
| actionlint | 1.7.12 (CI's pin) | without shellcheck, which isn't installed | 3 | 2 |
| zizmor | 1.32.0 | offline over `.github/`, default audits (phase 3 adds it) | 335 | 87 |
| hogli WF001 | the repository's own | `lint:workflows` job timeouts (phase 5 swaps it) | 0 | 0 |

**Scope, and why it changed from the first draft:**
- **semgrep runs two of PostHog's 26 local rules.** All local rules take 129 s a run, and CI's
  registry packs (`p/python` and others) need the network. The two rules take 17 s.
- **mypy, oxlint, stylelint and prettier are out.** mypy has no SARIF output, oxlint isn't
  installed, and prettier and stylelint are formatters.
- **ruff is clean at PostHog's pin**, so phase 2's campaign is semgrep's alone.
- **The first draft's phase 5 is replaced.** It swapped `check-idor-model-coverage.py`, which needs
  Django and PostHog's full backend to run, so it can't be scored here. Phase 5 now swaps hogli's
  WF001 instead. hogli runs through a shim that supplies its two imports.

## 3. Arms

| | Arm A: glue | Arm B: the box |
|---|---|---|
| Given | The repository, the tools on `$AB_HOME/bin`, Python and the shell | The same, plus `toolbox` on `$AB_HOME/box-bin` |
| May read | The repository only; not Toolbox | Also Toolbox's `README.md`, `docs/v1-spec.md`, `docs/design/`, `docs/reference/`, `examples/`, `packages/`, `crates/`, `schema/` and `plugins/` (B′'s allowlist) |
| Never | `$AB_HOME` beyond running its binaries; `experiments/`; `docs/research-log.md` | The same |
| Builds | Whatever it chooses | A portal, with vendored kits |

**Answer leaks removed before the run:** `packages/semgrep` named phase 2's rule and its counts,
`toolbox-and-tools.md` quoted the kit's PostHog numbers, and the catalog named the fork. All are
generic now. What remains is qualitative: that PostHog runs semgrep, and the tools
`finding-tools.md` found. Arm A sees both in PostHog's own CI.

**One interface for both arms:** a `TOOLS.md` at the repository root with a JSON block holding
`gate`, `accept`, `progress` and, per tool, a command that runs the tool alone. The harness reads
only that block, so neither arm is scored through the other's format. It doesn't count as glue.

## 4. Phases

Each phase starts a fresh general-purpose agent, on the same model, with the repository and
what earlier phases committed. Prompts are in [prompts/](prompts/), rendered by
[`prompts/make.py`](prompts/make.py). `--check` confirms that the two arms' prompts differ only in
the setup block, the `PATH` line and their directory.

| # | Task | Scored by (harness) |
|---|---|---|
| 1 | **Gather** semgrep, ruff and actionlint under one gate. Report each tool's findings | `cases 1`, `parity 1`, the reply against §2 |
| 2 | **Campaign:** pay semgrep's 209 suppressed findings down, from a recorded start | `cases 2`. Then `progress-fixes` applies 10 fixes, and the arm's `progress` must say start 209, now 199, paid 10 |
| 3 | **Add zizmor**, whose 335 findings must not fail CI while new ones do | `cases 3`, `parity 3` |
| 4 | **Upgrade ruff** to 0.15.0. Accept its 28 new findings for ruff only | `cases 4` (a new UP042 fails, and so does a semgrep regression), `parity 4` (28 at 0.15.0) |
| 5 | **Swap** hogli's WF001 for a semgrep rule, tool `job-timeouts` | `cases 5`. `swap` compares the rule with hogli's verdict on four states, including the reusable-call exemption |
| 6 | **Read:** which tool failed the last pull request, where, why, how to reproduce it, and campaign progress | `last-pr` commits the pull request and writes the key. The reply is scored against it |

**Planted changes** ([`ab.py`](harness/ab.py) `CASES`): each pull-request change moves exactly one
tool, checked by the dry run (§8). A case's expected verdict isn't written by hand. A case should
fail exactly when some tool in scope now has more findings in some file, measured by running the
tools directly.

**Measured per phase:**
- tokens, tool calls and time, from the agent's result, into `usage.tsv`;
- glue added, from `ab.py glue`: scripts, workflow steps and portal manifests. Docs, the tools'
  own configs and rules, baselines and vendored kits don't count;
- correctness: cases, parity, swap and the key.

## 5. The bar, set before running

The hypothesis is about the box: **several tools in a box cost less to maintain than glue.**
- **The box passes** if, over phases 1 to 6, it uses fewer tokens and leaves less glue than arm A,
  with equal correctness, and both arms pass take-it-out.
- **It fails** if either measure is worse. Then the box's value is the format it standardises,
  not the work it saves, and the plan should say so.

**Open:** adding this hypothesis to [repo-value.md](../../docs/design/repo-value.md) is a decision
to make before the run.

## 6. Harness

| File | Does |
|---|---|
| [`harness/setup.sh`](harness/setup.sh) | Rebuilds everything under `$AB_HOME` (default `/tmp/box-ab`) from nothing: the pinned tools, the semgrep wrapper, Toolbox (release build) and the fork at `153f1b8d`. Idempotent, about 3.5 minutes |
| [`harness/ab.py`](harness/ab.py) | `truth`, `cases`, `parity`, `swap`, `glue`, `progress-fixes`, `last-pr`, and `arm init/commit/restore` for copies and checkpoints |
| [`harness/ref.py`](harness/ref.py) | A reference implementation of the interface, only to dry-run the harness |
| [`prompts/make.py`](prompts/make.py) | The twelve prompts from one template |

The harness never reads a tool's error as "no findings". Each tool has its allowed exit codes, and
an error stops the run. This was found when semgrep's usage message once read as zero findings.

## 7. Runbook

```bash
H="python3 -I experiments/box-ab/harness/ab.py"
experiments/box-ab/harness/setup.sh                  # after any container restart
python3 -I experiments/box-ab/prompts/make.py --check
$H arm init A && $H arm init B                       # fresh copies at the base commit
```

**Per phase `p`, for both arms in parallel:**
1. Phase 6 only, first: `$H last-pr <arm dir> > key-6-<arm>.json`.
2. Launch a fresh general-purpose agent with `prompts/<p>-<arm>.md`.
3. Record its tokens, tool calls and time in `usage.tsv`.
4. `$H arm commit <arm> <p>`. Every phase ends in a tag, `phase-<p>`.
5. Score: `$H cases <arm dir> <p>`, `$H parity <arm dir> <p>` and `$H glue <arm dir>`; phase 5 also
   `$H swap`. Save each to `results/<p>-<arm>-*.json`. `cases` reruns every tool twice per
   planted change, about 9 minutes per arm at phase 6: run it in the background while the next
   phase's agents work on their own copies.
6. Phase 2 only, after scoring: `$H progress-fixes <arm dir>`, then run the arm's `progress`
   command and save its output.

**If a usage limit cuts a phase off:** discard it, `$H arm restore <arm> <p-1>`, and relaunch that
phase for that arm. Never resume a half-done agent. Last round, a cut-off phase 1 was discarded
the same way.

**Budget:** 40 tool calls per phase, as last round (arm B used 74 in last round's phase 1).
Expect about 2M tokens for twelve agent runs. Optionally run phase 1 twice per arm, its most
variable phase, for roughly 0.4M more.

## 8. Dry run

Run on a throwaway copy (`$H arm init dry`). `cases <copy> 6 true` uses a gate that always passes,
so only the `worse` column matters:

| Planted change | Tools made worse |
|---|---|
| semgrep: a new suppressed finding | semgrep |
| ruff: a new finding | ruff |
| actionlint: a new finding | actionlint |
| an unrelated edit | none |
| semgrep: one fixed | none |
| zizmor: a new finding (an unpinned `uses:`) | zizmor |
| ruff 0.15.0: a new UP042 | ruff |
| a job without `timeout-minutes` | job-timeouts |
| a new job with `timeout-minutes` | none |

**Problems the dry run found, all fixed:**
- **semgrep probe:** the first one went in `posthog/models/person/util.py`, which the rule excludes.
  It moved to `posthog/api/person.py`.
- **zizmor probe:** a template expression in `run:` also tripped actionlint. It became an unpinned
  `uses:`.
- **Reusable-workflow call:** zizmor's `self-repository` audit flags every local call. So that
  exemption is scored by `swap`, not by the gate.
- **semgrep errors:** a repeated `--disable-version-check` made semgrep print its usage, which read
  as zero findings. The wrapper now adds the flag only once, and the harness fails on a tool error.

**The rest of the harness, against `ref.py` on the same copy** ([results/dry/](results/dry/)):

| Check | Result |
|---|---|
| `cases 6`, with a real gate | All 10 as expected: the committed state passes, the 6 regressions fail, the 3 others pass |
| `parity 6` | All 5 tools match the truth |
| `parity`, with ruff's command narrowed to one rule and one directory | Caught: 13 findings against 31 |
| `swap` | hogli's verdict matched on all 4 states, the reusable call included |
| `glue`, with a 2-line script committed | 2 lines of glue. Baselines and `TOOLS.md` are not glue |
| `progress-fixes` | start 209, now 199, paid 10; `progress` printed the same |
| `last-pr` | Key: 1 new semgrep finding (`posthog/api/person.py:1566`), 2 new zizmor findings (`ci-python.yml:29`), campaign 209 → 200, paid 9 |

`last-pr` first called old findings new, because they had shifted down below the inserted lines.
A finding now counts as new only on a line the pull request added, and the count must match
each tool's increase.

## 9. Limits

- One run per phase and arm, on one repository: indicative, not significant.
- The read allowlist is in the prompt, not enforced. Agents' transcripts aren't audited.
- The box arm uses tools Toolbox's maintainers chose to slot (SARIF). A tool with only a bespoke
  format would cost the box a fitting, and isn't tested here.
- Two of PostHog's 26 semgrep rules, and actionlint without shellcheck (§2).
- Phase 6 favours the box by design (§1).

## Sources

- [docs/design/toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md): the box's promises
- [docs/research-log.md](../../docs/research-log.md), *The box* and *Box A/B preparation*: what was checked by hand
- The PostHog fork's `.github/workflows/ci-security.yaml` (semgrep 1.163.0), `ci-actionlint.yml` (actionlint 1.7.12), `ci-python.yml` (ruff), `pyproject.toml` (`ruff~=0.14.11`) and `tools/hogli-commands/hogli_commands/workflow_lint/`
