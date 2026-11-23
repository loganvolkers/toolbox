# Proving Repo-Level Value

*Status: plan, 2026-11-21 (D01M7Y46VDDBK2SW6H96ZSG89MP; refined by D01M80Y0XEK069GD08VCCNF0WB8). This is the bottom-up end: value
proven at a single repository. The top-down end, the central portal, proceeds in its own sessions, and
federation is unpaused. This is the
operator SDLC applied to one repository (a [repo portal](./repo-portals.md)), built under the
maintainer SDLC, with packages from the package author SDLC ([sdlc.md](../sdlc.md)).*

---

## 1. Conclusion

**We prove that one repository gets enough from Toolbox to keep it, before we build anything that
governs many.** A repo portal is a `toolbox.yaml`, a committed baseline and one CI step
([repo-portals.md](./repo-portals.md)). If maintainers and their coding agents get value from it,
federation is the same thing at fleet scale ([federation.md](./federation.md)). If they don't, a
central portal won't fix that.

The bet has five parts. Each has an experiment and a result that would stop us (§3).

R2, R3, R4, R6 and R7 are tested by DX Benchmarks ([dx-benchmarks.md](./dx-benchmarks.md)).
Each is catalogued in `portal/benchmarks/` with its status and outcome. R1 and R5 are measured on
the installed pilots.

| # | Hypothesis |
|---|---|
| R1 | Adopting Toolbox in a repository takes one pull request and no infrastructure |
| R2 | Its findings are true and worth fixing |
| R3 | Coding agents raise a repository's standards further with Toolbox's evidence and ratchet than with the same standards written down |
| R4 | The ratchet catches real regressions on real pull requests without becoming a chore |
| R5 | Standards travel between repositories as packages, with parameters and no edits |
| R6 | The box A/B ([experiments/box-ab](../../experiments/box-ab/README.md)): on the PostHog fork, several existing tools gathered under one gate, with a campaign, a new tool, an upgrade, a swap and a read. Arm A writes CI glue; arm B uses the box | Tokens per phase; lines of glue left to maintain; correctness (planted pull requests, take-it-out parity, the swap, the read key) | Over the six phases, the box uses fewer tokens and leaves less glue, with equal correctness, and both arms pass take-it-out | Either measure is worse: the box's value is the format it standardises, not the work it saves, and this plan says so |
| R7 | The spec A/B ([experiments/spec-ab](../../experiments/spec-ab/README.md)): results documents produced and consumed three ways, by hand from the spec alone, with Toolbox, and building on Backstage, scored by the spec's conformance suite | Conformance per requirement; tokens; glue; correctness against the box A/B's truth | S1: the spec stands alone (arm A conforms within budget); S2: Toolbox costs fewer tokens than the spec alone; S3: Backstage costs more at the repository | S1 fails: shrink the spec first. S2 fails: Toolbox's value is the reference implementation. S3 fails: repository work builds on Backstage |

## 2. What is paused and what continues

| Area | State |
|---|---|
| Central portal: Cloud Run deployment, `plan --remote`, preview, `verify` (M6) | **Top-down sessions** (D01M80Y0XEK069GD08VCCNF0WB8) |
| Self-service actions (M5, already paused by D64) | **Paused** |
| Federation, including static federation ([federation.md](./federation.md), [repo-portals.md](./repo-portals.md) §5) | **Unpaused** (D01M80Y0XEK069GD08VCCNF0WB8); how the two ends meet is decided once both are proven |
| Apps built on a portal's API (user SDLC beyond reading a report) | **Top-down sessions** (D01M80Y0XEK069GD08VCCNF0WB8) |
| `serve`, REST, MCP, UI, OpenFGA, Postgres (M2–M4) | **Kept green, not extended.** CI still runs their tests and acceptance scripts; bugs are fixed. `serve` and `mcp` on a laptop remain part of the repo-level agent loop |
| `toolbox report`, the ratchet and verdict, `actions/report`, the starter, packages, assessments, campaigns | **Focus** |

## 3. Hypotheses, experiments and stop conditions

