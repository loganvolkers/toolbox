# Thresholds and Levels: Why Each Number Is Where It Is

*Status: reference, 2026-11-23 (D01M83G9F4EEGN1GTP14HEYB8B6). About the box
([toolbox-and-tools.md](./toolbox-and-tools.md)): how this portal's standards pick their numbers
and levels. Maintainer SDLC. The `rule-quality` rubric asks whether each threshold and level is
justified (`threshold-justified`); this page is where the justification lives, so a fix text or
an assessment can cite it.*

---

## 1. Conclusion

**Every number here is either measured on this repository or chosen and labelled as a choice.**
- **Measured:** the number comes from the distribution on our own repositories, and the
  measurement and its date are given.
- **Chosen:** the number is ours, with the reason, and is labelled as such.

**Three principles place a rule on the ladder:**
- **Safety-critical rules go in the floor.** Gawande's killer items, and the reason GitHub blocks
  a pushed credential outright: a credential or a hidden instruction can't sit at gold, because
  the bronze ratchet would then let it through.
- **A prerequisite sits below what depends on it.** For example, `failures-visible` sits below the
  fail-rate rules.
- **Gold asks for what lower levels don't already imply.** A gold rule that bronze and silver
  together imply decides nothing.

## 2. The numbers

| Rule (scorecard) | Number | Why | Kind |
|---|---|---|---|
| `fail-rate-under-20`, `-10`, `-5` (maintainer-ci-reliability) | 20%, 10%, 5% of decided runs on the default branch that failed on their first attempt; silver and gold also need at least 10 decided runs (`minDecidedRuns`) | A run that passed only on a re-run counts as failed, so re-running a flaky job doesn't lower the rate; that replaced a separate `few-reruns` rule, which sat above the fail rate it guarded. At 10 runs one failure is 10%, the silver bound, so fewer runs can't tell the tiers apart (chosen). This repository's `ci` on main failed 32 of its last 100 runs (2026-11-20 to 2026-11-23, design phase, committing straight to main per D32), so bronze is the next step from where we are. Each level halves the share of red runs that a reader must check. DORA's "change failure rate" bands (elite 0–15%) are context, not a source: they measure deployments, not CI runs | Chosen, anchored to a measurement |
| `failures-visible` (maintainer-ci-reliability) | Bronze, no allowance | `continue-on-error`, `\|\| true`, `set +e` and an `if:` that skips the job on the default branch each hide a failure from every fail-rate rule, so it is their prerequisite. A step that is meant to fail passes when a later step reads `steps.<id>.outcome`, as this repository's `report-action` job does | Principle |
| `still-running` (maintainer-ci-reliability) | Bronze | GitHub disables a workflow after 60 days without activity, and its old run history then reads green | Principle |
| `cancels-superseded-runs`, `fast-feedback` (maintainer-ci-speed) | Bronze, and silver at a p90 of 15 minutes | They save runner time and queue latency; they aren't correctness or security properties, so they live on their own card rather than holding a reliable workflow back. 15 minutes is a chosen bound, the point where developers switch task while waiting | Principle; the 15 minutes chosen |
| `no-hardcoded-secrets` (10 scorecards), `no-hidden-instructions` (skills, agent docs) | Bronze | They are killer items. Before D01M83G9F4EEGN1GTP14HEYB8B6 they sat at gold, so a skill or connector carrying a credential or a hidden instruction still read silver and passed the bronze ratchet. SkillBench grades hidden Unicode as a hard F ([skill-standards.md](./skill-standards.md)) | Principle |
| `has-tests` (maintainer-crate) | Bronze: at least one test, and every test asserts something that can fail | A test with no assert, `unwrap`, `expect` or `?` passes whatever the code does, and so does `assert!(true)` or `assert_eq!(x, x)`. A call to a helper in the same file that asserts counts, so table-driven tests aren't flagged. On 2026-11-23 two tests in this repository asserted nothing; both now assert | Principle, with a measurement |
| `tests-scale-with-code` (maintainer-crate) | Silver: at least 2 tests per 1,000 code lines | On 2026-11-23 crates here ranged from 0 to 14 tests per 1,000 lines, with a median near 6. A floor of 2 flags crates that are untested or nearly so; the three below it (`toolbox-connector-git-files` and `toolbox-connector-scim` at 0, `toolbox-store` at 1.9) gained unit tests in the same change. It is not a coverage target | Measured |
| `tests-pass` (maintainer-crate) | Bronze, no ignored or compiled-out test | An ignored test never runs, nor does one under `#[cfg(any())]`. Tests that need OpenFGA fail instead of returning early, in CI and in the portal's runner job (`TOOLBOX_REQUIRE_OPENFGA`) | Principle |
| `titled` (maintainer-site) | A top-level title of at least 3 characters | The shortest real page titles are words such as "Run". Two characters is a placeholder, and a blank or nested title isn't a title | Chosen |
| `unique-title` (maintainer-site) | Gold | Two pages with one title can't be told apart in search or the sidebar. It is polish, not access | Principle |
| `readable` (meta-rule) | Silver: at most 30 CEL nodes in a check | Over 219 rules in three portals on 2026-11-23, the median was 5 nodes, the 95th percentile 16 and the maximum 25. 30 leaves headroom without hiding logic. Every rule meets it, so it is a guard at silver, not what makes gold | Measured |
| `pins-both-outcomes` (meta-rule) | Gold | Lesson L4: a rule whose tests only pin it failing can start passing everything unnoticed. Comparing a rule's evidence pins `fail`; a bare mention pins nothing | Principle |
| `computed-readable` (meta-field) | At most 60 nodes and no nested loops | Computed fields are where logic belongs, so they get twice a check's budget. Over 59 computed fields, the median was 12 and the 90th percentile 36; the one at 65 was split | Measured |
| `explains-fix` (meta-rule) | At least 5 words | Five words is the shortest instruction that says what and where ("Set `timeout-minutes` on each job") | Chosen |
| `guided` (meta-rubric) | At least 10 words, an outcome as a word (fail, pass or unsure) and a condition (when, if, unless, "and name") | The shortest real guidance was 13 words, bar one of 7 that didn't say when to fail and was rewritten. "Fail when the answer is no." names an outcome and says nothing. A lexical check is only a floor: whether guidance is good is what assessors' agreement measures (meta-standards-review.md, M2) | Chosen, with a measurement |
| `evidence-tracks-check`, `fix-names-a-place` (meta-rule) | Gold. A fixed evidence sentence counts only for a yes/no check of at most 8 CEL nodes, no loop and no list that must be empty | Lessons L1 and L2 ([scorecards-for-scorecards.md](./scorecards-for-scorecards.md) §2): agents fixed what the evidence named and the fix located. The first version (evidence must read a field the check reads) failed 68 of 219 rules, most of them falsely: `no-huge-functions` checks `maxFnLines` and lists `functions`, and for `has(entity.license)` a fixed sentence is the whole answer. Every constant-evidence rule in our portals was a check of 8 nodes or fewer. After narrowing, and after evidence and code spans were added to 19 toolbox rules, 213 and 217 of 219 rules met each on 2026-11-23; the rest are the reference portal's, left as written | Principle, with a measurement |
| `described` (meta-scorecard) | Silver | Agents read a scorecard's description before its rules. It sits at silver, not bronze, so that adding the rule didn't require editing the reference portal's byte-exact oracle | Principle |
| `fixtures-reach-every-level` (meta-scorecard) | Gold | A level no fixture reaches is untested; the silver ladder only needs two outcomes | Principle |
| `contract-tested` (user-mcp) | Gold: a successful observed call, or, where the runner is skipped (as in CI), a `tools/call` body with `name` and `arguments` outside a comment | An observed call shows the tool answers over MCP end to end; the contract tests' helper only checks the response shape (`crates/toolbox-api/tests/mcp.rs`, `call`), and what each test asserts beyond that is per test, so this rule is about reach, not depth. A call that errors proves nothing. The static mode is weaker: it sees that a test sends the call, not that the call succeeds | Principle |

