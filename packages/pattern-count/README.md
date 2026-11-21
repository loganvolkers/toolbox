# pattern-count: a Toolbox package

**Every match of a regular expression is a violation.** One `include` gives a repository the
standard, a ratchet and a campaign:

- `toolbox report` lists every file with matches, with the line of each and the fix;
- the ratchet fails a pull request that adds a match to any file (including a new file), and
  reports the files that lost matches;
- the campaign `<name>-zero` counts files with matches, and matches, from the day the baseline
  first records it.

It needs nothing but the `toolbox` binaries: files are read by `builtin:workspace`. If the pattern is
already a semgrep rule, use [`packages/semgrep`](../semgrep/README.md) instead: it keeps the rule
in one place and counts its `nosemgrep` suppressions.

**Box or tool:** this package is a **jig**, a quick tool made from Toolbox's own primitives
([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md) §4). It suits a proof of concept in
one repository. A regex matches text, so a rename can fool it. For a standard that lasts, write the pattern as a semgrep or ast-grep rule, which runs without Toolbox, and carry it with [`semgrep`](../semgrep/README.md).

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
      name: no-console
      kind: SourceFile
      files: ["web/src/**/*.ts", "web/src/**/*.tsx"]
      exclude: ["**/*.test.ts"]
      exempt:
        web/src/logger.ts: the logger itself writes to the console
      pattern: '\bconsole\.(log|debug)\('
      fix: Use the logger from web/src/logger.ts.
```

Then, from the repository root:

```bash
toolbox vendor <toolbox checkout>/packages/pattern-count --portal .toolbox    # offline: copy the package in (or include github:…@tag)
toolbox report .toolbox -o failures                                     # every violation, as JSON
toolbox report .toolbox --baseline .toolbox/results.json --update      # record the baseline; commit it
toolbox report .toolbox --baseline .toolbox/results.json               # the gate: exit 1 on a regression
toolbox report .toolbox --against origin/main                          # what this branch changed, no baseline needed
toolbox campaign --portal .toolbox --baseline .toolbox/results.json    # campaign progress since its start
toolbox report .toolbox -o standard --out .toolbox/STANDARD.md         # the standard's docs, from the manifest
```

The baseline records only files that violate, so adding clean files never makes it stale. In CI,
`--baseline` alone fails on regressions; `--ci` also fails when an improvement isn't recorded yet.
`STANDARD.md` ends with a copy-paste GitHub Actions workflow for the
[report action](../../actions/report/README.md).

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

- **Widen the pattern** (another call): edit `pattern`, then
  `toolbox report .toolbox --baseline .toolbox/results.json --update --accept no-console`. The new
  matches are recorded and listed, regressions in other scorecards still fail, and the campaign
  restarts from the new state. Review the list, then commit the manifest and `results.json`.
- **Exempt a file**: add it to `exempt` with its reason, then `--update`.

A textual pattern can be gamed: renaming a call so it no longer matches passes the gate. Review
fixes, or use a semgrep rule that matches the code's structure.

## Sources

- [docs/design/ratchets.md](../../docs/design/ratchets.md): the ratchet, measures and campaign starts
- [docs/reference/workspace-connector.md](../../docs/reference/workspace-connector.md): `counts` and `matchLines`
