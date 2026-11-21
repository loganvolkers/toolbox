# Repo portal starter

A minimal Toolbox portal for one repository ([docs/design/repo-portals.md](../../docs/design/repo-portals.md)).
Copy `toolbox.yaml` to `.toolbox/toolbox.yaml` and add the
[report action](../../actions/report/README.md): the repository is then scored on every push, and a
pull request fails when anything gets worse than the committed `.toolbox/results.json`.

| Scorecard | Kind | Bronze | Silver |
|---|---|---|---|
| `repo-hygiene` | Repository | README of 50+ words; a CODEOWNERS file | CODEOWNERS starts with a `*` rule; every owner is a team (`@org/team`) **or** only personal handles (`@user`); a LICENSE |
| `docs` | Doc (`*.md`, `docs/**/*.md`) | Relative links resolve | — |

Failing rules carry their evidence: for example, individuals mixed into a team CODEOWNERS, or the
broken links.

**Personal repositories.** Silver does not require a GitHub team. A catch-all `* @you` plus a LICENSE
is enough. Mixing `@org/team` with `@user` still fails silver — keep owners all-teams or all-personal
([D01M7Y57FVX0XYNGMX0A0XJ4GK9](../../docs/decisions/D01M7Y57FVX0XYNGMX0A0XJ4GK9.md)).

**JS/TS repositories.** Ignore `.toolbox/**` in ESLint (and similar linters). The portal’s
`tests/*.yaml` fixtures are not JavaScript; linting them fails with parse errors such as
“Invalid character”.

`repo-hygiene` is stage 1 of the CODEOWNERS graduation example in
[federation.md](../../docs/design/federation.md) §6: a standard that starts in one team's repositories.

```bash
toolbox report examples/repo-portal-starter            # from a repository root: score that checkout
toolbox test examples/repo-portal-starter              # the rules' own tests over fixtures/ (4 made-up repos)
```

In a Rust workspace, set `crates: true` in the connector config for per-crate metrics, and borrow
scorecards from [portal/toolbox.yaml](../../portal/toolbox.yaml) (Toolbox for Toolbox).
