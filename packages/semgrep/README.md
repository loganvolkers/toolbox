# semgrep: a Toolbox package

**A repository's existing semgrep rule as a ratcheted standard, with its `nosemgrep`
suppressions as the baseline a campaign pays down.** semgrep's own `--error` gate stops
unsuppressed findings, but a new `# nosemgrep` comment passes it. This package counts suppressed
findings too, so:

- the ratchet fails a pull request that adds a finding to a file, suppressed or not;
- the campaign `<name>-zero` pays the findings (suppressions included) down to zero;
- `toolbox report` lists each finding with its line, its rule, and whether it is suppressed.

On PostHog, `.semgrep/rules/no-direct-persons-db-orm.yaml` reports 104 findings in 33 files at
the commit the A/B used, all suppressed: a campaign the repository doesn't have yet
([experiments/standards-ab](../../experiments/standards-ab/README.md)).

| Brings | Named |
|---|---|
| Connector | `<name>` (`builtin:runner`, one semgrep run, `parse: semgrep`) |
| Kind | `<kind>` (default `SemgrepFile`): `path`, `total`, `suppressed`, `active`, `findings` |
| Scorecard | `<name>`, one level `clean`, rule `no-findings` |
| Campaign | `<name>-zero` |
| Tool | semgrep on the `PATH` (optional: without it the rule is `unknown`, not failing) |

## Use it

```yaml
apiVersion: toolbox.dev/v1alpha1
portal: {name: my-repo, owner: group:maintainers, toolbox: "0.1"}
include:
  - package: github:loganvolkers/toolbox/packages/semgrep@<tag>   # or a relative path
    with:
      name: persons-orm
      kind: PythonFile
      config: .semgrep/rules/no-direct-persons-db-orm.yaml
```

```bash
toolbox report .toolbox -o failures                                   # every finding, as JSON
toolbox report .toolbox --baseline .toolbox/results.json --update    # record the baseline; commit it
toolbox report .toolbox --baseline .toolbox/results.json             # the gate
```

The rule's own `paths` (include and exclude) decide what is scanned. semgrep runs with
`--disable-nosem`, so suppressed findings are reported. semgrep's JSON doesn't mark them, so a
finding counts as suppressed when its line, or the line above, has a `nosemgrep` comment. A
whole-repository run takes as long as semgrep does (minutes on PostHog), so run it where the
rule's semgrep job already runs.

## Parameters

| Parameter | Default | Meaning |
|---|---|---|
| `config` | required | semgrep's `--config`: a rule file or directory |
| `target` | `.` | What semgrep scans |
| `name` | `semgrep` | Names the connector, scorecard and campaign |
| `kind` | `SemgrepFile` | The kind (PascalCase) |
| `title`, `owner`, `fix` | | The scorecard's heading, owner and fix text |
| `root` | `.` | Where semgrep runs, relative to where `toolbox report` runs |

## Sources

- [crates/toolbox-connector-runner/src/lint.rs](../../crates/toolbox-connector-runner/src/lint.rs): the `semgrep` parser, checked against semgrep 1.163.0's JSON (docs/research-log.md)
- [docs/design/ratchets.md](../../docs/design/ratchets.md): the ratchet, measures and campaign starts
