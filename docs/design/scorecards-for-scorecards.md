# Scorecards for Scorecards: What Makes a Standard Good, and Where the Recursion Stops

*Status: deterministic meta scorecards built; LLM assessment and agent trials proposed, 2026-11-20
(D71). Toolbox for Toolbox ([portal/](../../portal/README.md)) now catalogues every scorecard and rule
of three portals: itself, the [repo-portal starter](../../examples/repo-portal-starter/README.md) and the
[reference portal](../../examples/reference-portal/README.md). It scores them with `meta-rule` and
`meta-scorecard`, which score themselves too.*

---

## 1. Conclusion

**Building and using 13 scorecards (56 rules) taught us nine properties of a good standard.** Five of
them can be checked deterministically today. They are now two meta scorecards, ratcheted in CI like
everything else:
- `meta-rule` asks of each rule: fix text, evidence, a test that names it, both outcomes on fixtures,
  no nested loops, a readable check;
- `meta-scorecard` asks of each scorecard: tested, no empty levels, fixtures that reach at least two
  levels, every rule tested.

The other four properties need judgement: whether the check measures the intent, whether it can be
gamed, whether the fix matches the check, and whether thresholds are justified. They need an **LLM
assessment** (advisory, cached by content hash) and **agent trials** (the experiment that found most
of our lessons).

**The recursion stops at one meta level, by self-application, not by more configuration.**

| Level | What it is | Scored by | Config |
|---|---|---|---|
| L0 | Entities: crates, docs, workflows, MCP tools … | L1 scorecards | — |
| L1 | Scorecards and rules (ours, the starter's, the reference portal's) | L2 | the portals' manifests |
| L2 | `meta-rule`, `meta-scorecard` | **L2 itself**: they are rules and scorecards, so they appear in their own input | the same `portal/toolbox.yaml` |

There is no T4T4T. Two things keep L2 a fixed point:
- **Self-application.** The meta rules are tested by the same expectations they demand, and decide
  both ways on fixtures they ship with.
- **Determinism.** Meta rules read only the manifest, the CEL, the tests and the fixtures. Live
  outcomes on real data are attached for information but never scored, so describing scorecards
  never depends on its own previous output, and the ratchet converges after one `--update`.

**What our own standards scored on the first run:**

| | Rules (68, including the 12 meta rules) | Scorecards (15) |
|---|---|---|
| Gold | 32 | 3 |
| Silver / bronze | 1 / 9 | 9 / 2 |
| None | 26 | 1 |
| Most common gaps | Untested (26 rules); never both outcomes on fixtures (18); no evidence (17) | A rule without a test (11 scorecards) |

The reference portal's 6 rules have no evidence at all, and 3 of them are untested. `user-cli` has no
test of any kind. This is the CODEOWNERS-quality story again, one level up: the standard that
everyone else is held to wasn't itself held to anything.

## 2. What we learned, and the property each lesson implies

| # | What happened | Property of a good standard |
|---|---|---|
| L1 | The fresh agent took `user-mcp` from 4 silver + 7 bronze to 11 gold using only the portal; fix texts that named the file and field were praised ([toolbox-for-toolbox.md](./toolbox-for-toolbox.md) §6.1) | **Actionable**: a fix that says what and where |
| L2 | The same agent needed one `explain` call per rule per entity to find *what* failed (F11); evidence (D66) fixed it | **Explained**: failing rules name the failing items |
| L3 | `contract-tested` passed for a tool merely *mentioned* in a test (F9); `documented` accepted any doc with a backticked common word (F14) | **Valid**: the check measures the intent, not a proxy an agent can satisfy |
| L4 | First run: a numbered `## 5. Sources` and a quoted "SDLC" were false positives; a time-estimate regex was dropped (F8) | **Tested**: expectations pin both outcomes over fixtures |
| L5 | `join` and `flatten` passed `toolbox validate` and failed only when scores were computed, twice | **Well-formed**: functions exist, fields exist, types fit (now: `cel.function` in validate) |
| L6 | Without zizmor, rules first came out `fail`, which looked like regressions | **Honest about missing data**: unknown ≠ fail (D70) |
| L7 | CI fail rates change every run; putting them in the ratchet would make it flap | **Deterministic where it gates**; observational signals are reported, not ratcheted |
| L8 | The 100k-entity budget fails on 2-vCPU runners (24.4 s vs 10 s); rules are evaluated once per entity | **Cheap**: no nested loops over unbounded lists; small checks |
| L9 | CI reliability thresholds (20/10/5%, p90 15 min) have no published source; we said so | **Justified**: thresholds carry a rationale or a source |
| L10 | Our ratchet caught our own regressions four times, but a *new* untested rule would have slipped through (new entities were neutral) | **Gated from day one**: new entities must meet a floor (`ratchet.newEntitiesAtLeast`, built) |
| L11 | `zizmor` `unpinned-uses` and `pinned-to-sha` flagged the same thing; we excluded the overlap by hand | **Non-redundant**: two rules shouldn't fail on exactly the same entities for the same reason |
| L12 | `tests-pass` and `clippy-clean` fail on every crate in the static ratchet because the runner is skipped | **Achievable in context**: a rule that cannot pass where it gates measures absent data, not the standard |