## 3. The ladders

| Scorecard | Bronze | Silver | Gold |
|---|---|---|---|
| `review-*` (rule, scorecard, crate, sdlc-stage, assessment) | No open finding: no review, current or stale, that failed a criterion | A current review. For an assessment, the review must be by another assessor | A current review that passes, cites what resolves, and has no doubts. For a crate, also a second assessor's review of that assessment that passes and cites it (`independently-reviewed`). For an assessment, the review must cite the assessment's file or a source it cites (`groundedIn`) |
| `sdlc-stage` | Status is one of the four, an exit criterion, a doc that exists | Commands that resolve, and checks a workflow executes (a `run:`, `uses:` or `with:` value, not a step name or comment) | Status `built` with no gaps |
| `maintainer-ci-reliability` | Not stuck red, still running, under 20% first-try failures, failures visible, job timeouts | Under 10% over at least 10 runs | Under 5% over at least 10 runs |
| `maintainer-ci-speed` | Superseded runs cancelled | p90 under 15 minutes | — |
| `sdlc-layer` | Every stage declared | No stage missing in fact: each has a doc that exists and names the stage, whatever its status says | Every stage built |
| `meta-scorecard` | Tested, no empty level | Described, a fixture ladder with a real level, every rule tested | Fixtures reach every level |

**Not checked, and why:**
- **Who wrote an assessment is self-declared** (`assessor.name`). Every commit here has the same
  author, whichever agent made it, so git can't tell assessors apart either. Renaming the reviewer
  passes the independence check, but that is a false statement in a committed file, visible in
  review. What is checked is the cheap route: a review must engage with its subject, citing the
  assessment's file or a source it cites, so a review of "Looks fine to me." citing any file in
  the repository doesn't count. The connector links reviews to what they review (`groundedIn`,
  `independentReview`), so `review-crate` can require a second assessor without reading another
  entity's computed fields.
- **Where a level can be granted depends on where the portal runs its connectors.** Every level
  is reached by a fixture in tests (`fixtures-reach-every-level`). The ratchet skips connectors
  whose data changes with time (run history); the published report runs them on main, so
  `maintainer-ci-reliability`'s levels are reachable where people read them.
- **`sdlc-stage` gold still reads the declared status.** The silver facts bound it: a stage can't
  reach gold without commands that run and checks CI runs. Tying gold to a current review would
  make every edit drop the stage a level, which D01M831P5QWJQ4A2FT85N5MXJQF rules out.

## Sources

- This repository, 2026-11-23:
  - `toolbox describe scorecards` over three portals;
  - `toolbox connector records --name workspace`;
  - GitHub Actions runs of `ci` on main, the last 100.
- Gawande, *The Checklist Manifesto* (2009), secondary summaries
- DORA, *Accelerate: State of DevOps* (change failure rate bands), secondary
- [meta-standards-review.md](./meta-standards-review.md), [scorecards-for-scorecards.md](./scorecards-for-scorecards.md), [skill-standards.md](./skill-standards.md)
