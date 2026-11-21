# file-size: a Toolbox package

**Files over a line limit are violations.** One `include` gives a repository the standard, a
ratchet and a campaign:

- `toolbox report` lists every file over the limit, with its line count and the fix;
- the ratchet fails a pull request that grows a violating file, or brings a new or renamed file
  over the limit, and reports the files that shrank;
- the campaign `<name>-zero` counts files over the limit, and lines over it, from the day the
  baseline first records it.

It needs nothing but the `toolbox` binaries: the files are read by `builtin:workspace`, without a
build or the network. Include it once per standard, with that standard's files and limit.

**Box or tool:** this package is a **jig**, a quick tool made from Toolbox's own primitives
([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md) §4). It suits a proof of concept in
one repository. For a standard many repositories share, prefer a tool that counts lines on its own (`wc -l`, tokei, scc) in the box.

| Brings | Named |
|---|---|
| Connector | `<name>` (`builtin:workspace`, `file` records) |
| Kind | `<kind>` (default `SizedFile`): `path`, `lines` |
| Scorecard | `<name>`, one level `within-limit`, rule `within-limit` |
| Campaign | `<name>-zero` |

## Use it

`.toolbox/toolbox.yaml`, for TypeScript components of at most 400 lines, tests excluded:

```yaml
apiVersion: toolbox.dev/v1alpha1
portal: {name: my-repo, owner: group:maintainers, toolbox: "0.1"}
include:
  - package: github:loganvolkers/toolbox/packages/file-size@<tag>   # or a relative path
    with:
      name: component-size
      kind: Component
      files: ["web/src/**/*.tsx"]
      exclude: ["**/__tests__/**", "**/*.test.tsx"]
      exempt: {web/src/icons.tsx: generated from the icon set}
      limit: 400
```

Then, from the repository root:

```bash
toolbox vendor <toolbox checkout>/packages/file-size --portal .toolbox    # offline: copy the package in (or include github:…@tag)
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

## Parameters

| Parameter | Default | Meaning |
|---|---|---|
| `files` | required | Globs of the files measured. `**` spans directories, `*` doesn't |
| `limit` | required | More lines than this is a violation. Raising it shows in review, and the ratchet reports the files it lets pass as improvements |
| `exclude` | `[]` | Globs never measured: tests, generated or vendored code |
| `exempt` | `{}` | Files measured but exempt, path → reason. The reason stays in the manifest, next to the path |
| `name` | `file-size` | Names the connector, scorecard and campaign |
| `kind` | `SizedFile` | The kind (PascalCase). Include the package twice with different `name` and `kind` for two standards |
| `title`, `owner`, `fix` | | The scorecard's heading, owner and fix text |
| `root` | `.` | The checkout, relative to where `toolbox report` runs |

## What a pull request sees

| Change | Gate |
|---|---|
| A violating file grows by one line | `✗ worse … within-limit evidence grew 270 → 271` (exit 1) |
| A new file over the limit, or one that crosses it | `✗ worse … new, at none; new entities must reach within-limit` or `… within-limit now fails` (exit 1) |
| A violating file shrinks, or drops under the limit | `✓ better …` (exit 0); `--update` records it |
| An unrelated edit | `matches` (exit 0) |

The rule's `measure` is the lines **over** the limit, so the campaign's measure is how much code
has to move.

## Changing the standard

- **Tighten the limit** (400 → 300): change `limit`, then
  `toolbox report .toolbox --baseline .toolbox/results.json --update --accept component-size`. The
  newly failing files are recorded and listed, regressions in other scorecards still fail, and
  the campaign restarts from the new state. Review the list, then commit the manifest and
  `results.json` together.
- **Exempt a file**: add it to `exempt` with its reason, then `--update` (it now passes).
- **Exclude a directory**: add a glob to `exclude`, then `--update`.

## Sources

- [docs/design/ratchets.md](../../docs/design/ratchets.md): the ratchet, measures and campaign starts
- [docs/reference/workspace-connector.md](../../docs/reference/workspace-connector.md): `files`, `exclude` and `file` records
