# Scorecards for Scorecards: What Makes a Standard Good, and Where the Recursion Stops

*Status: built, 2026-11-20 (D71, D79, D80). Seikika for Seikika ([portal/](../../portal/README.md))
catalogues every part of three portals: itself, the
[repo-portal starter](../../examples/repo-portal-starter/README.md) and the
[reference portal](../../examples/reference-portal/README.md). Deterministic meta scorecards cover
rules, scorecards, kinds, fields and connectors, and score themselves too. Judgement comes from
coding agents' assessments, pushed as data (§6). Agent trials (§3.4) remain proposed.
**Re-evaluated 2026-11-23 ([meta-standards-review.md](./meta-standards-review.md),
D01M831P5QWJQ4A2FT85N5MXJQF): meta gold means well-formed, not good. Reviews now put open findings
first and must be independent.***

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
gamed, whether the fix matches the check, and whether thresholds are justified.

**Judgement is pushed, not pulled (D79).** Every developer already runs a coding agent, so Seikika
holds no model keys and calls no model. It gives the agent a **rubric** (in the manifest), a **brief**
(`seikika assess brief`, or the `assessment_brief` MCP tool) and a skill
([`assess-entity`](../../plugins/seikika-operator/skills/assess-entity/SKILL.md)). The agent writes an
**assessment**: a file in `assessments/`, or a record pushed to Postgres with `seikika assess
submit`. Scorecards read assessments as data, keyed to a **content hash** of what was assessed, so an
assessment goes stale the moment its subject changes. Assessments are less real-time than checks, but
they are stored, visualised, queried and referenced like any other entity. `review-*` scorecards
report them, and they are **advisory** (`ratchet.advisory`): a stale review never blocks a change.

**Rules and scorecards are not the only parts of a portal that can be good or bad (§4).** Kinds,
fields and connectors are standards too: they are what every rule reads. They now have meta
scorecards (`meta-kind`, `meta-field`, `meta-connector`; §5). Relations, map rules, tests,
authorization and actions are covered through those, or by `seikika validate`, rather than by
scorecards of their own.

**The recursion stops at one meta level, by self-application, not by more configuration.**

| Level | What it is | Scored by | Config |
|---|---|---|---|
| L0 | Entities: crates, docs, workflows, MCP tools … | L1 scorecards | — |
| L1 | Scorecards and rules (ours, the starter's, the reference portal's) | L2 | the portals' manifests |
| L1′ | Kinds, fields, connectors (the model those rules read) | L2 | the portals' manifests |
| L2 | `meta-rule`, `meta-scorecard`, `meta-kind`, `meta-field`, `meta-connector`, `review-*` | **L2 itself**: they are rules, scorecards, kinds and fields, so they appear in their own input | the same `portal/seikika.yaml` |
| — | Assessments (judgement) | `review-*` (advisory) | files in `portal/assessments/` |

Seikika for Seikika stops at two levels. Two things keep L2 a fixed point:
- **Self-application.** The meta rules are tested by the same expectations they demand, and decide
  both ways on fixtures they ship with.
- **Determinism.** Meta rules read only the manifest, the CEL, the tests and the fixtures. Live
  outcomes on real data are attached for information but never scored, so describing scorecards
  never depends on its own previous output, and the ratchet converges after one `--update`.
