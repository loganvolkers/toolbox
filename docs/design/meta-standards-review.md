# Do the Meta Scorecards Measure Anything? A Re-evaluation

*Status: assessment and changes, 2026-11-23 (D01M831P5QWJQ4A2FT85N5MXJQF). This re-evaluates the
top level of [scorecards-for-scorecards.md](./scorecards-for-scorecards.md) (D71, D79, D80). About the
box ([seikika-and-tools.md](./seikika-and-tools.md)): how the box judges the standards it carries.
Maintainer SDLC.*

---

## 1. Conclusion

**The deterministic meta scorecards are a working hygiene floor, not a measure of quality.** Their
gold means *well-formed and tested*, not *good*:
- 173 of 210 rules reach `meta-rule` gold.
- Of the 19 meta-gold rules and scorecards that agents have reviewed, **none passed**: 17 failed and 2
  were unsure.
- An independent, adversarial review of a random sample failed all 16 of its entities, 33 of 64
  criteria, with a concrete way to game each `hard-to-game` failure (§2.2).

**The agents are not silently agreeing in their reviews; the system silently ignored the reviews.**
Reviewers fail things: 29% of verdicts in the first round and 52% in the adversarial one. The
agreement problem sits in three other places:

1. **Authors satisfy the form the meta standard checks** (Goodhart's law, §3). Rules arrive tested,
   with fixtures that decide both ways and evidence, because `meta-rule` demands it. That is real
   value, but it is compliance with a form. A fixture written to make a rule decide both ways says
   nothing about whether the rule measures its intent.
2. **Findings went unacted.** The two fresh failing reviews from 2026-11-20 were still open three days
   later. The `review-*` cards are advisory, and their two real findings were buried among about 200
   "not assessed" lines.
3. **Self-review was possible, and happened.** One of the three existing assessment reviews was
   written by the same agent as the assessment it reviewed. The `fix` text said "a different agent or
   person should review it", but nothing checked.

**Changes made (§4):**
- **Findings first:** each `review-*` card now drops an entity with a current failing review to
  `none`, so open findings head the report.
- **Independent reviews:** a self-written review no longer counts.
- **Closed gaming routes:** blank descriptions and blank guidance no longer pass, and fixture
  ladders need a real level.
- **Tests are matched to their scorecard:** a rule counts as tested only by an expectation on its
  own scorecard.
- **A campaign** closes the open findings.
- **The record:** 16 independent assessments are committed as data. The four that these changes
  touched were re-assessed afterwards by the same reviewer (§2.3).

**What would show the meta standard works** is outcome validity: whether raising a standard's level
improves what it measures. It needs agent trials and calibration between assessors, not more
deterministic rules (§5).

## 2. Evidence

### 2.1 The deterministic layer saturates

| Meta scorecard | Entities | Gold | Below bronze | What its gold requires |
|---|---|---|---|---|
| `meta-rule` | 210 | 173 (82%) | 29 | Fix ≥ 5 words, tested, both outcomes on fixtures, evidence, ≤ 30 CEL nodes, no nested loops |
| `meta-scorecard` | 43 | 28 | 1 | Tested, no empty level, a fixture ladder, every rule tested |
| `meta-field` | 411 | 184 | 2 | A fixture value, a description, a declared shape |
| `meta-kind` | 37 | 12 | 3 | Fixtures, tests, descriptions |

Each requirement is a property of the *form* of a standard. None asks whether a rule measures what
its name says, can be gamed, or changes anything. The design said so in 2026-11-20 and left those
questions to judgement ([scorecards-for-scorecards.md](./scorecards-for-scorecards.md) §1). Its
prediction that "a standard moves to `mandatory` only when its scorecard is at meta gold" (§7 there)
would have promoted standards on form alone. This review withdraws that prediction.

### 2.2 Judgement contradicts gold

**First round** (2026-11-20, `claude-code` and `cursor-grok`): 14 assessments, 56 verdicts, 16
fail and 7 unsure. Of the 5 rules and scorecards assessed, none passed:
- `maintainer-ci-reliability/fail-rate-under-20` was at meta gold and failed `hard-to-game`;
- `meta-rule/readable` and `user-mcp/contract-tested` were at meta gold and reviewed as unsure;
- `maintainer-crate/has-tests` failed `measures-intent` and `hard-to-game`;
- the `maintainer-ci-reliability` scorecard failed `no-obvious-gap`.

**Adversarial round** (2026-11-23, a fresh agent, `blind-review-2026-11-23`). The sample was 12
meta-gold rules, one per scorecard drawn at random (seed `20261123`), and 4 meta-gold scorecards:
`meta-rule` and `meta-scorecard` themselves, plus 2 drawn at random. Each `hard-to-game` pass needed
a gaming attempt that failed first. **All 16 failed: 33 of 64 criteria, and 1 unsure.**

| Finding | Entities | Checked here |
|---|---|---|
| Presence, not quality: an empty or whitespace value passes | `maintainer-site/titled`, `meta-rubric/guided`, `meta-connector/records-mapped` (a map with `set: "{}"`) | `guided` read in code (`model.rs`, `guidance.is_none()`); fixed (§4) |
| Self-declared facts are trusted | `sdlc-layer/nothing-missing` and `sdlc-stage` gold read the author's own `status:` | Read in `.seikika/standards/sdlc.yaml` |
| Self-review is possible | `review-assessment`, `review-crate` | Confirmed in data: `crate-design/crate:seikika-expr` was assessed and reviewed by `cursor-grok`; fixed for assessments (§4) |
| A fixture ladder of `none` and `skipped` passes silver | `meta-scorecard/fixtures-show-the-ladder` | Read in `meta.rs` (`fixture_outcomes` counts every level string); fixed |
| Tests are paired loosely | `meta-rule/tested` pairs every scorecard and rule in one expectation | Read in `meta.rs`; no rule is affected today, but the route is closed (§4) |
| Gold rules cheaper than bronze ones | `meta-rule/always-explains` (a constant string passes); `sdlc-stage/built` (a two-line edit) | `always-explains`: by definition |
| Detection gaps in security rules at gold, not bronze | `maintainer-skill/no-hidden-instructions` (missed code points), `maintainer-connector/no-hardcoded-secrets` (`#[cfg(test)]` hides the file; `sk_live_` missed), `maintainer-python/no-findings` (bare `# noqa`), `maintainer-crate/tests-pass` (`#[ignore]`) | Reviewer's reproduction, not repeated here |
| A wrong fact in the brief | The brief's live facts for `maintainer-python` read the committed baseline, where it is `{}` | Not checked |

**Limits of this evidence:**
- The adversarial reviewer was told that "a reviewer who passes everything is useless". That pushes
  against sycophancy, and may push towards contrarian verdicts (§3, Sharma et al.).
- Its claims checked here all held, but its other reproductions were not repeated.
- Both rounds used the same model family. Zheng et al. measured self-enhancement bias in model
  judges, so agreement between them would overstate reliability.

### 2.3 The changes, reviewed by the same independent agent

The four entities these changes touched were re-assessed at their new versions by the same fresh
reviewer (not by the author of the changes). All four still fail, and the results are in
`.seikika/assessments/`:

| Entity | Fixed | Still holds, or new |
|---|---|---|
| `review-assessment/passes-review` | Self-review now fails silver | Independence compares only the names assessors give themselves: rename the reviewer and four "Looks fine to me." passes reach gold. The review's own citations (`uncited`) aren't read |
| `review-crate/passes-review` | — | The crate hash covers `src/` only, not `tests/` or `Cargo.toml`. Crate assessments have no independence check |
| `meta-scorecard` | `none` with `skipped` fails silver | One real level with `skipped` still passes. There is no `described` rule, although scorecards can have a description |
| `review-crate` | — | **New: a one-byte edit makes a failing review stale and hides the finding at bronze.** Findings first only covers current reviews. The `no-doubts` fix suggests a route (`docs/questions.md`) that can't pass |

So the two `passes-review` rules are now redundant at gold, and the findings-first ladder can be
escaped by touching the subject. Both are open items (M7, M8 in §5).

## 3. The literature, and where our meta standard stands

| Source | What it says | Our meta standard | Verdict |
|---|---|---|---|
| **Goodhart (1975), Campbell (1969–79), Strathern (1997); Muller, *The Tyranny of Metrics* (2018)** | When a measure becomes a target it stops being a good measure; the more a metric decides things, the more it is corrupted | The meta checks are targets: the ratchet's new-entity floor gates every new rule on them, and authors write to them | **Confirmed by our data.** Form is satisfied, and quality isn't measured. Keep gates on form, never call form "quality", and don't gate on judgement (§4) |
| **Basili, Caldiera, Rombach, "The Goal Question Metric Approach" (1994); Jaspan, "Measuring Engineering Productivity", *Software Engineering at Google* ch. 7 (GSM)** | Start from a goal, ask the question that would show it, then pick the metric; check that the result is actionable before collecting it | Rules have a fix, but no stated goal or question; the `has-rationale` rule (`why:`) is proposed, not built | **Gap.** Without a goal, a rule can't be judged a proxy, which is what `measures-intent` keeps failing on |
| **Kaplan and Norton, the Balanced Scorecard (1992, 1996)** | Pair lagging outcome measures with the leading drivers believed to cause them, and state the cause-and-effect hypothesis | Every meta rule is a leading indicator of form; no outcome is measured | **Gap.** The outcome is whether a standard improves its subject; only agent trials and DX Benchmarks measure that |
| **Beck and Orosz, response to McKinsey (2023)** | Measure outcomes and impact, not only effort and output, and expect measurement to change behaviour | Meta gold measures the output of rule authors | **Gap**, and the behaviour change is visible: fixtures written to satisfy `meta-rule` |
| **Ford, Parsons and Kua, fitness functions (*Building Evolutionary Architectures*, 2017)** | An objective integrity assessment of an architectural characteristic, automated where possible | The deterministic meta rules are fitness functions over a portal's configuration | **Aligned.** Keep them, for what they are: integrity, not value |
| **Gawande, *The Checklist Manifesto* (2009, secondary sources)** | Short lists of killer items at a defined pause point, tested in the real world | Our pause point is the new-entity floor. Killer items are not ranked: security detections sit at gold | **Partly aligned.** Rank safety-critical rules into the floor (§5, M4) |
| **Google SRE, "The Evolving SRE Engagement Model" (*SRE* ch. 32)** | Production readiness reviews by an independent team, with checklists that evolve | Reviews exist, but nothing required independence, and checklists don't evolve from incidents | **Fixed in part:** independence for assessment reviews (§4) |
| **Zheng et al., "Judging LLM-as-a-Judge" (NeurIPS 2023)** | Strong model judges agree with people about as often as people agree with each other, with position, verbosity and self-enhancement biases | One model family judges everything; no calibration against people | **Gap:** calibration (§5, M2) |
| **Sharma et al., "Towards Understanding Sycophancy" (2023)** | Assistants tend to agree with the user; preference data rewards it | The risk the goal named | **Not observed in reviews**: they fail things. It is observed in authorship (point 1 of §1) |
| **Anthropic, "Demystifying evals for AI agents" (2026)** | Model graders need calibration with human graders; read the transcripts; graduate saturated evals into regression suites | `meta-rule` is saturated at 82% gold | **Aligned once renamed:** the deterministic layer is a regression suite, and judgement is the capability signal |
| **Cortex scorecard documentation and initiatives (vendor)** | Scorecards are living documents; initiatives give a subset of rules a deadline | Campaigns exist; none was aimed at review findings | **Fixed:** `close-review-findings` (§4) |

## 4. Changes made

| # | Change | Why | Where |
|---|---|---|---|
| C1 | Every `review-*` card: bronze `no-open-findings` (fails only on a current failing review), silver `assessed`, gold `passes-review` and `no-doubts` | Open findings head the report as `none`, distinct from "not reviewed yet" at bronze. Reviews stay advisory: a stale review never blocks a change | `meta.yaml`, `crates.yaml`, `sdlc.yaml`, `assessments.yaml` |
| C2 | `review-assessment/assessed` fails when the review's assessor is the assessment's own (`selfReviewed`) | Independent review is what the fix text promised (SRE PRR, judge bias) | `assessments.yaml`, a fixture, a test |
| C3 | `fixtures-show-the-ladder` needs at least one real level reached (`fixtureRealLevels`) | `none` with `skipped` was passing silver | `meta.yaml` |
| C4 | Blank `description:` and `guidance:` don't count, for kinds, fields and rubrics | The cheapest gaming route for `described` and `guided` | `crates/seikika-cli/src/model.rs`, a unit test |
| C5 | A rule is tested only by an expectation on its own scorecard (`score_segments`) | One expectation naming two cards used to test every rule named in it | `crates/seikika-cli/src/meta.rs`, a unit test |
| C6 | Campaign `close-review-findings` (review-rule and review-scorecard at bronze) | Findings are worked off against a date, not just listed | `.seikika/campaigns.yaml` |
| C7 | 16 adversarial assessments committed | Judgement is data (D79), and the findings are now open items on the campaign | `.seikika/assessments/` |

**Deliberately not changed:**
- **No regex for "the fix names a place".** Half of our fixes rightly say "in the evidence", and the
  evidence names the place. A regex would teach authors to add backticks.
- **Judgement doesn't gate the ratchet.** A review goes stale when its subject changes, so gating
  would block changes. A failing review would also punish the reviewer who wrote it (Campbell).
- **Live outcomes aren't scored.** Determinism, and a scorecard shouldn't get worse because the
  entities it scores got better.
- **The rule-level findings are left to the campaign.** The two security detections lead it.

**A regression accepted as a finding.** With five scorecard reviews in, `meta-rubric` reports that
`scorecard-quality`'s `one-concern` and `actionable` passed every time. Either those criteria are
too soft, or our scorecards are good on them. That is an open question (M5), not a reason to tune
the rubric on five samples.

## 5. Next

| # | Item | Source |
|---|---|---|
| M1 | **A sampled, adversarial review each release:** a fresh agent reviews a stratified random sample of meta-gold rules and scorecards. Track the fail rate; it should fall as findings close | §2.2; BetterBench and ABC ([dx-benchmarks.md](./dx-benchmarks.md) §5) |
| M2 | **Calibration:** two independent assessors on the same subjects, plus a person's spot check. Report agreement (Cohen's kappa) per rubric criterion, and distrust criteria where it is low | Zheng et al.; Anthropic 2026 |
| M3 | **Goals for rules:** an optional `why:` naming the goal and question (GQM). `measures-intent` is then judged against it | Basili 1994; GSM |
| M4 | **Killer items in the floor:** security detections such as hidden instructions and hard-coded secrets move to bronze, and their detection gaps are fixed | Gawande; this review |
| M5 | **Sharpen or confirm `one-concern` and `actionable`** once ten scorecards are reviewed | §4 |
| M7 | **Findings survive staleness:** a stale review whose failing criteria were never answered stays an open finding until it is re-assessed (or the change says why); otherwise a trivial edit hides it | §2.3 |
| M8 | **Stronger independence and quality:** compare assessor identity beyond a self-chosen name (the commit author of the assessment file), check a review's own `uncited`, give crate assessments the same check, hash a crate's `tests/` and `Cargo.toml`, and drop the redundant gold `passes-review` | §2.3 |
| M6 | **Outcome validity:** agent trials (`agent-can-raise-it`, [scorecards-for-scorecards.md](./scorecards-for-scorecards.md) §3.4) and DX Benchmarks that raise one standard and check whether its subject improved | Kaplan and Norton; Beck and Orosz |

