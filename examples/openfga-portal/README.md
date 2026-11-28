# OpenFGA portal

A repository portal for [OpenFGA](https://github.com/openfga/openfga) at v1.23.0, and Seikika's
second reference portal for design review
([D01M87PSM9YQGGW3YQ5X4XCJP63](../../docs/decisions/D01M87PSM9YQGGW3YQ5X4XCJP63.md)). It is about
the box: every scorecard comes from a generic package or from this directory, and the tools it
carries (staticcheck, actionlint, zizmor, OpenSSF Scorecard) run the same outside it. Design review
renders it next to Seikika for Seikika, so a report change that only works for one portal shows up.

| Track | What it scores | From |
|---|---|---|
| `code` | Go files within 800 lines, no `panic(` outside tests, no staticcheck findings | [`file-size`](../../packages/file-size/README.md), [`pattern-count`](../../packages/pattern-count/README.md), [`sarif`](../../packages/sarif/README.md) |
| `delivery` | Workflows, pinned actions, OpenSSF Scorecard checks | [`github-actions`](../../packages/github-actions/README.md), [`openssf-scorecard`](../../packages/openssf-scorecard/README.md) |
| `docs` | README, CODEOWNERS, licence; Markdown links that resolve | [standards/repo.yaml](standards/repo.yaml) (the starter's standards) |
| `vs-spicedb` | OpenFGA against SpiceDB on six factors | [standards/vs-spicedb.yaml](standards/vs-spicedb.yaml), records in [comparisons/](comparisons) |

The comparison is a pre-registered experiment: the factors, bars and method are in
[experiments/openfga-spicedb-compare](../../experiments/openfga-spicedb-compare/README.md), and
its harness writes `comparisons/*.yaml`. Do not edit those records by hand; rerun the harness.

## Run it

Connector roots resolve against the working directory, so the report runs from inside the
checkout, the way a repository's own `.seikika/` portal does.

```bash
examples/openfga-portal/fetch.sh             # clones v1.23.0 into checkout/ (git ignores it)
cd examples/openfga-portal/checkout
seikika doctor ..                           # staticcheck 2026.2.2 needs Go 1.27
seikika report .. --baseline ../results.json -o web --out /tmp/openfga-report
```

`results.json` is this portal's ratchet baseline, written by `--update`. It records each
campaign's starting point, so the report shows progress since then.

`--skip github-actions` leaves out the GitHub API reads when no token is set; the runner connectors
(staticcheck, OpenSSF Scorecard) need the tools `seikika doctor` lists. The report-ui fixture
script renders this portal the same way
([report-ui/README.md](../../npm/report-ui/README.md)).