- **Assessments recurse once.** Assessments are scored deterministically (`meta-assessment`) and
  reviewed by an agent against `assessment-quality` (`review-assessment`). Reviews made with
  `assessment-quality` are excluded from that card, so there is no review of reviews (§6.1).

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
| L1 | The fresh agent took `user-mcp` from 4 silver + 7 bronze to 11 gold using only the portal; fix texts that named the file and field were praised ([seikika-for-seikika.md](./seikika-for-seikika.md) §6.1) | **Actionable**: a fix that says what and where |
| L2 | The same agent needed one `explain` call per rule per entity to find *what* failed (F11); evidence (D66) fixed it | **Explained**: failing rules name the failing items |
| L3 | `contract-tested` passed for a tool merely *mentioned* in a test (F9); `documented` accepted any doc with a backticked common word (F14) | **Valid**: the check measures the intent, not a proxy an agent can satisfy |
| L4 | First run: a numbered `## 5. Sources` and a quoted "SDLC" were false positives; a time-estimate regex was dropped (F8) | **Tested**: expectations pin both outcomes over fixtures |
| L5 | `join` and `flatten` passed `seikika validate` and failed only when scores were computed, twice | **Well-formed**: functions exist, fields exist, types fit (now: `cel.function` in validate) |
| L6 | Without zizmor, rules first came out `fail`, which looked like regressions | **Honest about missing data**: unknown ≠ fail (D70) |
| L7 | CI fail rates change every run; putting them in the ratchet would make it flap | **Deterministic where it gates**; observational signals are reported, not ratcheted |
| L8 | The 100k-entity budget fails on 2-vCPU runners (24.4 s vs 10 s); rules are evaluated once per entity | **Cheap**: no nested loops over unbounded lists; small checks |
| L9 | CI reliability thresholds (20/10/5%, p90 15 min) have no published source; we said so | **Justified**: thresholds carry a rationale or a source |
| L10 | Our ratchet caught our own regressions four times, but a *new* untested rule would have slipped through (new entities were neutral) | **Gated from day one**: new entities must meet a floor (`ratchet.newEntitiesAtLeast`, built) |
| L11 | `zizmor` `unpinned-uses` and `pinned-to-sha` flagged the same thing; we excluded the overlap by hand | **Non-redundant**: two rules shouldn't fail on exactly the same entities for the same reason |
| L12 | `tests-pass` and `clippy-clean` fail on every crate in the static ratchet because the runner is skipped | **Achievable in context**: a rule that cannot pass where it gates measures absent data, not the standard |

## 3. The meta standard

### 3.1 Built (deterministic, in the ratchet)

| Scorecard | Level | Rule | Lesson | Check (over `seikika describe scorecards` facts) |
|---|---|---|---|---|
| `meta-rule` | bronze | `explains-fix` | L1 | fix ≥ 5 words |
| | bronze | `tested` | L4 | a test expectation pins `score('<card>', …).rules['<rule>']` to a literal outcome, or compares its evidence |
| | bronze | `decided-on-fixtures` | L4 | it passes or fails on at least one fixture entity |
| | silver | `both-outcomes-on-fixtures` | L4 | it passes on one fixture entity and fails on another |
| | silver | `always-explains` | L2 | `evidence:` present (`shows-evidence`, which exempted trivial checks, was dropped as implied by it) |
| | silver | `no-nested-loops` | L8 | comprehension nesting ≤ 1 in check and evidence |
| | silver | `readable` | L8 | ≤ 30 CEL nodes (every rule meets it, so it guards rather than ranks) |
| | gold | `pins-both-outcomes` | L4 | the tests pin the rule both passing and failing |
| | gold | `evidence-tracks-check` | L1 | the evidence names what failed ([thresholds.md](./thresholds.md)) |
| | gold | `fix-names-a-place` | L1, L2 | the fix names where to act |
| `meta-scorecard` | bronze | `tested` | L4 | an expectation names the scorecard |
| | bronze | `every-level-has-rules` | — | no empty level |
| | silver | `described` | L2 | a description saying what the standard is for |
| | silver | `fixtures-show-the-ladder` | L4 | the fixtures reach two outcomes, one a real level |
| | silver | `every-rule-tested` | L4 | no untested rule (rolled up from Rule entities) |
| | gold | `fixtures-reach-every-level` | L4, L12 | a fixture entity reaches each level, so every rung is exercised in tests |

Both use `ratchet: {newEntitiesAtLeast: bronze}`. Existing rules were baselined as they are, but
every *new* rule or scorecard must be tested, explained and decided on fixtures from its first
commit (L10).

