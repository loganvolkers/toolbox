---
id: D01M7Y5VWKW5TDS0VQWZFPRQ27N
decided: 2026-11-21T04:29:37Z
---

# Missing connector data is unknown, and it does not grant a level that has no data

**A rule that reads a connector scores `unknown` when that data is missing, and an all-unknown level is not granted.** This amends D70. `dataFrom` names the connector a rule reads. Dropping the `has()` guard makes a missing field a CEL error, which is already `unknown`. The displayed level climbs only through levels that have at least one known rule, and every known rule at that level must pass or be waived. An entity whose every rule is unknown stays `none` and drops out of the repository rollup, the same way D86 already drops an all-unknown constituent.

**`--ci` and `--update` refuse an unknown whose connector was not skipped.** A rule whose `dataFrom` connector is in `--skip` is an expected unknown: the fast ratchet skips `runner` and `github-actions`, so `tests-pass`, `clippy-clean` and the run-history rules are recorded and not compared. A zizmor or actionlint rule is `dataFrom: ci-lint`. CI does not skip `ci-lint`, so a missing linter still fails `--ci` (D70).

**The hosted report leads with the constituents that hold the repository.** `repoRollup` is in the report JSON. Overview opens on those scorecards, worst first. The none-count sort of every scorecard stays below that.

**The ratchet verdict is the gate for scores.** The pages workflow generates `site/public/report/` for each commit and does not commit it (D01M7YAXH89ZJQ7Q4HP77NMHBX8). This change does not bring that directory back, and it does not byte-compare it.

**`toolbox assess check` exits 1 when a recorded assessment is stale.** A missing assessment stays non-fatal. The subject hash is still the freshness check (D79).

**Where:** [design/github-actions-standards.md](../design/github-actions-standards.md), [design/repo-value.md](../design/repo-value.md), [design/site-standards.md](../design/site-standards.md)