| # | Experiment | Measure | Continue if | Stop or rethink if |
|---|---|---|---|---|
| R1 | Install on each installed pilot (§4) from the starter, from scratch | Steps and edits to the starter; CI time added on a cold and a warm run; questions the maintainer had to ask | One pull request; no edits beyond parameters; warm overhead small enough that nobody asks to turn it off | A pilot needs local changes to Toolbox, or its maintainer removes the step |
| R2 | Out-of-box runs on repositories we didn't write (§4, read-only), then triage of every failing rule | Each failure as true and worth fixing, true but not worth it, or false | At least 80% true, and at least half of those worth fixing | More than 20% false: the standards, not adoption, are the problem |
| R3 | A/B on the installed pilots: the same coding agent and budget, asked to raise the repository's standards. Arm A gets the standards as a written checklist; arm B gets `toolbox report` with evidence and the ratchet | Rules newly passing (scored afterwards by Toolbox and spot-checked by a person), regressions introduced, tokens | Arm B fixes at least 15 percentage points more at equal tokens (the bar E2 sets in [adversarial-critique.md](../adversarial-critique.md) §10) | No difference: the value is the standards text, so ship packages of written standards instead of a tool |
| R4 | The installed pilots keep the action on for their normal pull requests | Real regressions caught; intended changes flagged as worse; pull requests that had to commit a baseline update | At least one real regression caught, and maintainers keep the step | Maintainers disable it or routinely `--update` past it |
| R5 | Every pilot uses the same packages (`packages/github-actions` and the ones §5 adds) | Local overrides and forks per pilot | Parameters only (`with:`) | Each pilot forks a package: federation's package model needs rethinking first |

## 4. Pilots

*Revised 2026-11-21 (D01M7YE8W0Y9EZ5Z1ZF3HFPD767): the account's forks are the testbed.*

| Kind | Repositories | Proves | What it changes |
|---|---|---|---|
| **Installed** | `loganvolkers/rt-buddy` (a pnpm and Expo monorepo; the first customer, installed with `actions/report` at `v0.1.x`) | R1, R4 live, R5 | A pull request adding `.toolbox/` and one workflow |
| **Fork testbeds** | The account's forks of OpenFGA (Go), Codex (Rust and TypeScript, pnpm), gh-aw (Go, mostly generated workflows) and PostHog (Python and TypeScript, pnpm) | R2 by out-of-box runs, R3 by the A/B, R4 by replaying upstream history ([experiments/out-of-box](../../experiments/out-of-box/README.md)), R5 | Nothing upstream: runs happen locally, and results go to the research log |

Replaying history measures R4 on real changes by other people: each upstream commit goes through the
ratchet against its parent, as CI would have run it. What forks can't show is a maintainer other
than us choosing to keep the step on. That stays open, and it is the first thing federation tests.

## 5. Gaps to close before the pilots

| Gap | Why it matters | Work |
|---|---|---|
| Missing data scored as failure | `maintainer-crate` showed crates at `none` partly because the ratchet skips the runner, so `tests-pass` and `clippy-clean` read as failures. On pilots this would count as false positives under R2 | Done (D01M7Y5VWKW5TDS0VQWZFPRQ27N). A rule with `dataFrom` is `unknown` when that connector is skipped, and an all-unknown level is not granted |
| Cold start | The action builds Toolbox from source on a pilot's first run | Prebuilt binaries for pilots ([package-author-sdlc.md](./package-author-sdlc.md) §3); needs a pre-release |
| A thin starter | It checks a README, CODEOWNERS, a licence and doc links | Package the standards in Toolbox for Toolbox that aren't specific to Toolbox: agent instruction files, supply-chain basics, npm manifests, docs. Each is tested in its own `tests/` portal, as `packages/github-actions` is |
| Language coverage | The workspace connector knows Rust crates, npm manifests and Markdown | Measured by the out-of-box runs; add what R2's triage shows is missing |
| Pull request experience | Annotations and a job summary, but no published report | Publish the one-page report with each run; a summary comment is optional |

## 6. Exit

*Revised 2026-11-21 (D01M7YE8W0Y9EZ5Z1ZF3HFPD767).*