**Made possible by (also built):**
- **`seikika describe scorecards PORTAL… [--results PORTAL=FILE]`** emits `scorecard` and `rule`
  records with:
  - the manifest facts;
  - CEL analysis: nodes, depth, comprehension nesting, fields read, functions called, undeclared
    functions;
  - test mentions;
  - outcomes over the portal's own fixtures;
  - optionally, live outcomes from a results file.
- **`seikika_expr::analysis`**: the CEL walker. `seikika validate` now rejects functions the runtime
  doesn't declare (`cel.function`; L5).
- **`ratchet.newEntitiesAtLeast`**: a scorecard property. A scorecard new to the baseline baselines
  everything first, as Betterer's first run does.

### 3.2 Proposed: deterministic, next

| Rule | Lesson | What it needs |
|---|---|---|
| `has-rationale` | L9 | **Built** as `explains-why` on meta-rule (bronze) and meta-scorecard (silver): a `rationale:` and `links:` on every rule and scorecard (D01M868BY0KJNEYE9WBFZYEYFYH). Still open: validate that thresholds in checks have a source |
| `fields-exist` / `types-fit` | L5 | A CEL type checker over kind schemas (Q10); today the analysis knows which fields a rule reads, not their types |
| `not-redundant` | L11 | Outcome vectors over fixtures and live data: flag rules of the same kind whose failing sets are identical. Advisory first: tiered thresholds (fail rate 20/10/5%) legitimately coincide |
| `ratchetable` | L7, L12 | Mark connectors `volatile: true` (run history) and derive which rules read volatile fields; skip them from the ratchet automatically instead of `--skip` by hand, and flag a rule that fails on every entity only because its source is skipped |
| `within-budget` | L8 | Per-rule evaluation time measured by `derive` (sampled), with a budget per rule. Nodes and loops are the deterministic proxy until then |
| `evidence-names-items` | L2 | The evidence expression returns a list derived from entity data (a comprehension or a list field), not a constant sentence |

### 3.3 Superseded: LLM assessment pulled by Seikika

The first proposal had a `runner` job call a model with an API key, cached by content hash in a
committed `portal/assessments.json`. D79 replaced it with the push model in §6: the same rubric and
the same content-hash staleness, but the developer's own agent does the judging. Seikika needs no key,
no model choice and no egress, and assessments become entities rather than a cache.

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

## 4. Holistic review: which parts of a portal need standards

Every part of a portal is configuration that someone writes and others depend on. We asked of each:
can it be good or bad in a way that matters, can that be checked deterministically from the manifest,
tests and fixtures, and is it already covered?

| Part | What "bad" looks like | Decision | How |
|---|---|---|---|
| **Rules** | Untested, unexplained, gameable, expensive | Scorecard (D71) and rubric (D79) | `meta-rule`, `review-rule` (`rule-quality`) |
| **Scorecards** | Untested, empty levels, no ladder, one concern missing | Scorecard (D71) and rubric (D79) | `meta-scorecard`, `review-scorecard` (`scorecard-quality`) |
| **Kinds** | Never in fixtures, so their map rules and computed fields never run in tests; undescribed, so agents guess what an entity is | **Scorecard** | `meta-kind`: in fixtures, tested, described, fields described |
| **Fields** | Never populated in fixtures (mapping untested), undescribed, opaque `object`s no check can type | **Scorecard** | `meta-field`: populated, described, has a shape |
| **Connectors** | Declared records nothing maps; no fixtures; no freshness, so stale data looks current; no refresh, so an agent can't see its fix land | **Scorecard** | `meta-connector`: mapped, fixtures, freshness, refreshable |
| Field usage | 59 of 190 fields are read by no CEL | **Information, not a rule** | `Field.readBy`: display fields (titles, descriptions) are legitimately read by no rule |
| Relations | Edges to nothing; relations nothing reads | Covered | `relation.dropped` diagnostics, `unresolved: report`; `Kind.relationsIn/Out` for information |
| Map rules | Wrong ids, CEL errors | Covered | `seikika validate` (CEL, xrefs) and `meta-field` `populated-in-fixtures` |
| Computed fields | Errors on real data | Covered | `computed.error` diagnostics; computed fields are fields, so `meta-field` applies |
| Tests | Too few, one-sided | Covered | `meta-rule` `tested`/`both-outcomes`, `meta-kind` `tested`; a test kind would score the scorers of the scorers |
| Rubrics | Vague questions; criteria a check could answer | `seikika validate` + review | Kind and subject field exist, unique criteria; a rubric is reviewed with its portal change |
| Authorization model | Wrong access | Covered elsewhere | `fga model test` ([authorization.md](../authorization.md)) and the plan's authz diff |
| Actions | Unsafe self-service | Not yet | M5 is built after the self-service review (D01M86BY3YQX3SA1W99QXF1QA1H); `meta-action` is not |
| Crates, workflows, docs | Design that a check can't see | Rubric | `crate-design` (D79); workflows and docs can follow the same pattern |

