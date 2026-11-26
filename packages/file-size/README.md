# file-size: a Seikika package

**Files over a line limit are violations.** One `include` gives a repository the standard, a
ratchet and a campaign:

- `seikika report` lists every file over the limit, with its line count and the fix;
- the ratchet fails a pull request that grows a violating file, or brings a new or renamed file
  over the limit, and reports the files that shrank;
- the campaign `<name>-zero` counts files over the limit, and lines over it, from the day the
  baseline first records it.

It needs nothing but the `seikika` binaries: the files are read by `builtin:workspace`, without a
build or the network. Include it once per standard, with that standard's files and limit.

**Box or tool:** this package is a **jig**, a quick tool made from Seikika's own primitives
([seikika-and-tools.md](../../docs/design/seikika-and-tools.md) §4). It suits a proof of concept in
one repository. For a standard many repositories share, prefer a tool that counts lines on its own (`wc -l`, tokei, scc) in the box.

| Brings | Named |
|---|---|
| Connector | `<name>` (`builtin:workspace`, `file` records) |
| Kind | `<kind>` (default `SizedFile`): `path`, `lines` |
| Scorecard | `<name>`, one level `within-limit`, rule `within-limit` |
| Campaign | `<name>-zero` |

## Use it

`.seikika/seikika.yaml`, for TypeScript components of at most 400 lines, tests excluded:

```yaml
apiVersion: seikika.com/v1alpha1
portal: {name: my-repo, owner: group:maintainers, seikika: "0.1"}
include:
  - package: github:seikika/seikika/packages/file-size@<tag>   # or a relative path
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
seikika vendor <seikika checkout>/packages/file-size --portal .seikika    # offline: copy the package in (or include github:…@tag)
seikika report .seikika -o failures                                     # every violation, as JSON
seikika report .seikika --baseline .seikika/results.json --update      # record the baseline; commit it
seikika report .seikika --baseline .seikika/results.json               # the gate: exit 1 on a regression
seikika report .seikika --against origin/main                          # what this branch changed, no baseline needed
seikika campaign --portal .seikika --baseline .seikika/results.json    # campaign progress since its start
seikika report .seikika -o standard --out .seikika/STANDARD.md         # the standard's docs, from the manifest
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
| `root` | `.` | The checkout, relative to where `seikika report` runs |
| `track` | none | The portal track its scorecard and campaign belong to; leave unset if the portal has no tracks |

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
  `seikika report .seikika --baseline .seikika/results.json --update --accept component-size`. The
  newly failing files are recorded and listed, regressions in other scorecards still fail, and
  the campaign restarts from the new state. Review the list, then commit the manifest and
  `results.json` together.
- **Exempt a file**: add it to `exempt` with its reason, then `--update` (it now passes).
- **Exclude a directory**: add a glob to `exclude`, then `--update`.

## Sources

- [docs/design/ratchets.md](../../docs/design/ratchets.md): the ratchet, measures and campaign starts
- [docs/reference/workspace-connector.md](../../docs/reference/workspace-connector.md): `files`, `exclude` and `file` records
