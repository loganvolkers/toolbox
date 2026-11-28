# Seikika for Seikika

*The portal Seikika maintainers run on this repository. Design and findings:
[docs/design/seikika-for-seikika.md](../docs/design/seikika-for-seikika.md).*

Seikika catalogues its own crates, docs, scripts and tests, the CLI commands and MCP tools it promises,
and scores each against one [SDLC layer's](../docs/sdlc.md) standard. It runs on a laptop or in CI:
no Postgres, no OpenFGA, no network.

| Scorecard | Layer | Kind | What it raises |
|---|---|---|---|
| `maintainer-crate` | 1. Maintainer | Crate | Crate docs, tests that pass, clippy, public-API docs, function size and complexity, unwraps |
| `maintainer-sdk-docs` | 1. Maintainer | SdkDocs | Public entry points, item and prop comments, links, and package references. New SDKs must reach gold |
| `maintainer-docs` | 1. Maintainer | Doc | Links resolve, the SDLC layer is named, design docs have a status banner and sources |
| `maintainer-benchmark` | 1. Maintainer | Benchmark | A declared question and readable artifacts, a registered comparison method, and published results with follow-up |
| `operator-cli` | 2. Operator | Command | Operator commands have help, are documented and are run by a script or test |
| `user-cli` | 3. User | Command | Read commands have help, structured output (`-o json`) and are exercised |
| `user-mcp` | 3. User | McpTool | Tools are described and annotated, inputs described, documented, and contract-tested over MCP |
| `maintainer-ci-security` | 1. Maintainer | Workflow | actionlint clean; no high zizmor findings; least-privilege token; actions pinned to SHAs; then no medium findings and no write tokens on risky triggers; then zizmor-clean |
| `maintainer-ci-reliability` | 1. Maintainer | Workflow | Main not stuck red; workflow still enabled; first-try fail rate ≤ 20/10/5% (silver and gold over at least 10 runs); failures visible; job timeouts |
| `maintainer-ci-speed` | 1. Maintainer | Workflow | Superseded runs cancelled; p90 ≤ 15 min |
| `maintainer-action-deps` | 1. Maintainer | ActionDependency | Each third-party action is SHA-pinned in every workflow that uses it |
| `maintainer-testing` | maintainer | TestingContract | Behavior specifications, test paths and CI discovery |
| `review-testing` | advisory | TestingContract | Current agent review of risk coverage, oracles and execution |
| `meta-rule` | all (our standards) | Rule | Fix text; tested; decided and both outcomes on fixtures; evidence; no nested loops; readable; at gold, tests pin both outcomes, evidence names what failed and the fix names a place. New rules must reach bronze |
| `meta-scorecard` | all (our standards) | Scorecard | Tested; no empty level; fixtures reach two levels; every rule tested. New scorecards must reach bronze |
| `meta-kind`, `meta-field`, `meta-connector` (`PortalConnector`) | all (our model) | Kind, Field, PortalConnector | In fixtures and tested, described, fields described; populated, described, has a shape; records mapped, fixtures, freshness, refreshable (D79) |
| `meta-assessment`, `meta-rubric` | all (our judgement) | Assessment, Rubric | Suggestions and a named assessor, then cited findings, then cited reasons; rubrics described, used, guided, discriminating (D80) |
| `review-rule`, `review-scorecard`, `review-crate`, `review-assessment` | advisory | Rule, Scorecard, Crate, Assessment | Coding agents' assessments against the rubrics in the meta package and `standards/`, stored in `assessments/`: fresh, no failing criterion, no doubts. Never gate (D79, D80) |
| `review-sdlc-stage` | advisory | SdlcStage | Independent of the stage's declared status, reviews whether an agent can complete the stage and verify its exit |
| `maintainer-skill` | 1. Maintainer | Skill | Spec-valid frontmatter; description says when, in third person; then a short procedure with an example and a body budget; then limits, eval cases, and no hidden text or hardcoded secrets. New skills must reach bronze |
| `maintainer-connector` | 1. Maintainer | Connector | Described, config documented, speaks stdio or remote; then named kinds, snapshot and not-found (push-only exempt), a unit test; then CI runs it and no hardcoded secrets. New connectors must reach bronze |
| `maintainer-dependency` | 1. Maintainer | Dependency | Bounded registry requirement; then one version in the lockfile; then a shared dependency is declared in the workspace. New dependencies must reach bronze |
| `maintainer-agent-doc` | 1. Maintainer | AgentDoc | One source of instructions whose links resolve; then a test command and a 500-line ceiling; then a pointer at the decision log and no secrets or hidden characters. New files must reach bronze |
| `maintainer-package` | 1. Maintainer | PackageJson | Named, private or versioned, bounded registry ranges; then a lockfile and a check script; then a description, `engines.node`, and `npm ci` in CI. New manifests must reach bronze |
| `maintainer-schema` | 1. Maintainer | JsonSchema | Draft 2020-12, an absolute id, a description; then a title and the compiler loads it; then local `$ref`s only. New schemas must reach bronze |
| `maintainer-example` | 1. Maintainer | Example | A heading and resolving links; then a command that checks the example; then a pointer at the design. New write-ups must reach bronze |
| `maintainer-plugin` | 1. Maintainer | Plugin | The 1.0.0 plugin schema and legal skill paths; then `mcp.json` and a one-token stdio command; then no secret header, path escape or hardcoded secret. New packages must reach bronze |
| `maintainer-site` | 1. Maintainer | SitePage | A title, a description and resolving links; then in the sidebar and not a draft; then no secret shape. New pages must reach bronze |
| `maintainer-supply-chain` | 1. Maintainer | Repository | Dependabot, code scanning and a security policy; then CODEOWNERS covers CI; then the code-scanning action is pinned. New repositories must reach bronze |
| `maintainer-repo` | 1. Maintainer | Repository | The lowest counted scorecard, with unknown rules excluded. New repositories must reach bronze |

## Layout

| Path | Holds |
|---|---|
| `seikika.yaml` | The portal, policies, connectors, MCP instructions and the `include` list |
| `standards/<topic>.yaml` | One topic's kinds, the scorecards that score them, and their rubrics (for example `cli.yaml`: `Command`, `McpTool`, `operator-cli`, `user-cli`, `cli-skill`, `user-mcp`) |
| `campaigns.yaml` | Tracks and time-bound campaigns over those scorecards ([campaigns.md](../docs/design/campaigns.md)) |
| `fixtures/`, `tests/` | Curated fixtures and the standards' own tests (`seikika test portal`) |
| `testing/` | [Functional correctness contracts](../docs/design/correctness-testing.md) and their test evidence |
| `sdlc/`, `assessments/` | The stages of the four SDLC layers (D87) and agents' assessments (D79) |
| `results.json` | The ratchet's baseline, generated by `--update` |
| `STANDARD.md` | Every scorecard's rules and fixes, the packages' parameters, the campaigns and the commands, generated by `seikika report portal -o standard` (CI checks it is current; regenerate, don't edit) |

The [meta-scoring package](../packages/meta-scoring/README.md) supplies the catalog and
review scorecards. Seikika for Seikika also describes the starter and reference portals so
their fixture evidence is visible. Its review campaign covers Seikika's own definitions.
The assessment evidence campaign follows citations and independent reviews of those judgements.

A new standard goes in its topic's file, or a new file in `standards/`, which the glob picks up.
`seikika validate portal` names the file and pointer of any problem (D95). Connectors stay in
`seikika.yaml`, because one connector's sources can't be split across files.

`results.json` is generated: never edit it by hand, and never merge it by hand. On a rebase
conflict, take upstream's copy, then regenerate it:

```bash
git checkout --ours portal/results.json   # during a rebase, "ours" is upstream
seikika report portal --skip runner,github-actions --baseline portal/results.json --update
git add portal/results.json
```

After OpenSSF greening (or any improvement to `maintainer-openssf`), commit that same `--update`
command so rust CI does not fail `REPORT_RESULT: stale`. Record live OpenSSF scores with the
**runner** first (Scorecard installed; do not `--skip runner` for that pass), then rewrite the CI
baseline under `--skip runner,github-actions` as above.

Before pushing edits to `.github/workflows/*.yml` (except `release.yml`, which cargo-dist owns),
run `yamark format` on the touched files so `yamark format --check` in rust CI stays green.

The site's copy of the report isn't committed: the pages workflow generates it for each commit
(D01M7YAXH89ZJQ7Q4HP77NMHBX8). To preview it locally with the same pinned linters CI uses, run
`scripts/regen-site-report.sh` (actionlint 1.7.12 via `scripts/ci-tools.sh`, `zizmor==1.32.0`, then
`doctor` and `report … -o web --out site/.generated`). A bare `seikika report … -o web` without
those tools on `PATH` writes `unknown` for ci-lint rules and diverges from the pages build.

## Run it

```bash
seikika report portal --skip runner,github-actions               # ~1 s: every scorecard, with evidence for failing rules
seikika report portal                               # ~2 min: also cargo test, clippy and the MCP trace per crate
seikika report portal --skip runner,github-actions --baseline portal/results.json          # the ratchet (CI adds --ci)
seikika report portal --skip runner,github-actions --baseline portal/results.json --update # record an improvement; commit it
seikika report portal --skip runner,github-actions -o html --out report.html            # one page to publish
seikika report portal --skip runner,github-actions -o web --out report                  # element, JSON, and that page; a host fills the slots
scripts/self-portal.sh serve                        # live portal on :8090 (API, MCP, UI)
seikika test portal                                 # the standards' own tests, over the curated fixtures
seikika assess brief rule:seikika/user-mcp/contract-tested --portal portal --skip runner,github-actions   # brief your agent
seikika assess check portal --skip runner,github-actions                       # validate assessments/: fresh, stale, missing
```

`seikika` here is `target/debug/seikika` (connectors are found next to it). Run `pnpm install --frozen-lockfile` from the checkout root before a live report. The SDK documentation check uses the workspace's TypeScript and Vue tools. The ratchet needs
`zizmor` 1.32.0 (`pip install zizmor==1.32.0` or `pipx install zizmor==1.32.0`) and
`actionlint` 1.7.12 (`scripts/ci-tools.sh <bin-dir> actionlint`, the same pin as CI) on the
`PATH`; without them their rules are `unknown` (reported, not compared) and `--ci`/`--update`
refuse to run. For the site report preview, prefer `scripts/regen-site-report.sh`, which installs
both. `github-actions` needs `GITHUB_TOKEN` and changes over time, so it is skipped in the ratchet.
CI fails when anything gets worse than `portal/results.json`, and when an improvement hasn't been
recorded in it.

With `serve` running, any agent can use it:

```bash
seikika score crate:seikika-derive --url http://127.0.0.1:8090 --as maintainer
seikika scorecard maintainer-crate --rule fully-documented --url http://127.0.0.1:8090 --as maintainer   # a work list with evidence
seikika refresh crate:seikika-derive --url http://127.0.0.1:8090 --as maintainer   # reruns that crate's tests + clippy
seikika mcp --url http://127.0.0.1:8090 --as maintainer                             # stdio MCP for an agent harness
```

To register it in an MCP client (for example Claude Code's `.mcp.json`), use the stdio bridge:
`{"mcpServers": {"seikika-self": {"command": "target/debug/seikika", "args": ["mcp", "--url", "http://127.0.0.1:8090", "--as", "maintainer"]}}}`.

## How it works

| Connector | Records | How | Refresh |
|---|---|---|---|
| `workspace` ([crate](../crates/seikika-connector-workspace)) | `crate`, `doc`, `file`, `skill`, `connector`, `dependency`, `agent-doc`, `package-json`, `json-schema`, `example` | `cargo metadata`, a `syn` parse of each crate, markdown and file scans with configurable regex captures and counts, one record per `SKILL.md`, one record per `seikika-connector-*` crate, one record per direct Cargo dependency, one record per `AGENTS.md` and its aliases, one record per `package.json`, one record per JSON Schema, and one record per example or experiment write-up | Re-reads one crate, doc, file, skill, connector, dependency, agent instruction file, npm manifest, schema or example |
| `surfaces` (the runner binary) | `command`, `mcp-tool`, `scorecard`, `rule`, `kind`, `field`, `connector`, `rubric`, `sdk-docs` | Runs `seikika describe` and the [SDK documentation check](../scripts/sdk-docs.mjs) | Reruns jobs for the requested record kinds |
| `ci-lint` (the runner binary) | `zizmor`, `actionlint` | Runs both linters over `.github/workflows` and `actions/`; one record per file, clean ones included | Reruns them |
| `github-actions` (the github binary) | `workflow-runs` | Run history per workflow on `main` from the GitHub API: fail rate, re-runs, failing streak, p90 time | Re-reads it |
| `runner` ([crate](../crates/seikika-connector-runner)) | `test`, `clippy`, `mcp-calls` | Runs `cargo test -p`, `cargo clippy -p` per crate, and the MCP contract tests with `SEIKIKA_MCP_TRACE` (observed calls per tool) | Reruns one crate's jobs |

Connectors report raw facts (every function with its length and complexity, every undocumented
public item); the manifest's scorecards set the thresholds. `fixtures/` is a trimmed snapshot of
this repo with hand-made runner results, and `tests/standards.test.yaml` pins what each rule means.
