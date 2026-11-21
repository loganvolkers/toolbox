# Runner Connector Reference

**`builtin:runner` puts tools in the box: it runs local commands and turns their output into
records.** This page is about the box ([toolbox-and-tools.md](../design/toolbox-and-tools.md)):
- **Slots** read standard output any tool can produce.
- **Fittings** read one tool's own format.
- Every lint record carries the command that reproduces it with the tool alone.

To add a tool, prefer the SARIF slot: most linters and security scanners can print SARIF.

```yaml
connectors:
  - name: lint
    package: builtin:runner
    runtime: in-image
    config:
      name: lint
      root: .
      jobs:
        - name: zizmor
          command: [zizmor, --offline, --no-exit-codes, --format, sarif, "{files}"]
          parse: sarif
          files: [.github/workflows/*.y*ml]
          optional: true
    records: [zizmor]
```

## Jobs

| Key | Meaning |
|---|---|
| `name` | The job, and the record kind it produces (`<name>:<path>` ids for lint jobs) |
| `command` | The argv. A word that is exactly `{files}` becomes the files matching `files` that exist; with none, the job doesn't run. `{crate}` is each Cargo workspace member in `per: crate` jobs |
| `parse` | How the output is read (below) |
| `files` | Lint jobs: globs of the files the job covers. Each gets a record, clean ones included |
| `output` | Lint jobs: read the report from this file (relative to `root`) instead of stdout, for tools that write one |
| `optional` | A missing program skips the job with a log line; rules over its records are `unknown`, not failing |
| `per` | `crate` (one run per workspace member) or `once` (default) |
| `kinds` | `ndjson` jobs: the record kinds the command prints |

## Slots and fittings

| `parse` | Kind | Reads |
|---|---|---|
| `sarif` | **Slot** | SARIF 2.1.0, from any tool: ruff, semgrep, zizmor, gitleaks, an ESLint SARIF formatter. Only semgrep and zizmor are checked here (docs/research-log.md) |
| `ndjson` | **Slot** | Connector `record` messages a tool prints itself (`toolbox describe` does) |
| `zizmor`, `actionlint`, `semgrep` | Fitting | Those tools' own JSON. Prefer `sarif` where the tool has it; actionlint has no SARIF output |
| `cargo-test`, `clippy` | Fitting | `cargo test` and `cargo clippy` output, per crate |
| `tally` | Fitting | An NDJSON file the command writes, counted by a key |

**What a lint record holds**, for every file (the shape is the same whichever slot or fitting read
it):

| Field | Meaning |
|---|---|
| `path` | The file, relative to `root` |
| `total`, `suppressed`, `active` | Findings, those suppressed inline (SARIF `suppressions`, such as `nosemgrep`), and the rest |
| `counts` | Findings per severity: the tool's own `…severity` result property when it has one (zizmor's `zizmor/severity`: `high`, `medium`, `low`, `informational`), else the SARIF level (`error`, `warning`, `note`) |
| `findings` | `{rule, ruleId, level, severity, line, message, suppressed, properties}`; `rule` is the short name (after the last `/` or `.` of `ruleId`) |
| `command` | The command that reproduces this file's findings with the tool alone (take-it-out) |
| `via` | The slot or fitting that read it |

SARIF lists files without results only when the tool fills `artifacts`. semgrep doesn't, so a file
that loses its last finding drops out of the records. The ratchet counts that as fixed for
scorecards whose every entity must pass.

## Checking that the box doesn't change what a tool says

```bash
toolbox connector records --portal portal --name ci-lint    # the records, with each `command`
python3 scripts/take-it-out.py portal ci-lint               # rerun each command alone and compare
```

`take-it-out.py` counts each file's findings from the tool's own output, independently of
Toolbox's parsers, and fails on any difference. CI runs it for Toolbox for Toolbox.

## Sources

- [crates/toolbox-connector-runner/src/](../../crates/toolbox-connector-runner/src/): the jobs (`main.rs`), and the slot and fittings (`lint.rs`)
- [docs/research-log.md](../research-log.md), *Box and tools*: zizmor 1.32.0 and semgrep 1.163.0 SARIF, checked