## Sources

- Goodhart (1975); Campbell (1969/1979); Strathern, "'Improving ratings': audit in the British University system" (1997); Muller, *The Tyranny of Metrics* (2018). Read through secondary sources (Wikipedia, NN/g); exact wording not checked
- Basili, Caldiera and Rombach, "The Goal Question Metric Approach", *Encyclopedia of Software Engineering* (1994), cs.umd.edu (abstract and summaries)
- Jaspan, "Measuring Engineering Productivity", *Software Engineering at Google*, ch. 7, abseil.io/resources/swe-book (summaries; the chapter is the primary)
- Kaplan and Norton, "The Balanced Scorecard" (HBR 1992), *The Balanced Scorecard* (1996), "Linking the Balanced Scorecard to Strategy" (CMR 1996) (secondary)
- Beck and Orosz, "Measuring developer productivity? A response to McKinsey" (The Pragmatic Engineer, 2023) (summaries)
- Ford, Parsons and Kua, *Building Evolutionary Architectures* (2017), ch. 2 (definition quoted from search results)
- Gawande, *The Checklist Manifesto* (2009) (secondary summaries)
- Beyer et al., *Site Reliability Engineering*, ch. 32, "The Evolving SRE Engagement Model", sre.google (summary)
- Zheng et al., "Judging LLM-as-a-Judge with MT-Bench and Chatbot Arena" (NeurIPS 2023), arXiv 2306.05685 (abstract)
- Sharma et al., "Towards Understanding Sycophancy in Language Models" (2023), arXiv 2310.13548 (abstract)
- Grace, Hadfield, Olivares, De Jonghe, "Demystifying evals for AI agents" (Anthropic, 2026), read in full
- Cortex documentation, scorecards and initiatives (vendor)
- This repository: `.seikika/results.json`, `.seikika/assessments/`, `seikika assess check .seikika`, `seikika describe scorecards portal`
