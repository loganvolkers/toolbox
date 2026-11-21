# pattern-count: a Toolbox package

**Every match of a regular expression is a violation.** One `include` gives a repository the
standard, a ratchet and a campaign:

- `toolbox report` lists every file with matches, with the line of each and the fix;
- the ratchet fails a pull request that adds a match to any file (including a new file), and
  reports the files that lost matches;
- the campaign `<name>-zero` counts files with matches, and matches, from the day the baseline
  first records it.

It needs nothing but the `toolbox` binaries: files are read by `builtin:workspace`. First used for
PostHog's "never add new direct ORM queries like `Person.objects.filter(...)` … use the routed
helpers" ([experiments/standards-ab](../../experiments/standards-ab/README.md)). If the pattern is
already a semgrep rule, use [`packages/semgrep`](../semgrep/README.md) instead: it keeps the rule
in one place and counts its `nosemgrep` suppressions.

| Brings | Named |
|---|---|
| Connector | `<name>` (`builtin:workspace`, `file` records with `counts` and `matchLines`) |
| Kind | `<kind>` (default `PatternFile`): `path`, `matches`, `matchLines` |
| Scorecard | `<name>`, one level `clean`, rule `no-matches` |
| Campaign | `<name>-zero` |

## Use it

`.toolbox/toolbox.yaml`:

```yaml
apiVersion: toolbox.dev/v1alpha1
portal: {name: my-repo, owner: group:maintainers, toolbox: "0.1"}
include:
  - package: github:loganvolkers/toolbox/packages/pattern-count@<tag>   # or a relative path
    with:
      name: person-orm
      kind: PythonFile
      files: ["**/*.py"]
      exclude: ["**/tests/**", "**/test_*.py", "**/*_test.py", "**/conftest.py"]
      exempt:
        app/models/person/util.py: the routed helpers themselves
      pattern: '\b(Person|PersonDistinctId)\.objects\b'
      fix: Use a routed helper from app/models/person/util.py (or add one there).
```

Then, from the repository root:

```bash
toolbox report .toolbox -o failures                                     # every file with matches, as JSON
toolbox report .toolbox --baseline .toolbox/results.json --update      # record the baseline; commit it
toolbox report .toolbox --baseline .toolbox/results.json               # the gate: exit 1 on a regression
toolbox report .toolbox --baseline .toolbox/results.json -o json | jq '.campaigns'   # campaign progress
```

Each failure in `-o failures` has `path`, `measure` (its matches), `evidence` (`line 55`, one per
match) and `fix`.

## Parameters

| Parameter | Default | Meaning |
|---|---|---|
| `files` | required | Globs of the files scanned. `**` spans directories, `*` doesn't |
| `pattern` | required | The regular expression ([Rust regex syntax](https://docs.rs/regex/latest/regex/#syntax)); each match is one violation. In YAML, single-quote it so `\b` stays a backslash |
| `exclude` | `[]` | Globs never scanned: tests, migrations, vendored code |
| `exempt` | `{}` | Files scanned but exempt, path → reason |
| `name` | `pattern-count` | Names the connector, scorecard and campaign |
| `kind` | `PatternFile` | The kind (PascalCase). Include the package twice with different `name` and `kind` for two patterns |
| `title`, `owner`, `fix` | | The scorecard's heading, owner and fix text |
| `root` | `.` | The checkout, relative to where `toolbox report` runs |

## Changing the standard

- **Widen the pattern** (add a model): edit `pattern`, then
  `toolbox report .toolbox --baseline .toolbox/results.json --update --accept person-orm`. The new
  matches are recorded and listed, regressions in other scorecards still fail, and the campaign
  restarts from the new state. Review the list, then commit the manifest and `results.json`.
- **Exempt a file**: add it to `exempt` with its reason, then `--update`.

A textual pattern can be gamed: renaming a call so it no longer matches passes the gate. Review
fixes, or use a semgrep rule that matches the code's structure.

## Sources

- [docs/design/ratchets.md](../../docs/design/ratchets.md): the ratchet, measures and campaign starts
- [docs/reference/workspace-connector.md](../../docs/reference/workspace-connector.md): `counts` and `matchLines`
