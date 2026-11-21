# Workspace Connector Reference

**`builtin:workspace` reads a local checkout, without building it or using the network, and
reports one record per matching file, directory or fact.** A portal turns records into entities
with a kind's `map`. This page lists its configuration and its records' fields. For a standard
over files, start from a package that configures it for you:
[`file-size`](../../packages/file-size/README.md) (lines per file) or
[`pattern-count`](../../packages/pattern-count/README.md) (matches of a regex per file).

```yaml
connectors:
  - name: workspace
    package: builtin:workspace
    runtime: in-image
    config:
      root: .
      crates: false
      files: ["src/**/*.py"]
      exclude: ["**/tests/**"]
      counts: {orm: '\bPerson\.objects\b'}
    records: [file]
```

## Configuration

Every key is optional. Globs are over `/`-separated paths relative to `root`: `**` spans
directories, `*` and `?` don't. `target`, `node_modules`, `.git`, `dist` and `.venv` are never
walked.

| Key | Default | What it does |
|---|---|---|
| `root` | `.` | The checkout, relative to where the command runs |
| `exclude` | `[]` | Globs removed from every walk, whatever else matches them |
| `lastChange` | `true` | Each file's last commit date, from one `git log` over the history. `false` skips it (records have `lastChange: null`); large checkouts are much faster without it |
| `files` | `[]` | Globs: one `file` record per match |
| `docs` | `[]` | Globs: one `doc` record per Markdown match |
| `counts` | `{}` | `{name: regex}`: on `file` and `doc` records, matches per regex (`counts.<name>`); on `file` records, the line of each match (`matchLines.<name>`) |
| `captures` | `{}` | `{name: regex}`: each regex's first group, unique and sorted (`captures.<name>`) |
| `crates` | `true` | One `crate` record per Cargo workspace member (runs `cargo metadata`; set `false` outside Rust) |
| `repo` | `false` | One `repo` record for the checkout: README, LICENSE, `.github` files, CODEOWNERS rules |
| `workflows` | `[]` | Globs: `workflow` records for GitHub Actions workflows, and `action-ref` records for the actions they use |
| `betterer` | `[]` | Globs: one `betterer` record per test in each Betterer results file |
| `skills`, `connectors`, `agentDocs`, `plugins`, `packages`, `schemas`, `examples`, `pages`, `assessments` | `[]` | Globs for Toolbox's own standards (`skill`, `connector`, `agent-doc`, `plugin`, `package-json`, `json-schema`, `example`, `site-page`, `assessment` records) |
| `dependencies` | `false` | One `dependency` record per direct external Cargo dependency |
| `jsWorkspace` | `false` | One `js-workspace` record: pnpm or npm workspace, lockfile, `packageManager` pin |
| `expoApps`, `storybooks` | `[]` | Globs: `expo-app` and `storybook` records ([packages/pnpm-expo](../../packages/pnpm-expo/), [packages/storybook](../../packages/storybook/)) |
| `data` | `[]` | `[{kind, globs, paths?}]`: entries of YAML or JSON files as records of `kind`, each with a string `id` |

## Records

The external id is `<kind>:<path>` for per-file records.

| Record | Fields |
|---|---|
| `file` | `path`, `extension`, `lines`, `counts`, `matchLines`, `captures`, `lastChange` |
| `doc` | `path`, `title`, `words`, `links`, `brokenLinks`, `codeSpans`, `hasStatus`, `hasSources`, `missingDecisions`, `counts`, `captures`, `lastChange` |
| `repo` | README, LICENSE, CODEOWNERS and `.github` facts ([repo-portals.md](../design/repo-portals.md)) |
| `workflow`, `action-ref` | Triggers, permissions, timeouts, `uses:` and how each is pinned ([github-actions-standards.md](../design/github-actions-standards.md)) |
| Others | Described in the design docs that use them (`toolbox describe portal .toolbox` lists a portal's kinds and fields) |

`lines` counts lines as `wc -l` would, plus a last line without a newline. `matchLines` is 1-based.

## Checking a configuration

```bash
toolbox connector test --name workspace --portal .toolbox -- toolbox-connector-workspace   # the protocol
toolbox report .toolbox -o failures                   # what the portal's rules make of the records
```

## Sources

- [crates/toolbox-connector-workspace/src/main.rs](../../crates/toolbox-connector-workspace/src/main.rs): the configuration and records, in its module comment
- [docs/design/toolbox-for-toolbox.md](../design/toolbox-for-toolbox.md) §3: why the connector reads files statically