**Repo-level value is proven when R1, R2, R4 and R5 continue on rt-buddy and on at least three of the
four fork testbeds, and R3 is either met or replaced by evidence that the ratchet keeps the gains
agents make (R4).** Federation (D68) then resumes, starting from the packages that R5 showed
travel. Its first step is a pilot with maintainers other than us, the one thing forks can't prove.

## 8. Results so far

| # | State | Evidence |
|---|---|---|
| R1 | **Partly met** | Adopting is one portal file and one CI step. On the forks, runs take 0.4–36.4 s. The first out-of-box run failed on all four forks until the runner and the `github-actions` package stopped assuming a Rust repository with an `actions/` directory. rt-buddy is being installed by another session |
| R2 | **Continue** | After this round's fixes, and once PR #24 scores missing run data as `unknown`: 939 of 940 failing (entity, rule) pairs on the four forks are true, and 70% of the true ones are worth fixing. An agent did the triage, sampling the evidence |
| R3 | **Failed** ([D01M7ZD07SE4ADHZP2QKA3XG67E](../decisions/D01M7ZD07SE4ADHZP2QKA3XG67E.md)) | Round 1 (raise standards): the checklist arm fixed as much or more (86% and 100% against 83% and 82%); Toolbox made no regressions and no suppressions ([experiments/out-of-box/ab](../../experiments/out-of-box/ab/README.md)). Round 2 (a standard's lifecycle on the forks' own `AGENTS.md` standards): every arm exact; bespoke scripts 445k tokens, Toolbox 582k. Rerun after fixing the measured friction and adding packages: Toolbox 510k, within 3% of the scripts after the first standard and ahead on reuse in calls and time, but learning the format cost 55k more, so the pre-registered bar (fewer tokens) is missed. Toolbox's repositories maintain about a third as much code ([experiments/standards-ab](../../experiments/standards-ab/README.md)) |
| R4 | **Early: continue** | 141 replayed upstream commits: 3 flagged, of which 1 real (a new action not pinned to a SHA, gh-aw `b5694a414`), 1 probably real (actionlint, an output of a job not in `needs`) and 1 noise (low-severity zizmor notes under gold `zizmor-clean`). 5 commits (3.5%) improved a score and would have had to commit the baseline under `--ci` |
| R5 | **Continue** | The same packages, unedited, ran on all four forks after the fixes, with parameters only (`githubOrg`, `githubRepo`) |
| R6 | **Failed the bar** ([experiments/box-ab](../../experiments/box-ab/README.md) §0) | Over six phases on the PostHog fork, both arms were fully correct (planted pull requests, take-it-out parity, the WF001 swap against hogli, the paydown, the read key). The box left less glue (306 lines added against 341) but used 11% more tokens (679k against 610k), most of it in phase 1, learning Toolbox. By the rule set before the run, the box's value here is the format it standardises, not the work it saves. Its feedback drove fixes to `--accept` and campaigns, unknown outcomes, repeated tool pins, vendoring and the kit docs |

## 9. Order of work

1. Missing data as `unknown` (§5). Done (D01M7Y5VWKW5TDS0VQWZFPRQ27N).
2. Out-of-box runs and their triage (R2).
3. Package the generic standards (§5, R5).
4. Pre-release binaries for pilots, if agreed.
5. Install the pilots (R1, R4).
6. The A/B (R3). Done: failed (D01M7ZD07SE4ADHZP2QKA3XG67E).
7. Make the box a box: the SARIF slot, take-it-out, and the box's friction, used on Toolbox for
   Toolbox first ([toolbox-and-tools.md](./toolbox-and-tools.md) §7). Done for items 1, 2 and 4.
8. The box A/B ([experiments/box-ab](../../experiments/box-ab/README.md)): several tools in a box
   against hand-written glue. Designed; its hypothesis is to be decided before it runs.
9. Review the pilot period (R4, R5) and decide on federation.

## Sources

- [repo-portals.md](./repo-portals.md), [federation.md](./federation.md), [packages.md](./packages.md)
- [adversarial-critique.md](../adversarial-critique.md) §10: kill criteria E1 and E2
- [toolbox-for-toolbox.md](./toolbox-for-toolbox.md): the first repo portal
