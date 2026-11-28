# Workspace Connector Reference

**`builtin:workspace` reads a local checkout, without building it or using the network, and
reports one record per matching file, directory or fact.** The `git` mode clones the child
repositories a parent portal declares. The `githubOrg` mode reads GitHub repository facts through
the API without a checkout. A portal turns records into entities with a kind's `map`. This page lists its configuration and its records' fields. For a standard
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
      files: ["web/src/**/*.ts"]
      exclude: ["**/tests/**"]
      counts: {debug: '\bconsole\.log\('}
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
| `skills`, `connectors`, `agentDocs`, `plugins`, `packages`, `schemas`, `examples`, `pages`, `assessments` | `[]` | Globs for Seikika's own standards (`skill`, `connector`, `agent-doc`, `plugin`, `package-json`, `json-schema`, `example`, `site-page`, `assessment` records) |
| `assessmentManifest` | absent | Portal path relative to `root`. Validates assessment files and checks whether each one covers the current rubric and records its hash. |
| `standardPackages` | `[]` | Globs for Seikika package manifests, usually `packages/*/seikika-package.yaml` |
| `dependencies` | `false` | One `dependency` record per direct external Cargo dependency |
| `jsWorkspace` | `false` | One `js-workspace` record: pnpm or npm workspace, lockfile, `packageManager` pin |
| `expoApps`, `storybooks` | `[]` | Globs: `expo-app` and `storybook` records ([packages/pnpm-expo](../../packages/pnpm-expo/), [packages/storybook](../../packages/storybook/)) |
| `portals` | `[]` | Globs, usually `*/.seikika/seikika.yaml`: one `portal` record per child portal found in a checkout |
| `git` | `[]` | `[{key, url, branch}]`, written by `federation.children`: each repository is cloned into `<root>/<key>`, kept at its branch, and read as one `portal` record |
| `githubOrg` | unset | `{organization, publicOnly?, workflows?, reports?}` switches to `federated-repo` records. Enable `workflows` for offline `zizmor` audits and `reports` to read published child snapshots. Optional flags default to `false` |
| `data` | `[]` | `[{kind, globs, paths?, hashPaths?}]`: entries of YAML or JSON files as records of `kind`, each with a string `id` |

For a `data` source, `paths` lists fields whose repository-relative values
are checked for existence. `hashPaths` lists fields holding files to include
in the record's content hash. These fields accept a path or a list of paths.
A missing or unreadable file contributes a distinct marker. Targets outside
the checkout and directories are treated as unreadable. Without `hashPaths`,
the hash covers the declaration as before.

## Records

The external id is `<kind>:<path>` for per-file records.

| Record | Fields |
|---|---|
| `file` | `path`, `extension`, `lines`, `counts`, `matchLines`, `captures`, `lastChange` |
| `doc` | `path`, `title`, `words`, `links`, `brokenLinks`, `codeSpans`, `hasStatus`, `hasSources`, `missingDecisions`, `counts`, `captures`, `lastChange` |
| `repo` | README, LICENSE, CODEOWNERS and `.github` facts ([repo-portals.md](../design/repo-portals.md)) |
| `federated-repo` | GitHub slug, provider ID and visibility, default branch and commit, child manifest presence, README, LICENSE and CODEOWNERS facts, and any acquisition error. With `reports`, includes `report` or `reportError`. The external ID is `federated-repo:<owner>/<name>`. [Organization federation example](../../examples/github-org-federation/README.md) |
| `federated-workflow` | `repository`, `visibility`, `path`, `commit`, `sourceUrl`, `audited`, `findings`, and `auditError`. The external ID is `federated-workflow:<owner>/<name>/<path>` |
| `portal` | `key`, `name`, `title`, `owner`, `path`, `dir`, `file`, `actions`, `runner`, `acceptApprovalsFrom`, `scorecards`, `packages`, `packageError`, `error`; `report` or `reportError` for a child publication; `git` for a cloned child; `warning` when two share a name ([federated-actions.md](../design/federated-actions.md) §5.2) |
| `workflow`, `action-ref` | Triggers, permissions, timeouts, `uses:` and how each is pinned ([github-actions-standards.md](../design/github-actions-standards.md)) |
| `connector` | Package, protocol, test, and public guide facts used by the first-party connector scorecard ([connector standards](../design/connector-standards.md#3-the-catalog)) |
| `standard-package` | Manifest `path`, `name`, `version`, and `publicGuide`. The guide route is null when the directory or guide is missing, a draft, or hidden from the sidebar. [Package directory decision](../decisions/D01M8CRS50G5086P5DCFQN636BA.md) |
| Others | Described in the design docs that use them (`seikika describe portal .seikika` lists a portal's kinds and fields) |

`lines` counts lines as `wc -l` would, plus a last line without a newline. `matchLines` is 1-based.

## Checking a configuration

```bash
seikika connector test --name workspace --portal .seikika -- seikika-connector-workspace   # the protocol
seikika report .seikika -o failures                   # what the portal's rules make of the records
```

## Sources

- [crates/seikika-connector-workspace/src/main.rs](../../crates/seikika-connector-workspace/src/main.rs): the configuration and records, in its module comment
- [docs/design/seikika-for-seikika.md](../design/seikika-for-seikika.md) §3: why the connector reads files statically
