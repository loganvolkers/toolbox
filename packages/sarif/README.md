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
| Tool pin | `<tool>` at `<version>` (`toolbox doctor`; `toolbox tools install` runs mise, uv, pipx or pip) |
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
      install: {pypi: ruff, mise: "pipx:ruff", aqua: astral-sh/ruff}
      command: [ruff, check, --no-cache, --output-format, sarif, --exit-zero, "{files}"]
      files: [scripts/*.py]
```

ruff's rules and ignores stay in `ruff.toml`, which ruff reads with or without Toolbox. Include the
package again with another `name` and `kind` for a second tool, or for a second run of the same
tool (another semgrep rule set). Two includes of one tool declare the same pin: give both the same
`version`, `install` and `versionArgs`, and they merge into one `tools:` entry. Different values are
a `toolbox validate` error naming the field.

| Tool | `command` | Checked here |
|---|---|---|
| ruff | `[ruff, check, --no-cache, --output-format, sarif, --exit-zero, "{files}"]` | Yes (0.14.0) |
| semgrep | `[semgrep, --config, <rules>, --metrics=off, --disable-version-check, --sarif, --quiet, .]` | Yes (1.163.0); see also [`semgrep`](../semgrep/README.md) |
| zizmor | `[zizmor, --offline, --no-exit-codes, --no-ignores, --format, sarif, .github/]` | Yes (1.32.0); see also [`github-actions`](../github-actions/README.md). Over the directory, so composite actions and `dependabot.yml` are covered; `--no-ignores` because zizmor drops ignored findings from its SARIF |
| osv-scanner | `[osv-scanner, scan, source, --offline, --format, sarif, --lockfile, "{files}"]` | Yes (2.7.1), in Toolbox for Toolbox as `maintainer-advisories` against a pinned RustSec snapshot (D01M885GT32STM506V32XKPP37G); ignores live in `osv-scanner.toml`, which drops them from the SARIF |
| gitleaks, ESLint with a SARIF formatter, Checkov, trivy | Their SARIF options | Not yet |

## Parameters

| Parameter | Default | Meaning |
|---|---|---|
| `name`, `kind` | required | The connector, scorecard and campaign name; the entity kind (PascalCase) |
| `tool`, `version` | required | The program and the version the standard was made with |
| `command` | required | The tool's command printing SARIF to stdout. `{files}` becomes the files matching `files` |
| `files` | `[]` | Globs of the files covered; each gets a record, clean ones included. Without it, only files with findings have records |
| `countSuppressed` | `false` | Count findings the tool's SARIF marks as suppressed (semgrep's `nosemgrep`) as violations, so a new suppression fails and the campaign pays them down too. It only sees what the SARIF carries: zizmor removes `# zizmor: ignore[...]` findings entirely, so a new ignore reads as an improvement unless the command passes `--no-ignores` |
| `install`, `versionArgs` | `{}`, `[--version]` | The tool's id per installer (`mise`, `aqua`, `pypi`) for `toolbox tools install`, and how `toolbox doctor` asks its version |
| `title`, `owner`, `fix`, `root` | | The scorecard's heading, owner and fix text; where the tool runs |

## Sources

- [docs/reference/runner-connector.md](../../docs/reference/runner-connector.md): the SARIF slot and the record fields
- [docs/research-log.md](../../docs/research-log.md), *The box*: ruff 0.14.0, semgrep 1.163.0 and zizmor 1.32.0 SARIF, checked
