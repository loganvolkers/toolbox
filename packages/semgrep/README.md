# semgrep: a Seikika package

**A repository's existing semgrep rule as a ratcheted standard, with its `nosemgrep`
suppressions as the baseline a campaign pays down.** semgrep's own `--error` gate stops
unsuppressed findings, but a new `# nosemgrep` comment passes it. This package counts suppressed
findings too, so:

- the ratchet fails a pull request that adds a finding to a file, suppressed or not;
- the campaign `<name>-zero` pays the findings (suppressions included) down to zero;
- `seikika report` lists each finding with its line, its rule, and whether it is suppressed.

On the repository the standards A/B used, the rule's findings were all suppressed inline: a
campaign the repository didn't have yet
([experiments/standards-ab](../../experiments/standards-ab/README.md)).

**Box or tool:** this package is a **kit**: it carries semgrep, an existing tool, with the rule
the repository already has ([seikika-and-tools.md](../../docs/design/seikika-and-tools.md)).
semgrep runs as it would without Seikika (`semgrep --config <rule> --sarif`), and its findings come
in through the box's standard **SARIF slot**, which reads semgrep's own suppression marks. Each
record's `command` repeats the run with semgrep alone.

| Brings | Named |
|---|---|
| Connector | `<name>` (`builtin:runner`, one semgrep run, `parse: sarif`) |
| Kind | `<kind>` (default `SemgrepFile`): `path`, `total`, `suppressed`, `active`, `findings` |
| Scorecard | `<name>`, one level `clean`, rule `no-findings` |
| Campaign | `<name>-zero` |
| Tool | semgrep on the `PATH` (optional: without it the rule is `unknown`, not failing) |

## Use it

```yaml
apiVersion: seikika.com/v1alpha1
portal: {name: my-repo, owner: group:maintainers, seikika: "0.1"}
include:
  - package: github:seikika/seikika/packages/semgrep@<tag>   # or a relative path
    with:
      name: raw-sql
      kind: PythonFile
      config: .semgrep/rules/no-raw-sql.yaml
```

```bash
seikika report .seikika -o failures                                   # every finding, as JSON
seikika report .seikika --baseline .seikika/results.json --update    # record the baseline; commit it
seikika report .seikika --baseline .seikika/results.json             # the gate
```

The rule's own `paths` (include and exclude) decide what is scanned. semgrep's SARIF reports
`nosemgrep` findings with `suppressions: [{kind: inSource}]`, so they count without any parsing of
comments. SARIF lists only files with findings, so a file that loses its last finding drops out,
and the ratchet counts that as fixed. A
whole-repository run takes as long as semgrep does. On a closed network semgrep's startup version
check stalls for minutes; the kit passes `--disable-version-check`, and so should any command you
run alone (on one large Python and TypeScript repository: 13 s with it, about 200 s without).

**One rule set, one target.** `config` and `target` are single values. For several rule files or
several paths, as CI often runs semgrep, use the [`sarif`](../sarif/README.md) kit with semgrep's
full command and `countSuppressed: true`. semgrep names a finding after its rule file's path
(`.semgrep/rules/x.yaml` gives `semgrep.rules.x`), so moving the file renames every finding the
baseline holds: move rule files with an `--update`.

## Parameters

| Parameter | Default | Meaning |
|---|---|---|
| `config` | required | semgrep's `--config`: a rule file or directory |
| `target` | `.` | What semgrep scans |
| `name` | `semgrep` | Names the connector, scorecard and campaign |
| `kind` | `SemgrepFile` | The kind (PascalCase) |
| `title`, `owner`, `fix` | | The scorecard's heading, owner and fix text |
| `root` | `.` | Where semgrep runs, relative to where `seikika report` runs |

## Sources

- [crates/seikika-connector-runner/src/lint.rs](../../crates/seikika-connector-runner/src/lint.rs): the SARIF slot, checked against semgrep 1.163.0's and zizmor 1.32.0's SARIF (docs/research-log.md)
- [docs/design/ratchets.md](../../docs/design/ratchets.md): the ratchet, measures and campaign starts