**Why not a scorecard for everything?** Each meta scorecard adds config that must itself be tested
(they all are), and the sprawl budget (D45) applies to us too. A part earns a scorecard only when its
failure is common, checkable from facts we already have, and has a fix an agent can apply.
Everything else is a diagnostic, a validate error or information.

## 5. The rest of the model: kinds, fields, connectors

`seikika describe portal PORTAL…` (formerly `describe scorecards`, still an alias) adds three record
kinds to `scorecard` and `rule`. Every record carries a content `hash` of its definition, which is
what an assessment is versioned by.

| Record | Facts |
|---|---|
| `kind` | described; fields and how many are described; scorecards on it; relations in and out; fixture entities; whether a test names one |
| `field` | type; described; computed; `opaque` (`object`, or a list of objects); allowed sources; `readBy` (rules, evidence, `appliesTo`, computed fields, authority, tuple derivations); fixture entities and how many have a value |
| `connector` | runtime, package, declared records; `unmapped` records; records `withoutFreshness`; `refreshable` records; fixture messages |

| Scorecard | bronze | silver | gold |
|---|---|---|---|
| `meta-kind` | `in-fixtures`, `tested` | `described` | `fields-described` |
| `meta-field` | `populated-in-fixtures` | `described` | `has-shape` (not opaque) |
| `meta-connector` | `records-mapped`, `has-fixtures` | `freshness-declared` | `refreshable` |

All three use `ratchet: {newEntitiesAtLeast: bronze}`: a new kind, field or connector must arrive
exercised by fixtures. **First run, over 20 kinds, 190 fields and 9 connectors:**
- **Kinds:** three are in no test (`File`; the reference portal's `User` and
  `GitHubAppInstallation`), and 16 have undescribed fields. S4S's own kinds went to described on the
  same day.
- **Fields:** 129 have no description, including every field of the reference portal and the
  starter, and 20 are opaque objects (e.g. `Crate.tests`, `Rule.check`). Two have no value in any
  fixture: the live-only `Scorecard.liveEntities` and `Rule.live`.
- **Connectors:** the starter declared `repo` and `doc` with no freshness and no refresh, so pilots
  copying it would have shown stale data as current. Fixed in the same change. S4S's `workspace`
  lacked freshness for `workflow` and `action-ref`; fixed too.

## 6. Judgement, pushed by coding agents (D79)

**The loop:**

1. A portal declares **rubrics**: a kind, the criteria (each a question with optional guidance), and
   a `subject` field that versions what is assessed (default `hash`). `seikika validate` checks the
   kind, the subject field and unique criterion names.
2. **`seikika assess brief <entity> --portal <dir>`**, or the read-only **`assessment_brief`** MCP
   tool, gives the agent the rubric, the entity's facts (authorization applies, as for `get`), the
   subject hash, rubric hash, file path and a YAML template.