## 3. The meta standard

### 3.1 Built (deterministic, in the ratchet)

| Scorecard | Level | Rule | Lesson | Check (over `toolbox describe scorecards` facts) |
|---|---|---|---|---|
| `meta-rule` | bronze | `explains-fix` | L1 | fix ≥ 5 words |
| | bronze | `tested` | L4 | a test expectation names `score('<card>', …).rules/evidence['<rule>']` |
| | bronze | `decided-on-fixtures` | L4 | it passes or fails on at least one fixture entity |
| | silver | `both-outcomes-on-fixtures` | L4 | it passes on one fixture entity and fails on another |
| | silver | `shows-evidence` | L2 | `evidence:` present, unless the check is trivial (≤ 4 CEL nodes) |
| | silver | `no-nested-loops` | L8 | comprehension nesting ≤ 1 in check and evidence |
| | gold | `readable` | L8 | ≤ 30 CEL nodes |
| | gold | `always-explains` | L2 | `evidence:` present |
| `meta-scorecard` | bronze | `tested` | L4 | an expectation names the scorecard |
| | bronze | `every-level-has-rules` | — | no empty level |
| | silver | `fixtures-show-the-ladder` | L4 | the fixtures reach at least two levels |
| | gold | `every-rule-tested` | L4 | no untested rule (rolled up from Rule entities) |

Both use `ratchet: {newEntitiesAtLeast: bronze}`. Existing rules were baselined as they are, but
every *new* rule or scorecard must be tested, explained and decided on fixtures from its first
commit (L10).

**Made possible by (also built):**
- **`toolbox describe scorecards PORTAL… [--results PORTAL=FILE]`** emits `scorecard` and `rule`
  records with:
  - the manifest facts;
  - CEL analysis: nodes, depth, comprehension nesting, fields read, functions called, undeclared
    functions;
  - test mentions;
  - outcomes over the portal's own fixtures;
  - optionally, live outcomes from a results file.
- **`toolbox_expr::analysis`**: the CEL walker. `toolbox validate` now rejects functions the runtime
  doesn't declare (`cel.function`; L5).
- **`ratchet.newEntitiesAtLeast`**: a scorecard property. A scorecard new to the baseline baselines
  everything first, as Betterer's first run does.

### 3.2 Proposed: deterministic, next

| Rule | Lesson | What it needs |
|---|---|---|
| `has-rationale` | L9 | An optional `why:` on rules (a source or reasoning); validate that thresholds in checks have one |
| `fields-exist` / `types-fit` | L5 | A CEL type checker over kind schemas (Q10); today the analysis knows which fields a rule reads, not their types |
| `not-redundant` | L11 | Outcome vectors over fixtures and live data: flag rules of the same kind whose failing sets are identical. Advisory first: tiered thresholds (fail rate 20/10/5%) legitimately coincide |
| `ratchetable` | L7, L12 | Mark connectors `volatile: true` (run history) and derive which rules read volatile fields; skip them from the ratchet automatically instead of `--skip` by hand, and flag a rule that fails on every entity only because its source is skipped |
| `within-budget` | L8 | Per-rule evaluation time measured by `derive` (sampled), with a budget per rule. Nodes and loops are the deterministic proxy until then |
| `evidence-names-items` | L2 | The evidence expression returns a list derived from entity data (a comprehension or a list field), not a constant sentence |

### 3.3 Proposed: LLM assessment (judgement, advisory)

Some properties are semantic: L3 (valid, not gameable), the fix matching the check (F14), and L9
(rationale quality). Deterministic checks can't see them. An assessor reads each rule with its kind
schema, fixtures and evidence and answers a fixed rubric.

| Question | Output |
|---|---|
| Does the check measure what the rule's name and fix say? | pass / fail + reason |
| Could an entity pass without meeting the intent (e.g. a mention instead of a call)? | the cheapest gaming strategy, if any |
| Does the fix lead to passing the check? | pass / fail + reason |
| Is the threshold justified by the rationale? | pass / fail / no rationale |

