# Decision Citations: A Cited Id Must Be a File

*Status: amended 2026-11-20 (D84, D01M7XKEXQH4HG5371629M50E2G). `maintainer-docs` gains
`cited-decisions-exist`. The decision files stay the index and stay skipped. This is the maintainer
SDLC. Related: [decision-files.md](./decision-files.md),
[toolbox-for-toolbox.md](./toolbox-for-toolbox.md).*

---

## 1. Conclusion

**A document that names a decision must name one the log contains.**

A citation is either a legacy id (`D` plus one to four digits) or a ULID id (`D` plus 26 Crockford
characters). The check is a set comparison against `id` and `legacy` in `docs/decisions/*.md`, not a
reading of the prose:

| Question | What answers it | In this repo |
|---|---|---|
| Does every cited id have a file? | The citation tokens against each file's frontmatter | bronze |

The log directory is not scored for citing itself. Adding a file and citing it belong in the same commit.

D84 checked `| Dnn |` rows of one table. The table was the file two agents had to edit. The id and the place it lives changed; the check did not: a citation that names nothing is still a bronze failure.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **This repository's decision files** | Each decision is one file. A backfilled file also records its old number | The frontmatter is the existence check. Rewriting a past file is still forbidden |
| **The docs scorecard** | Bronze already requires resolving links and naming the SDLC layer | A missing id is the same class of defect: a pointer that goes nowhere |

## 3. The rule

`cited-decisions-exist` passes when `missingDecisions` is empty. The evidence is the missing ids.
The fix is to cite a file that exists, or to add one.

Fixtures: one document cites an id that is not a file and fails; `doc:docs/dashboards.md` cites
nothing missing and passes (D71).

## Sources

- [decision-files.md](./decision-files.md), the id and the directory.
- [toolbox-for-toolbox.md](./toolbox-for-toolbox.md), the docs scorecard this rule extends.
