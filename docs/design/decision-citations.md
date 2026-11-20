# Decision Citations: A Cited Id Must Be a Row

*Status: rule built, 2026-11-20 (D83). `maintainer-docs` gains `cited-decisions-exist`. The decision
log stays the index and stays skipped. Related:
[toolbox-for-toolbox.md](./toolbox-for-toolbox.md),
[scorecards-for-scorecards.md](./scorecards-for-scorecards.md).*

---

## 1. Conclusion

**A document that names a decision must name one the log contains.**

The maintainer SDLC treats `docs/decisions.md` as append-only. A citation is the token `D` plus
digits. The check is a set comparison, not a reading of the prose:

| Question | What answers it | In this repo |
|---|---|---|
| Does every cited id have a row? | `\bD\d+\b` against the `| Dnn |` rows of `docs/decisions.md` | bronze |

The log itself is in `policy.historicalDocs`, so it is not scored for citing itself. Adding a row
and citing it belong in the same commit.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **This repository's decision log** | Each decision is one table row whose first cell is the id | The row is the existence check. Rewriting a past row is still forbidden |
| **The docs scorecard** | Bronze already requires resolving links and naming the SDLC layer | A missing id is the same class of defect: a pointer that goes nowhere |

## 3. The rule

`cited-decisions-exist` passes when `missingDecisions` is empty. The evidence is the missing ids.
The fix is to cite a row that exists, or to add the row.

Fixtures: one document cites an id that is not a row and fails; `doc:docs/dashboards.md` cites
nothing missing and passes (D71).

## Sources

- This repository's `docs/decisions.md`.
- [toolbox-for-toolbox.md](./toolbox-for-toolbox.md), the docs scorecard this rule extends.
