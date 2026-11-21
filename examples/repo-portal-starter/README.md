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

**JS / Expo monorepos.** Do **not** turn on `packages: ["**/package.json"]` (or expect
`maintainer-package`) until you have **pnpm-aware** rules. That scorecard is per-manifest and
npm-oriented; a root `pnpm-lock.yaml` alone does not give you workspace / Expo hygiene. Include
[`packages/pnpm-expo`](../../packages/pnpm-expo/) instead (enable `jsWorkspace: true` and `expoApps`
globs on `builtin:workspace`). See that package’s README for the consumer diff and minimum Toolbox
version.

**Fetch before measuring.** If the remote default branch already has `.toolbox/` but your local
checkout does not (shallow clone, cloud VM, sparse checkout), `toolbox report` scores the wrong
tree. `git fetch` / pull the portal files (and any committed `results.json` / `toolbox.lock`) before
you measure or ratchet.

## A standard of your own

Most teams already have standards written down (in `AGENTS.md`, a style guide, a review checklist)
that nothing enforces. Three packages turn the common shapes into a ratcheted standard with a
campaign in one `include`, with no script to maintain:

| Written standard | Package | Example |
|---|---|---|
| "Keep files under N lines" | [`file-size`](../../packages/file-size/README.md) | Codex: no Rust module over 800 lines |
| "Never use X; use Y instead" | [`pattern-count`](../../packages/pattern-count/README.md) | PostHog: no `Person.objects` outside the routed helpers |
| Already a semgrep rule, with `nosemgrep` suppressions | [`semgrep`](../../packages/semgrep/README.md) | PostHog's person-table rule: 104 suppressed findings to pay down |

```yaml
include:
  - package: github:loganvolkers/toolbox/packages/pattern-count@<tag>
    with:
      name: no-direct-orm
      kind: PythonFile
      files: ["**/*.py"]
      exclude: ["**/tests/**"]
      exempt: {app/models/util.py: the helpers themselves}
      pattern: '\bPerson\.objects\b'
      fix: Use a helper from app/models/util.py.
```

The loop is the same for every standard:

```bash
toolbox report .toolbox -o failures                                      # what violates, where, and the fix (JSON)
toolbox report .toolbox --baseline .toolbox/results.json --update       # record today's state; commit it
toolbox report .toolbox --baseline .toolbox/results.json                # the gate: exit 1 when anything got worse
toolbox report .toolbox --baseline .toolbox/results.json -o json | jq '.campaigns'   # progress since the start
toolbox report .toolbox --baseline .toolbox/results.json --update --accept no-direct-orm   # after tightening the standard
```

For a standard none of them fits, write the kind and scorecard yourself: the workspace connector's
records are listed in [docs/reference/workspace-connector.md](../../docs/reference/workspace-connector.md),
and a rule's `measure` (a number, compared by the ratchet) is described in
[ratchets.md](../../docs/design/ratchets.md) §3.

`repo-hygiene` is stage 1 of the CODEOWNERS graduation example in
[federation.md](../../docs/design/federation.md) §6: a standard that starts in one team's repositories.

```bash
toolbox report examples/repo-portal-starter            # from a repository root: score that checkout
toolbox test examples/repo-portal-starter              # the rules' own tests over fixtures/ (4 made-up repos)
```

In a Rust workspace, set `crates: true` in the connector config for per-crate metrics, and borrow
scorecards from [portal/toolbox.yaml](../../portal/toolbox.yaml) (Toolbox for Toolbox).
