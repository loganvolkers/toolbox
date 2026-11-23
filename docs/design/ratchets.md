# Ratchets: What Toolbox Learns From Betterer and Its Alternatives

*Status: research and decision, 2026-11-20 (D67). The ratchet itself is built: `toolbox report
--baseline` (D66, [toolbox-for-toolbox.md](./toolbox-for-toolbox.md)). Federation, where ratchets
roll up across a fleet, is in [federation.md](./federation.md).*

---

## 1. Conclusion

**Copy Betterer's ideas; don't depend on Betterer; read its results files.**

| Question | Answer |
|---|---|
| What is Betterer? | A TypeScript tool that records a test's result in a committed file (`.betterer.results`) and fails when the result gets worse. Improvements are written back, so the bar only moves up. It runs per repository, with no fleet view or ownership (§2) |
| What do we learn? | Five ideas, all now in `toolbox report` or proposed (§3): a committed results file; `better / same / worse` instead of pass/fail; per-issue (here per-evidence-item) granularity so a failing rule can still improve; a `ci` mode that also fails when the file is stale; and conflict-free regeneration instead of a merge tool |
| Use Betterer directly in Toolbox for Toolbox? | **No** (§4). It is a Node runtime for a Rust repository, it duplicates what our connectors measure, its results carry no owners or levels, and the project's release activity has slowed (last stable release 5.4.0 on 2022-08-09; 6.0.0-alpha.1 on 2024-12-01) |
| Integrate? | **Yes, by reading its results file** (§5, built): the `workspace` connector parses `.betterer.results` (`betterer` globs) into one record per test, with issue counts per file. Teams that already ratchet with Betterer appear in fleet scorecards with no change on their side. This is the "adopted by a few teams, but no fleet-wide story" gap the user described |
| Alternatives? | Every mature linter has grown a baseline: ESLint bulk suppressions, PHPStan, detekt, basedpyright, RuboCop's todo file, golangci-lint's new-from-rev (§6). They all ratchet *one tool's* findings in *one repository*. Toolbox ratchets *scorecard outcomes* with owners and levels, locally and across the fleet. OpenSSF Scorecard shows the per-repo and fleet duality works at scale |

## 2. Betterer, from its source

*Read from a clone of `phenomnomnominal/betterer` (HEAD 2025-08-09); release data from the npm
registry; repository metadata from the GitHub search API, 2026-11-20.*

| Aspect | What it does | Where |
|---|---|---|
| Test | `new BettererTest({test, constraint, goal})`: any function returning a value, a constraint comparing it with the stored value, an optional goal | `website/docs/tests.md` |
| Constraints | `bigger` / `smaller` return `better`, `same` or `worse` | `packages/constraints/src/` |
| File tests | `BettererFileTest`: issues per file, stored as `"path:fileHash": [[line, col, length, message, issueHash]]`. Hashes let issues follow moved lines and moved files. **New issues anywhere make the result worse, even if others were fixed** | `packages/betterer/src/test/file-test/constraint.ts`, `differ.ts` |
| Results file | `.betterer.results`, a JavaScript module (`exports[\`name\`] = { value: \`…\` }`), committed "like a Jest snapshot" | `website/docs/results-file.md`, `src/results/print.ts` |
| Modes | `start` (write improvements, fail on worse), `ci` (fail on any difference: the file must match), `precommit` (update and stage), `watch`, `merge` (resolve conflicts in the results file), `--update` (accept worse), `--strict` | `website/docs/running-betterer.md`, `workflow.md` |
| Built-in tests | TypeScript strictness, ESLint, stylelint, regexp, tsquery, coverage, Angular | `packages/*` |
| Health | MIT; 639 stars, 49 forks, 72 open issues; about 295 `.betterer.results` files in public GitHub code search; last stable 5.4.0 (2022-08-09), 6.0.0-alpha.1 (2024-12-01); 32 commits in 2022, 17 in 2023, 1 in 2025 in the last 50 | npm registry, GitHub search API |

