# Repo portal starter

A minimal Seikika portal for one repository ([docs/design/repo-portals.md](../../docs/design/repo-portals.md)).
Copy `seikika.yaml` to `.seikika/seikika.yaml` and add the
[report action](../../actions/report/README.md): the repository is then scored on every push, and a
pull request fails when anything gets worse than the committed `.seikika/results.json`.

| Scorecard | Kind | Bronze | Silver |
|---|---|---|---|
| `repo-hygiene` | Repository | README of 50+ words; a CODEOWNERS file | CODEOWNERS starts with a `*` rule; every owner is a team (`@org/team`) **or** only personal handles (`@user`); a LICENSE |
| `docs` | Doc (`*.md`, `docs/**/*.md`) | Relative links resolve | — |

Failing rules carry their evidence: for example, individuals mixed into a team CODEOWNERS, or the
broken links.

**Personal repositories.** Silver does not require a GitHub team. A catch-all `* @you` plus a LICENSE
is enough. Mixing `@org/team` with `@user` still fails silver — keep owners all-teams or all-personal
([D01M7Y57FVX0XYNGMX0A0XJ4GK9](../../docs/decisions/D01M7Y57FVX0XYNGMX0A0XJ4GK9.md)).

**JS/TS repositories.** Ignore `.seikika/**` in ESLint (and similar linters). The portal’s
`tests/*.yaml` fixtures are not JavaScript; linting them fails with parse errors such as
“Invalid character”.

**JS / Expo monorepos.** Do **not** turn on `packages: ["**/package.json"]` (or expect
`maintainer-package`) until you have **pnpm-aware** rules. That scorecard is per-manifest and
npm-oriented; a root `pnpm-lock.yaml` alone does not give you workspace / Expo hygiene. Include
[`packages/pnpm-expo`](../../packages/pnpm-expo/) instead (enable `jsWorkspace: true` and `expoApps`
globs on `builtin:workspace`). See that package’s README for the consumer diff and minimum Seikika
version.

**Fetch before measuring.** If the remote default branch already has `.seikika/` but your local
checkout does not (shallow clone, cloud VM, sparse checkout), `seikika report` scores the wrong
tree. `git fetch` / pull the portal files (and any committed `results.json` / `seikika.lock`) before
you measure or ratchet.

## A standard of your own

Most teams already have standards written down (in `AGENTS.md`, a style guide, a review checklist)
that nothing enforces. Three packages turn the common shapes into a ratcheted standard with a
campaign in one `include`, with no script to maintain:

| Written standard | Package | Example |
|---|---|---|
| Already checked by a linter with SARIF output (ruff, semgrep, zizmor, gitleaks) | [`sarif`](../../packages/sarif/README.md): the tool in the box, its config in its own file | No ruff findings in `scripts/` |
| "Keep files under N lines" | [`file-size`](../../packages/file-size/README.md) | No component over 400 lines |
| "Never use X; use Y instead" | [`pattern-count`](../../packages/pattern-count/README.md) | No `console.log` outside the logger |
| Already a semgrep rule, with `nosemgrep` suppressions | [`semgrep`](../../packages/semgrep/README.md) | Pay the suppressions down to zero |

```yaml
include:
  - package: github:seikika/seikika/packages/pattern-count@<tag>
    with:
      name: no-console
      kind: SourceFile
      files: ["web/src/**/*.ts"]
      exclude: ["**/*.test.ts"]
      exempt: {web/src/logger.ts: the logger itself}
      pattern: '\bconsole\.log\('
      fix: Use the logger from web/src/logger.ts.
```

The loop is the same for every standard:

```bash
seikika vendor <seikika checkout>/packages/pattern-count --portal .seikika   # offline: copy the package in
seikika report .seikika -o failures                                      # what violates, where, and the fix (JSON)
seikika report .seikika --baseline .seikika/results.json --update       # record today's state; commit it
seikika report .seikika --baseline .seikika/results.json                # the gate: exit 1 when anything got worse
seikika report .seikika --against origin/main                           # what this branch changed
seikika campaign --portal .seikika --baseline .seikika/results.json     # progress since the start
seikika report .seikika --baseline .seikika/results.json --update --accept no-console   # after tightening the standard
seikika report .seikika -o standard --out .seikika/STANDARD.md          # the docs and a CI workflow, from the manifest
seikika connector records --portal .seikika --name no-console           # what the tool reported, and the command to rerun it
```

**Box or tool:** the starter and these commands are the box. The tools are what the box carries:
an existing linter through the SARIF slot (`parse: sarif` on the runner connector) whenever there
is one, and a jig from the packages above only when there isn't
([seikika-and-tools.md](../../docs/design/seikika-and-tools.md)).

For a standard none of them fits, write the kind and scorecard yourself: the workspace connector's
records are listed in [docs/reference/workspace-connector.md](../../docs/reference/workspace-connector.md),
and a rule's `measure` (a number, compared by the ratchet) is described in
[ratchets.md](../../docs/design/ratchets.md) §3.

`repo-hygiene` is stage 1 of the CODEOWNERS graduation example in
[federation.md](../../docs/design/federation.md) §6: a standard that starts in one team's repositories.

```bash
seikika report examples/repo-portal-starter            # from a repository root: score that checkout
seikika test examples/repo-portal-starter              # the rules' own tests over fixtures/ (4 made-up repos)
```

In a Rust workspace, set `crates: true` in the connector config for per-crate metrics, and borrow
scorecards from [portal/seikika.yaml](../../portal/seikika.yaml) (Seikika for Seikika).