**Mechanics** (a `runner` job, like zizmor):
- results are cached by `sha256(rule YAML, kind schema, model id, rubric version)`;
- the cache is committed (`portal/assessments.json`, Betterer-style), so CI is deterministic and
  re-assessment happens only when a rule changes;
- verdicts are reported per rule and reviewed in the PR that changes the rule;
- an assessment never gates on its own: a `fail` turns a rule's meta level `unknown` until a person
  accepts or rejects it, like a waiver.

This needs an API key, so it's not built here.

### 3.4 Proposed: agent trials (the strongest signal)

The fresh-agent experiment found L1–L3 and L11 in one run. That makes it the best scorecard test we
have, and it should be repeatable:
- **The task:** an agent is given only the portal (MCP) and "raise `<scorecard>` without gaming it".
- **What the trial records:** levels before and after, tool calls, which evidence and fix texts it
  used, friction notes, and a reviewer's verdict on whether any change gamed a rule.
- **Output:** a `trial` record per scorecard, scored by a future `meta-scorecard` rule
  (`agent-can-raise-it`).
- **Cadence:** expensive and non-deterministic, so it runs on demand or on a schedule, and is never
  part of CI.

## 4. Predictions

| As Toolbox grows… | The meta standard will need |
|---|---|
| **Fleets accumulate hundreds of rules across team packages** ([federation.md](./federation.md)) | Redundancy across packages, an owner per rule, and retirement: a rule passing on every entity for a long time becomes a guardrail (ratchet-only) or is removed |
| **Agents write rules** (they already write fixes) | `ratchet.newEntitiesAtLeast` on `meta-rule` as the merge gate: an agent-authored rule must arrive tested, explained and decided on fixtures, and the LLM assessment flags gameable ones |
| **Graduation needs a quality bar** | A standard moves to `mandatory` only when its scorecard is at meta gold, and promotion is a meta check, not a meeting |
| **Rules read more volatile data** (run history, incidents, costs) | First-class volatility, so ratcheted and observed standards are told apart automatically (§3.2 `ratchetable`) |
| **Evaluation cost becomes visible** at 100k+ entities and many packages | Per-rule cost budgets and the CEL analysis in `plan`: "this change adds a nested loop over 2M relations" |
| **Teams copy the starter** | The starter's own meta level matters most. It is gold on 4 of 6 rules today; `codeowners-catch-all` is untested and `docs/links-resolve` never passes on its fixtures |

## 5. Verification

- **Unit tests:**
  - CEL analysis: size, fields, loops, `join` and `flatten` undeclared, common functions declared;
  - `toolbox validate` rejects an undeclared function (`cel.function`) and still accepts all three
    real portals;
  - the ratchet's new-entity floor, including the bootstrap case;
  - meta describe, run against three portals: 83 records (68 rules, 15 scorecards).
- **`toolbox test portal`:** 40/40. Nine expectations are new; they cover every meta rule, including
  self-application: `meta-rule` scores `rule:toolbox/meta-rule/tested`.
- **The ratchet:**
  - `--update` twice is stable, and `--ci` matches (a fixed point);
  - with the floor in force, adding the meta scorecards baselined every existing rule and
    scorecard;
  - before the bootstrap fix, the 27 below bronze were reported as regressions (alongside 3 real
    ones in our own code), which is why the bootstrap rule exists.
- **The ratchet caught this change, twice:** `compare` grew past 80 lines and complexity 15, and the
  CEL walker reached complexity 17. Both were split into named steps; the baseline was not loosened.
- **Not built:** LLM assessment, agent trials, rationale, type checks, redundancy, volatility,
  per-rule timing (§3.2–3.4).

## Sources

- Lessons: [toolbox-for-toolbox.md](./toolbox-for-toolbox.md) §5–7 (F1–F14 and the fresh-agent run),
  [ratchets.md](./ratchets.md), [github-actions-standards.md](./github-actions-standards.md),
  [research-log.md](../research-log.md), [decisions.md](../decisions.md) D65–D70.
- Prior art for self-application: a compiler compiling itself; the
  [meta-catalog](./meta-catalog.md) design (a portal catalogues its own configuration, "including the
  scorecards that score scorecards").
- Implementation: `crates/toolbox-expr/src/analysis.rs`, `crates/toolbox-cli/src/meta.rs`,
  `crates/toolbox-cli/src/report.rs` (`newEntitiesAtLeast`), `portal/toolbox.yaml` (`Scorecard`,
  `Rule`, `meta-rule`, `meta-scorecard`).
