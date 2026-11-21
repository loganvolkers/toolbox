# sarif: a Toolbox package

**Box or tool:** this package is the box's **SARIF slot as one include**
([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md)). It carries an existing tool,
any tool that prints SARIF 2.1.0, and gives it:
- a home: a connector, a record per file, and the tool pinned under `tools:` for `toolbox doctor`;
- a ratchet: a new finding, or a new file with findings, fails a pull request;
- a campaign to zero, counted from the start the baseline records.

The tool keeps its own configuration file and runs exactly as it would alone. Every record's
`command` reruns that file's check with the tool by itself, and `scripts/take-it-out.py` checks
that the counts match.

| Brings | Named |
|---|---|
| Connector | `<name>` (`builtin:runner`, `parse: sarif`) |
| Tool pin | `<tool>` at `<version>` (`toolbox doctor`, `toolbox doctor --mise`) |
| Kind | `<kind>`: `path`, `total`, `active`, `findings`, `command` |
| Scorecard | `<name>`, one level `clean`, rule `no-findings` |
| Campaign | `<name>-zero` |

## Use it

Toolbox for Toolbox carries ruff this way ([portal/toolbox.yaml](../../portal/toolbox.yaml)):

```yaml
include:
  - package: github:loganvolkers/toolbox/packages/sarif@<tag>   # or a path, or `toolbox vendor`
    with:
      name: maintainer-python
      kind: PythonScript
      tool: ruff
      version: 0.14.0
      mise: pipx:ruff
      command: [ruff, check, --no-cache, --output-format, sarif, --exit-zero, "{files}"]
      files: [scripts/*.py]
```

ruff's rules and ignores stay in `ruff.toml`, which ruff reads with or without Toolbox. Include the
package again with another `name` and `kind` for a second tool.

| Tool | `command` | Checked here |
|---|---|---|
| ruff | `[ruff, check, --no-cache, --output-format, sarif, --exit-zero, "{files}"]` | Yes (0.14.0) |
| semgrep | `[semgrep, --config, <rules>, --metrics=off, --sarif, --quiet, .]` | Yes (1.163.0); see also [`semgrep`](../semgrep/README.md) |
| zizmor | `[zizmor, --offline, --no-exit-codes, --format, sarif, "{files}"]` | Yes (1.32.0); see also [`github-actions`](../github-actions/README.md) |
| gitleaks, ESLint with a SARIF formatter, Checkov, trivy | Their SARIF options | Not yet |

## Parameters

| Parameter | Default | Meaning |
|---|---|---|
| `name`, `kind` | required | The connector, scorecard and campaign name; the entity kind (PascalCase) |
| `tool`, `version` | required | The program and the version the standard was made with |
| `command` | required | The tool's command printing SARIF to stdout. `{files}` becomes the files matching `files` |
| `files` | `[]` | Globs of the files covered; each gets a record, clean ones included. Without it, only files with findings have records |
| `countSuppressed` | `false` | Count inline suppressions (`noqa`, `nosemgrep`) as violations, so the campaign pays them down too |
| `mise`, `versionArgs` | `""`, `[--version]` | How mise installs the tool, and how `toolbox doctor` asks its version |
| `title`, `owner`, `fix`, `root` | | The scorecard's heading, owner and fix text; where the tool runs |

## Sources

- [docs/reference/runner-connector.md](../../docs/reference/runner-connector.md): the SARIF slot and the record fields
- [docs/research-log.md](../../docs/research-log.md), *The box*: ruff 0.14.0, semgrep 1.163.0 and zizmor 1.32.0 SARIF, checked