3. The agent reads the sources, judges each criterion `pass`, `fail` or `unsure` with a reason and a
   suggestion, and writes the file. The
   [`assess-entity`](../../plugins/seikika-operator/skills/assess-entity/SKILL.md) skill is the
   procedure.
4. **`seikika assess check <dir>`** validates every file against
   [`crates/seikika-model/schema/assessment.schema.json`](../../crates/seikika-model/schema/assessment.schema.json) and the rubric (every
   criterion answered exactly once, none invented, the entity and rubric match), and lists
   entities as fresh, stale or missing. A rubric edit makes files with a recorded rubric hash
   stale. CI runs this check.
5. **Storage**, either:
   - **as a file**: the workspace connector reads `assessments` globs into `assessment` records
     (repository portals);
   - **in Postgres**: **`seikika assess submit <file> --url …`** validates the file and pushes it to
     `/v1/ingest` as a remote connector's record (server portals).

   Both paths produce the same record (`seikika_connect::assessment`), so a portal maps it once.
6. The portal maps `assessment` records to an `Assessment` kind related to what it assesses. Computed
   fields compare `subjectHash` with the entity's hash (`reviewState`: fresh, stale or missing).
   Advisory `review-*` scorecards turn that into levels (since D01M831P5QWJQ4A2FT85N5MXJQF):
   - bronze: no open finding (no current review with a failing criterion);
   - silver: assessed and current, and for assessments, by another assessor;
   - gold: no failing criterion and no doubts.

**Why push:**

