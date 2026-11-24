# Round 8: Toolbox passes the bar on a held-out task

Round 8 ran each arm three times against Toolbox frozen at `8519cd5`, on a task no Toolbox doc,
skill or example was written for: `add-notice` ([task-notice.md](../../prompts/task-notice.md),
README §8). Every one of the twelve runs fully passes: C1 and P1–P11 hold in each, as four
independent adjudicators observed them ([ADJUDICATION.md](ADJUDICATION.md), records in
[adjudication/](adjudication/)). No run needed a fix to be operated from its handoff.

**The bar passes.** H1 holds: every arm delivers 3 of 3. Against every rival, H2 holds by more
than W, and nothing is lost. The pass rests on H2, which the
[fit audit](../FIT-AUDIT-r8.md) discounts little, not on the labelled calls results.

| Arm | Lines to maintain (median, range) | Tool calls (median, range) | Wall time (s, median) | Fully passes |
|---|---|---|---|---|
| Toolbox | 241 (186–242) | 36 (23–42) | 364 | 3 of 3 |
| GitHub Actions | 1330 (1056–1339) | 52 (47–57) | 968 | 3 of 3 |
| Backstage | 1882 (1855–2649) | 81 (75–85) | 1284 | 3 of 3 |
| Vanilla | 952 (536–1160) | 20 (16–30) | 411 | 3 of 3 |

| Rival | H2 lines | H3 tool calls | Bar |
|---|---|---|---|
| GitHub Actions | holds: 241 vs 1330, W 283 | inconclusive: 36 vs 52, W 19 | met |
| Backstage | holds: 241 vs 1882, W 794 | holds\*: 36 vs 81, W 19 | met |
| Vanilla | holds: 241 vs 952, W 624 | inconclusive: 36 vs 20, W 19 | met |

\* *With a shipped task skill tuned on rounds 4–7 and extended for R9 before this round.* Not a
held-out result (fit audit §1).

## How to read it

The [round 8 fit audit](../FIT-AUDIT-r8.md) found P4 and P5 hold with corrections. Its labels
apply to these numbers:

- **Delivery (H1): no discount** for R1–R8 or the held-out effect. **Toolbox's P9 passes and its
  R9 help come from a change built for round 8's task before the freeze** (README §8, limit 8):
  they show that group approvers resolved in a checkout work, not that Toolbox handled a
  requirement no one anticipated.
- **Lines (H2): little discount.** The R9 help shapes only Toolbox's `authz` block and a short
  model. Most of Toolbox's kept lines are the README script each run designed and tested itself,
  for an effect no doc covers. The narrowest margin is vanilla's: 711 lines against W 624.
- **Calls (H3): labelled.** Toolbox's calls come with the shipped `add-action` skill; the rivals
  had their platforms' untuned reference docs, and gha had no GitHub team docs. Against vanilla,
  which needs no docs at all, Toolbox's calls are higher (36 vs 20) and the result is
  inconclusive, not won.

Where Toolbox's lines go: toolbox-1 kept 241. Its org portal is 20 lines with a 6-line group
model. Each repository has a 4-line portal file. The action is a 31-line definition, a 36-line
README script and 131 lines of tests, which are committed in each repository and counted once,
as copies (§5.1). Approvals and their refusals, run records, the origin read before success and
the org-level group come from Toolbox. The rivals wrote those themselves:

- **Vanilla:** 536–1160 lines.
- **GitHub Actions:** 1056–1339 lines. Each run built a local approval gate and run history
  around act, since GitHub's environments can't run in the sandbox.
- **Backstage:** 1855–2649 lines, for backend code for approvals and audit, scaffolder templates
  and actions, and a CLI.

## Checks

- **Prompts and isolation (P1, P4):** all twelve runs got exactly their committed prompt
  (`<run>.prompt.json`, `identical: true`), and no transcript reads another run's workspace or lists
  the round's directory (`<run>.isolation.json`, empty). Fit audit correction 5 is therefore met.
- **Frozen product (P3):** every run used the binary and docs copied at setup from `8519cd5`.
  After the freeze, only the fit audit's corrections 1 and 7 touched the product (`38e9549`:
  `group:release` renamed to `group:security` in the example, its tests and the acceptance script,
  and a decision amending the `add-action` skill's scope). All three Toolbox runs had finished
  before then.
- **Strikes:** only runtime data, by one rule for every arm. Toolbox: run records and the
  generated key. GitHub Actions: run records, a run counter and act step results. Backstage:
  backstage-3's leftover scaffolder clone and two scratch id files; none in runs 1 and 2. Vanilla:
  run records, a run counter and clone metadata.

## Disclosed by the runs and adjudicators

- **GitHub Actions:** identity is the `--as` claim. The approval gate is a local wrapper anyone
  with filesystem access can bypass, and the GitHub-side settings scripts were only dry-run.
  Every handoff says so.
- **Backstage:** backstage-3 let a test run land on alpha's `main` (carol was not taken back out
  of the group) and reverted it by hand before the final run. backstage-1 keeps its org origin
  path and database settings in a gitignored `app-config.local.yaml`.
- **Toolbox:** in a checkout, `--as` is also taken on trust; a served portal authenticates
  (D01M8727W7FEED6VVRY6MB3JFR3, `add-action`).

## The fit audit's corrections

| # | Correction | Status |
|---|---|---|
| 1 | Rename round 8's group in the product | Done in `38e9549` (`group:security`) |
| 2 | Label R9 and the calls results | Done here and in the catalog entry |
| 3 | Seal the next held-out task before the product freezes | Recorded for any later round |
| 4 | Give the gha arm GitHub's team docs | Done in `harness/fetch.sh` for any later round |
| 5 | Check gha-3's, backstage-2's and backstage-3's prompts | Done: identical |
| 6 | Fill the catalog entry | Done in `portal/benchmarks/actions-ab.yaml` |
| 7 | Record that `add-action` no longer covers the handoff | Done: D01M87579Y5AZWR1Z29S802VP2K |