**What it lacks, for our purposes:**
- **Owners.** Nobody owns a Betterer test, so nobody is routed a regression or an exemption.
- **Levels.** A test is better or worse; there is no "bronze, silver, gold", and no notion of a standard being *met*.
- **A fleet view.** Each repository's file is an island: no aggregation, no comparison between repos, no way to see adoption.
- **A promotion path.** Nothing turns "this team's test" into "the company standard".
- **Non-JavaScript ecosystems.** Tests are TypeScript/JavaScript and run in Node.

## 3. Ideas we take, and where they landed

| Betterer idea | In Toolbox | State |
|---|---|---|
| Commit the results; review changes like a snapshot | `toolbox report --baseline FILE` writes `toolbox-report/v1` JSON: per scorecard and entity, the level and the failing rules with evidence counts. Sorted keys, one entity per block, so diffs are reviewable | ✅ (D66) |
| `better / same / worse`, not pass/fail | A rule that starts failing is *worse*; a failing rule whose evidence list shrinks is *better*, and one whose list grows is *worse*. So "43 undocumented items, and no more" is enforceable before the rule passes | ✅ |
| `ci` mode: the file must match | `--ci` fails on regressions **and** when the file is out of date, so improvements must be committed too. This repo's CI runs it on `portal/results.json` | ✅ |
| Machine-readable results for any pipeline | `--verdict FILE` writes `toolbox-verdict/v1`: the result, the exit code, and each regression, improvement and change as `{scorecard, entity, message}`. The GitHub Action reads it for its outputs and annotations; other CI systems read the same file ([actions/report/README.md](../../actions/report/README.md#other-ci-systems)). Tested end to end in `crates/toolbox-cli/tests/ratchet.rs` and in CI's `report-action` job | ✅ (D96) |
| `start` mode writes improvements | `--update` writes the file when nothing regressed (and creates it) | ✅ |
| Merge tool for conflicts | Not needed: the file is a pure function of the code, so a conflict is resolved by rerunning `--update` on the merged tree | ✅ by design |
| Per-issue tracking with content hashes (survives moved lines) | Evidence is compared by count, not identity, so moved code doesn't churn. The baseline also keeps a key per evidence item (its text with digits folded, hashed), and a grown list marks only the new items (D01M841J2SN4V3AVZZ5D91YAR51). Judging by identity would catch "fixed one, broke another" (Betterer treats that as worse) | Keys ✅, used to explain; judging by them is R1, §7 |
| Goals | Scorecard levels are the goals; the ratchet is the path to them | ✅ (existing) |
| `precommit` mode | A pre-commit hook running `toolbox report --skip <slow connectors> --baseline … --update` | Documented, not packaged |
| `bigger` / `smaller` constraints on a number | A rule's `measure` (CEL, a number) is recorded and compared instead of the evidence count: lines in a file, matches of a pattern. One line more is worse, one less better, without one evidence item per unit | ✅ (D01M7Z2ZZ97VZQNTDT4QQHM3HQD) |
| `--update` accepts worse | `--update --accept <scorecard>` records that scorecard's regressions, lists them, and still fails on regressions elsewhere: for an intended change to a standard (a tighter threshold, a wider pattern). It also restarts the campaigns that include the scorecard | ✅ (D01M7Z2ZZ97VZQNTDT4QQHM3HQD) |
| Goals with progress | A campaign's starting point is recorded in the baseline the first time it is active: entities short of the goal and the included rules' summed measure. `-o json` and the summary show the paydown from there; changing the campaign's `start` restarts it | ✅ (D01M7Z2ZZ97VZQNTDT4QQHM3HQD) |
| Issues as a list | `-o failures` prints every failing rule as JSON: scorecard, rule, entity, path, the entity's scalar fields, measure, evidence, fix, and the connector records it came from | ✅ |
| A results file that only holds issues | A scorecard whose new-entity floor is its top level (every entity must pass) records only the entities that don't. A clean file, new or not, adds nothing, so adding clean files never makes the baseline stale (Codex's dropped from 174 KB to 30 KB). An entity absent from such a baseline that now fails is a regression | ✅ (D01M7ZRAJT47W11X5X3FEJ6QTX8) |
| Compare with a commit | `--against <rev>` scores the portal at that commit in a temporary git worktree and compares, without a baseline file: what one commit or branch changed | ✅ |
| Why it got worse | Every regression carries its root cause: the rule's evidence, the command that reproduces it, and for a repository level the constituent scorecard that fell and the change below it ([root-causes.md](./root-causes.md)) | ✅ (D01M7ZZYZ1SXSTHDZQK8A1CZMMN) |
| Exemptions aren't fixes | An entity that becomes `skipped` (exempted, out of scope) is reported as a change, not an improvement | ✅ |

**First catch.** The first time CI's ratchet ran locally, it caught the author of this document:
- a change had raised a runner function's cyclomatic complexity to 16;
- the cause was fixed (`execute` split into named steps); the baseline was not edited
  ([research-log.md](../research-log.md)).

## 4. Should Toolbox for Toolbox use Betterer directly?

| Option | For | Against |
|---|---|---|
| **A. Adopt Betterer as the ratchet** (`.betterer.ts` with tests calling cargo, clippy and our scanners) | Mature UX, VS Code extension, merge tooling | Adds a Node toolchain to the maintainer loop for a Rust repository. The tests would re-implement what the `workspace` and `runner` connectors already measure. Results have no owners or levels, so they can't feed scorecards without a second model. A slowing upstream on the critical path (charter goal G3: no lock-in) |
| **B. Copy the ideas into `toolbox report`** (chosen) | One model: scorecards, evidence, ratchet. Works for any ecosystem a connector can read. The same file feeds the fleet view (federation) | We maintain the comparison logic (about 120 lines with tests) |
| **C. Read `.betterer.results` as data** (also done) | Teams that already use Betterer roll up with no change | Read-only: Toolbox reports their state; Betterer remains their enforcement |

**Decision (D67):** B for our own repository and for Toolbox's product; C as an integration for
fleets that already use Betterer. A is rejected.

## 5. The Betterer integration (built)

- **Config:** `workspace` connector option `betterer: ["**/.betterer.results"]`.
- **Records:** kind `betterer`, external id `betterer:<path>#<test>`, data:
  - `{test, path, value}` for every test;
  - plus `{kind: "file", issues, files: {path: count}}` for file tests;
  - or `{kind: "number", number}` for numeric ones.
- **Parsing without Node.** The results file is JavaScript, but its shape is fixed:
  - a regex finds each `exports[...] = { value: ... }` entry;
  - Betterer's own `escape()` (`src/results/print.ts`) is reversed.
- **Tests:** a unit test covers a results file taken from Betterer's test snapshots.
- **Using the records.** A portal maps them onto repositories and scores them. For example: "no Betterer
  test has more issues than last quarter", or "Betterer adopted: at least one test".

## 6. Alternatives (baselines in other tools)

*Verified against each project's own docs in its repository on 2026-11-20 unless marked.*

| Tool | Mechanism | Granularity | Fleet view | Source |
|---|---|---|---|---|
| **ESLint** bulk suppressions | `--suppress-all` / `--suppress-rule` write `eslint-suppressions.json`; new violations of those rules fail; fixed ones must be pruned (`--prune-suppressions`), or ESLint exits non-zero | Rule × file counts | No | `eslint/eslint` `docs/src/use/suppressions.md` |
| **PHPStan** baseline | `--generate-baseline` writes `phpstan-baseline.neon`: message × path × count, ignored in later runs. "Not the best tool when you have 15,000 errors" | Message × file | No | `phpstan/phpstan` `website/src/user-guide/baseline.md` |
| **basedpyright** baseline | `--writebaseline`; matched by file, rule and column (not line, to survive edits); auto-updated as errors are removed | Rule × file × column | No | `DetachHead/basedpyright` `docs/benefits-over-pyright/baseline.md` |
| **detekt** (Kotlin) baseline | `baseline.xml` of known issues | Issue ids | No | Docs path in `detekt/detekt` not found in the sparse checkout; **unverified** |
| **RuboCop** | `--auto-gen-config` writes `.rubocop_todo.yml` disabling or relaxing cops where they fail today | Cop × file | No | Secondary (well known; the doc page wasn't matched in the sparse checkout) |
| **golangci-lint** | `new-from-rev`, `new-from-merge-base`, `new-from-patch`: report only issues in changed code | Diff hunks | No | `golangci/golangci-lint` `pkg/config/issues.go` |
| **SonarQube** "new code" | Quality gates on code changed in the new-code period | Diff period | Yes (server) | Secondary: docs site not reachable from this session |
| **OpenSSF Scorecard** | 19 checks (Branch-Protection, Code-Review, Pinned-Dependencies …) scored per repository; runs as a CLI or Action on one repo and "scans more than 1.3 million repositories every week" for public data | Check × repo | Yes (public dataset, REST API) | `ossf/scorecard` `README.md`, `docs/checks.md` |

**What the landscape says:**
1. Baselines are the accepted way to raise a bar on an existing codebase. Every major linter
   converged on one, so the ratchet is table stakes.
2. They are all per tool and per repository. Nobody aggregates baselines into owned standards with
   levels across a fleet. **That is Toolbox's slot.**
3. OpenSSF Scorecard proves the duality: the same check definitions run locally on one repo and as a
   fleet scan. Toolbox should keep that property, with one manifest, `report` locally and `serve`
   centrally (see [federation.md](./federation.md)).
4. Granularity varies: counts per file (PHPStan, ESLint), position-tolerant identity (basedpyright,
   Betterer hashes), or diff scoping (golangci-lint, Sonar). Toolbox starts with counts per rule and
   entity (§7).

## 7. Open items

| # | Item | Proposal |
|---|---|---|
| R1 | Evidence identity | The baseline records a key per evidence item (D01M841J2SN4V3AVZZ5D91YAR51), and the verdict uses them to mark new items. Open: an optional `evidenceKey` CEL per rule (e.g. the function name without the line) and comparing keys rather than counts, so "fixed one, broke another" is worse, as in Betterer |
| R2 | Read other baselines | ESLint suppressions, PHPStan baselines and basedpyright baselines are JSON/NEON with counts. One `baselines` reader in the workspace connector would roll them all up the way §5 does for Betterer |
| R3 | Diff-scoped rules | "No *new* function over 80 lines" via the ratchet already works per entity; per-diff scoping (golangci-lint style) is not needed while entities are files and crates |
| R4 | Pre-commit packaging | A documented hook, then a `toolbox report --precommit` mode that updates and stages the baseline |

## Sources

- Betterer: repository `phenomnomnominal/betterer` (README, `website/docs/*.md`,
  `packages/constraints`, `packages/betterer/src/results/print.ts`, `src/test/file-test/*`); npm
  registry metadata for `@betterer/cli`; GitHub search API (repository metadata, code search for
  `filename:.betterer.results`), all read 2026-11-20.
- ESLint `docs/src/use/suppressions.md` and `command-line-interface.md`; PHPStan
  `website/src/user-guide/baseline.md`; basedpyright `docs/benefits-over-pyright/baseline.md`;
  golangci-lint `pkg/config/issues.go`; OpenSSF Scorecard `README.md` and `docs/checks.md`. Each
  was read from its GitHub repository on 2026-11-20.
- Toolbox: [toolbox-for-toolbox.md](./toolbox-for-toolbox.md), `crates/toolbox-cli/src/report.rs`,
  `crates/toolbox-connector-workspace/src/betterer.rs`.