| | Pull (Seikika calls a model) | Push (the developer's agent assesses) |
|---|---|---|
| Keys, egress, model choice | Seikika's problem, per portal | None |
| Context | What the job sends | Whatever the agent can read: code, docs, history |
| Cost | Every changed entity, every run | When someone asks, or a review card shows it stale |
| Determinism in CI | Needs a committed cache | Files are data; CI only validates them |
| Audit | A cache entry | A reviewed file in a PR, or a record with its assessor |

**Why advisory:** an assessment goes stale whenever its subject changes, by design. If staleness
gated merges, every edit to a rule would need a re-assessment in the same PR. That is the right habit
for agents (the skill says so), but it is the wrong thing to block a person on.
`ratchet: {advisory: true}` keeps such cards out of the baseline and the comparison: reported, never
gating.

Routine `seikika assess check` reports fresh, stale and missing subjects. It fails on invalid
files, including missing verdicts for criteria. Land rubric additions with verdicts for the
affected assessments. Use `--require-fresh` for release preflight and high-risk review gates.

The [PR assessment pilot](pr-assessments.md) scopes deterministic checks and advisory judgment
to a candidate diff. It keeps evidence, runs and budgets outside the entity-assessment baseline
under D01M8D9AK7Q9B18B10FC38WB2BQ.

**First assessments (by this session's agent, using the brief):**

| Entity | Verdict | Finding |
|---|---|---|
| `user-mcp/contract-tested` | unsure | A call without an assertion still counts |
| `maintainer-ci-reliability/fail-rate-under-20` | fail | `continue-on-error` lowers the fail rate without fixing anything; the threshold is unsourced (L9) |
| `maintainer-crate/has-tests` | fail | Any `#[test]` passes, so it over-claims "tested" |
| `meta-rule/readable` | unsure | 30 nodes is unexplained; moving logic into a computed field escapes it |
| `maintainer-ci-reliability` (scorecard) | fail | No rule on `continue-on-error`, which undermines all three fail-rate rules |
| `crate:seikika-expr` | fail | `cel::Value` leaks through its public API, so cel-rust isn't replaceable (charter goal 3); crate docs are stale |

Four of the six found something no deterministic rule could. The crate assessment went stale within
the hour, when this change made lazy lists indexable, and was re-assessed. That is the staleness
mechanism working as intended.

### 6.1 Assessments and rubrics are standards too (D80)

An assessment is configuration someone wrote that others will act on, so it gets the same treatment
as a rule: deterministic checks that gate, and judgement that advises. The recursion stops after
one level, the same way §1's does.

| Layer | Scored by | Kind | Gates? |
|---|---|---|---|
| Rules, scorecards, crates | `review-rule`, `review-scorecard`, `review-crate` (rubrics `rule-quality`, `scorecard-quality`, `crate-design`) | — | No (advisory) |
| Every assessment | **`meta-assessment`** (deterministic): bronze, every fail or unsure has a `suggestion` and the assessor is named; silver, findings cite something checkable (a path, `D71`, `§4`, a number); gold, every reason does | `Assessment` | Yes (ratchet; new assessments must reach bronze) |
| Every rubric | **`meta-rubric`** (deterministic, from `rubric` records in `describe portal`): bronze, described and used; silver, every criterion has guidance; gold, three or more assessments and no criterion that all of them passed | `Rubric` | Yes |
| Assessments made with other rubrics | **`review-assessment`**, from an agent's assessment against **`assessment-quality`**: supported by its citations, complete, reproducible by a second assessor, suggestions that would change the verdict | `Assessment` | No (advisory) |
| Assessments made with `assessment-quality` | Only `meta-assessment` | — | **The stop:** `review-assessment` skips them (`appliesTo`) |

The assessment record carries what the deterministic card needs (`seikika_connect::assessment`):
- `unsuggested`;
- `uncited`, from a citation pattern;
- a content `hash`, which is what a review of the assessment is versioned by, so editing an
  assessment makes its review stale.

**First run:**
- **`meta-rubric` found that `rule-quality`'s `fix-leads-to-pass` passed in all four rule
  assessments.** A question nothing fails isn't discriminating; it is either sharpened or answered by
  `meta-rule`'s `explains-fix`. It also found five unguided criteria in `scorecard-quality` and
  `crate-design`; they now have guidance.
- **`meta-assessment` put 6 of 9 assessments below silver.** My own reasons often name a rule or a
  behaviour without a path or a number (e.g. "`continue-on-error` lowers the fail rate"). The check
  is a proxy, since a path is not proof, but it is the cheap half; `assessment-quality`'s `supported`
  is the expensive half.
- **Three `assessment-quality` reviews, of the `has-tests`, `seikika-expr` and
  `maintainer-ci-reliability` assessments:**
  - **`has-tests`, fail on `actionable`:** its suggestions add a companion rule but leave `has-tests`
    itself failing.
  - **`seikika-expr`, fail on `complete`:** it passed `one-responsibility` although the crate docs
    describe only M0, which its own suggestion admits.
  - **`reproducible` is `unsure` in all three,** because the same session wrote the assessments and
    the reviews. The rubric says to review someone else's assessment. Independence can't be checked
    deterministically yet: comparing the assessor with the author of the subject's last change needs
    commit authorship per entity.

**Not built:**
- disagreement between assessors (assessment history keyed by assessor; today a newer file replaces
  the older one);
- independence of the reviewer;
- detecting a rubric criterion that a deterministic rule already answers.

## 7. Predictions

| As Seikika grows… | The meta standard will need |
|---|---|
| **Fleets accumulate hundreds of rules across team packages** ([federation.md](./federation.md)) | Redundancy across packages, an owner per rule, and retirement: a rule passing on every entity for a long time becomes a guardrail (ratchet-only) or is removed |
| **Agents write rules** (they already write fixes) | `ratchet.newEntitiesAtLeast` on `meta-rule` as the merge gate: an agent-authored rule must arrive tested, explained and decided on fixtures, and a `rule-quality` assessment flags gameable ones |
| **Graduation needs a quality bar** | ~~A standard moves to `mandatory` only when its scorecard is at meta gold~~ Withdrawn (D01M831P5QWJQ4A2FT85N5MXJQF): meta gold is form. Promotion needs a current, independent, passing review and, where there is one, an agent trial |
| **Rules read more volatile data** (run history, incidents, costs) | First-class volatility, so ratcheted and observed standards are told apart automatically (§3.2 `ratchetable`) |
| **Evaluation cost becomes visible** at 100k+ entities and many packages | Per-rule cost budgets and the CEL analysis in `plan`: "this change adds a nested loop over 2M relations" |
| **Teams copy the starter** | The starter's own meta level matters most. It is gold on 4 of 6 rules today; `codeowners-catch-all` is untested and `docs/links-resolve` never passes on its fixtures |

## 8. Verification

- **Unit tests:**
  - CEL analysis: size, fields, loops, `join` and `flatten` undeclared, common functions declared;
  - `seikika validate` rejects an undeclared function (`cel.function`) and still accepts all three
    real portals;
  - the ratchet's new-entity floor, including the bootstrap case;
  - meta describe, run against three portals: 83 records (68 rules, 15 scorecards).
- **`seikika test portal`:** 40/40. Nine expectations are new; they cover every meta rule, including
  self-application: `meta-rule` scores `rule:seikika/meta-rule/tested`.
- **The ratchet:**
  - `--update` twice is stable, and `--ci` matches (a fixed point);
  - with the floor in force, adding the meta scorecards baselined every existing rule and
    scorecard;
  - before the bootstrap fix, the 27 below bronze were reported as regressions (alongside 3 real
    ones in our own code), which is why the bootstrap rule exists.
- **The ratchet caught this change, twice:** `compare` grew past 80 lines and complexity 15, and the
  CEL walker reached complexity 17. Both were split into named steps; the baseline was not loosened.
- **D79: the model and assessments:**
  - `seikika test portal`: 53/53. Thirteen new expectations cover every rule of `meta-kind`,
    `meta-field`, `meta-connector` and the three `review-*` cards, both ways: fresh, stale, missing,
    failing and unsure.
  - An end-to-end CLI test (`crates/seikika-cli/tests/assess.rs`) covers `describe portal`, `assess
    brief`, `assess check` (valid and invalid files) and `assess submit` into a running server, where
    a scorecard reads the pushed assessment.
  - An MCP contract test covers `assessment_brief`, including an unknown rubric.
  - Lazy `inbound` lists are now indexable (`inbound.r[0]`), with a unit test; computed fields
    needed it.
  - The ratchet caught this change too: `main` passed 150 lines, the workspace `resync` reached
    complexity 19, and new public items were undocumented. All were fixed; the baseline was not
    loosened.
- **Not built:** agent trials, type checks, redundancy, volatility, per-rule timing
  (§3.2, §3.4).

## Sources

- Lessons: [seikika-for-seikika.md](./seikika-for-seikika.md) §5–7 (F1–F14 and the fresh-agent run),
  [ratchets.md](./ratchets.md), [github-actions-standards.md](./github-actions-standards.md),
  [research-log.md](../research-log.md), [decisions.md](../decisions/README.md) D65–D70.
- Prior art for self-application: a compiler compiling itself; the
  [meta-catalog](./meta-catalog.md) design (a portal catalogues its own configuration, "including the
  scorecards that score scorecards").
- Implementation: `crates/seikika-expr/src/analysis.rs`, `crates/seikika-cli/src/meta.rs` and
  `model.rs` (`describe portal`), `crates/seikika-cli/src/report.rs` (`newEntitiesAtLeast`,
  `advisory`), `crates/seikika-cli/src/assess.rs`, `crates/seikika-api/src/assess.rs`
  (`assessment_brief`), `crates/seikika-connect/src/assessment.rs`,
  [`crates/seikika-model/schema/assessment.schema.json`](../../crates/seikika-model/schema/assessment.schema.json), `portal/seikika.yaml`
  (`Kind`, `Field`, `Connector`, `Assessment`, the meta and review scorecards, the rubrics),
  `portal/assessments/`.
