# Seikika for Seikika: Scoring Our Own Four SDLC Layers With Seikika

*Status: experiment built 2026-11-20 (D65). Agent skills are catalogued and scored (D72,
[skill-standards.md](./skill-standards.md)). First-party connectors are too (D73,
[connector-standards.md](./connector-standards.md)). Direct Cargo dependencies are too (D74,
[dependency-standards.md](./dependency-standards.md)). Agent instruction files are too (D75,
[agents-md-standards.md](./agents-md-standards.md)). npm manifests are too (D76,
[package-standards.md](./package-standards.md)). JSON Schemas are too (D77,
[schema-standards.md](./schema-standards.md)). Example and experiment write-ups are too (D78,
[example-standards.md](./example-standards.md)). Agent plugin packages are too (D81,
[plugin-standards.md](./plugin-standards.md)), harness aliases and the check commands in `AGENTS.md`
are too (D82, D83, [agents-md-standards.md](./agents-md-standards.md)), cited decision ids are too
(D84, [decision-citations.md](./decision-citations.md)), GitHub Advanced Security config is too
(D85, [supply-chain-standards.md](./supply-chain-standards.md)), and the repository's level is the
lowest of those scorecards (D86, [repo-level.md](./repo-level.md)). Public docs pages are too
(D91, [site-standards.md](./site-standards.md)), and the site publishes this repo's HTML report
(D92). The portal is [`portal/`](../../.seikika/README.md); run
it with `scripts/self-portal.sh`. It applies Seikika to the [four SDLC layers](../sdlc.md) of
Seikika itself. Related: [meta-catalog.md](./meta-catalog.md) (a portal cataloguing its own
configuration), [maintainer-sdlc.md](./maintainer-sdlc.md), [user-sdlc.md](./user-sdlc.md).*

---

## 1. Conclusion

**It works, and it changes what Seikika is for.** Organisations build catalogs to measure, maintain
and raise their standards. The same holds for the Seikika repository. A local portal can:
- catalogue Seikika's crates, docs, scripts, tests, CLI commands and MCP tools;
- score them against one standard per SDLC layer;
- give an agent everything it needs to pick a failing rule, fix it, refresh, and see the score move.

It needs no server, no Postgres and no OpenFGA: one binary plus two local connectors.

What the experiment showed:

| Question | Answer |
|---|---|
| Can Seikika run usefully as a local, single-repo portal? | **Yes.** In-memory store, dev auth, no authorization. A static report takes about 1 s; with tests and clippy per crate, about 2 min; a per-crate refresh, about 7 s |
| Do local connectors fit the connector model? | **Yes**, including the control channel. `refresh crate:X` routes to both connectors through the records behind the entity; the runner reruns only that crate's jobs |
| Can an agent raise a standard using only the portal? | **Yes** (§6): a fresh agent with only the portal's interfaces took all 11 MCP tools to gold on the user SDLC scorecard, and I raised the maintainer docs scorecard |
| Did it find real problems? | **Yes**, on its first run (§5). Examples: none of the 11 MCP tools had a direct contract test, several CLI arguments had no help, a 418-line function, and three docs (one mine, written minutes earlier) with an SDLC layer left unnamed or no sources |
| Did it find gaps in Seikika itself? | **Yes** (§7): rule-level evidence, multi-hop derivation, score history and ratchets, connector hot reload, and a productised local mode |

**The recursion.** Each SDLC layer of Seikika becomes a catalog that Seikika scores:

| Layer | What is catalogued | Scorecards here |
|---|---|---|
| 1. Maintainer | Crates, docs, scripts and tests of this repository, the agent skills it ships, its first-party connectors, the libraries it builds against, the instruction files agents load, the npm manifests it builds, the JSON Schemas it compiles, the examples it asks agents to copy, the agent plugin packages it ships, the public docs pages it publishes, and the repository's own supply-chain config | `maintainer-crate`, `maintainer-docs`, `maintainer-skill`, `maintainer-connector`, `maintainer-dependency`, `maintainer-agent-doc`, `maintainer-package`, `maintainer-schema`, `maintainer-example`, `maintainer-plugin`, `maintainer-site`, `maintainer-supply-chain`, `maintainer-repo` |
| 2. Operator | The commands and docs operators use to build portals (and, through the [meta catalog](./meta-catalog.md), a portal's own configuration) | `operator-cli` |
| 3. User | The contracts apps and agents rely on: MCP tools and read commands | `user-mcp`, `user-cli` |

**The generalisation.** Nothing in the two connectors is specific to Seikika: Seikika-specific
behaviour lives in the manifest's config and rules. So any team can run the same scorecards their
fleet portal applies, locally, on one repository, before they push ("shift-left scorecards"). That
is a layer-4 (user) use of Seikika: developers building on its context.

## 2. Why

- **Standards are the point of a catalog.** A catalog without standards is an inventory. What
  organisations want is to measure, maintain and raise their engineering standards across every part
  of their software lifecycle.
- **We should hold ourselves to it.** AGENTS.md states standards (docs, verification, naming the
  SDLC layer, sources). Until now they were checked by reading. Now they are rules with evidence and
  fixes.
- **Open source means local.** Nobody will host a Seikika server for the Seikika project. So the
  self portal must run on a laptop and in CI from the repository alone. That is also a strong test of
  charter goal G1 at its smallest size.

## 3. Design

```
scripts/self-portal.sh serve|report|snapshot
  └─ seikika serve .seikika/      (in-memory store, dev auth, no OpenFGA)
       ├─ workspace connector   cargo metadata + syn → crate; markdown → doc; scripts/tests → file
       └─ runner connector      cargo test -p / cargo clippy -p → test, clippy;  seikika describe → command, mcp-tool
```

| Piece | What it does | Generic? |
|---|---|---|
| [`seikika-connector-workspace`](../../crates/seikika-connector-workspace) | `crate` records: `cargo metadata` plus a `syn` parse of each crate. Reports every function (lines, cyclomatic complexity), every undocumented public item, unit and integration test counts, unwraps outside tests, and in-workspace dependencies. `doc` records: title, status banner, sources section, broken relative links, inline code spans. `file` records: lines. `skill` records: one per `SKILL.md` (D72, [skill-standards.md](./skill-standards.md)). `connector` records: one per `seikika-connector-*` crate (D73, [connector-standards.md](./connector-standards.md)). `dependency` records: one per direct Cargo dependency (D74, [dependency-standards.md](./dependency-standards.md)). `agent-doc` records: one per `AGENTS.md` and its aliases (D75, [agents-md-standards.md](./agents-md-standards.md)). `package-json` records: one per npm manifest (D76, [package-standards.md](./package-standards.md)). `json-schema` records: one per document in `crates/seikika-model/schema/` (D77, [schema-standards.md](./schema-standards.md)). `example` records: one per example or experiment write-up (D78, [example-standards.md](./example-standards.md)). `plugin` records: one per `plugins/*/plugin.json` (D81, [plugin-standards.md](./plugin-standards.md)). `site-page` records: one per Starlight content page (D91, [site-standards.md](./site-standards.md)). `repo` records: one per checkout, with Dependabot, code scanning, a security policy and CODEOWNERS (D85, [supply-chain-standards.md](./supply-chain-standards.md)). Docs and files also get **configurable regex `captures` and `counts`** | Any Rust workspace, markdown docs, Agent Skills directories, connector crates, agent instruction files, npm manifests, JSON Schemas, example write-ups, agent plugin packages, and a Starlight docs site |
| [`seikika-connector-runner`](../../crates/seikika-connector-runner) | Runs configured jobs, per crate or once, and parses `cargo test`, clippy JSON, or passes through NDJSON records from any command. Supports refresh by external id | Any local command |
| `seikika describe` | Prints Seikika's CLI commands (from clap) and built-in MCP tools as connector records: help, undocumented arguments, structured output, the MCP tool each read command calls | Seikika-specific, and doubles as a contract snapshot for the user SDLC |
| [`.seikika/seikika.yaml`](../../.seikika/seikika.yaml) | Kinds `Crate`, `Doc`, `File`, `Command`, `McpTool`. Relations from docs and files to commands, tools and crates come from captures. Five scorecards; one policy (append-only docs exempt from wording rules) | — |
| [`portal/tests`](../../.seikika/tests/standards.test.yaml) + fixtures | 16 expectations that pin what each rule means, over a trimmed snapshot of the repo | — |

**Design choices**

- **Connectors report facts; the manifest holds the standard.** The workspace connector applies no
  thresholds. It lists every function, and `maintainer-crate` decides that more than 80 lines is
  long (gold) and more than 150 is huge (silver). Raising the bar is a reviewed manifest change,
  with `seikika plan` showing which entities drop a level.
- **Mentions become relations through captures.** A doc mentions an MCP tool when one of its code
  spans equals the tool's name. A script exercises a command when a capture regex finds
  `seikika <command>`. The mapping emits candidate ids, and the pipeline drops those that don't
  resolve (see F5).
- **Static and executed facts are separate connectors.** Reads are fast and resync every 5 minutes.
  Executions are slow, resync hourly, and are refreshed per entity.

## 4. Standards (v0)

| Scorecard | Bronze | Silver | Gold |
|---|---|---|---|
| `maintainer-crate` (skips crates under 50 code lines) | Crate docs; has tests; tests pass; clippy clean | ≥ 80% of public items documented; no function > 150 lines; complexity ≤ 30 | 100% documented; no function > 80 lines; complexity ≤ 15; ≤ 1 unwrap per 100 lines |
| `maintainer-docs` (skips append-only logs) | Relative links resolve; every "SDLC" names its layer | Design docs: status banner, sources section | — |
| `operator-cli` | Has help; documented | All arguments described; run by a script or test | — |
| `user-cli` | Has help; `-o json` | All arguments described; exercised | — |
| `cli-skill` | A `SKILL.md` mentions the command's invocation | — | — |
| `user-mcp` | Described; annotated | Inputs described; documented | Contract-tested over MCP (not just via the CLI) |
| `maintainer-skill` | Spec-valid frontmatter; description says when, in third person | At least three numbered steps; a fenced example; links resolve; body within 500 lines / ~5000 tokens; no Windows paths | States a limit or error path; at least three eval cases; no hidden text or hardcoded secrets; one reference hop and at most three supporting files |
| `maintainer-connector` | Description and crate docs; config documented; `run_stdio` or `run_remote` | Named kinds; `snapshot_complete` and `not_found` (push-only exempt); at least one unit test | CI runs `connector test` (or an acceptance script for a push connector); no hardcoded secret shapes |
| `maintainer-dependency` | A bounded version requirement from a registry, not git | Present in `Cargo.lock` as exactly one version | A dependency used by more than one crate is declared in `[workspace.dependencies]` |
| `maintainer-agent-doc` | The file is `AGENTS.md`, or an alias that points at it; relative links resolve | The canonical file names a build or test command and stays within 500 lines | The canonical file points at the decision log; its check commands appear in a workflow; no secret shapes or hidden characters |
| `maintainer-package` | Named; private or versioned; no wildcard, git or tarball range | A lockfile beside the manifest, and a test, build, check or typecheck script | A description, `engines.node`, and CI that installs with `npm ci` |
| `maintainer-schema` | Draft 2020-12, an absolute `https` `$id`, a root description | A title, and the compiler or the reference validator loads the file | Every `$ref` is local; no secret shape |
| `maintainer-example` | A heading, and relative links resolve | The write-up names a command that checks it | It points at a design doc or a decision; no secret shape |
| `maintainer-plugin` | The 1.0.0 plugin schema, a legal name, a description, skills only at `skills/<name>/SKILL.md` | `mcp.json` on the 1.0.0 schema, and a one-token stdio command | No secret header, no `..` path, no secret shape |
| `maintainer-site` | A title, a description, and links that resolve | In the sidebar, and not a draft | No secret shape |
| `maintainer-supply-chain` | Dependabot for cargo, npm and github-actions; a code-scanning workflow; a `SECURITY.md` | CODEOWNERS covers `.github/workflows/ci.yml` | The code-scanning action is pinned to a commit |
| `maintainer-repo` | Every counted scorecard is at least bronze | Every counted scorecard is at least silver | Every counted scorecard is gold |

The rules are deliberately simple. Their value is in the evidence fields an agent can act on, such as
`undocumented`, `longFunctions`, `complexFunctions`, `clippy.messages`, `tests.output`,
`undocumentedArgs`, `undescribedProperties`, `documentedIn`, `exercisedBy` and `viaCli`.

## 5. First run (2026-11-20, before any fixes)

| Scorecard | Result |
|---|---|
| `maintainer-crate` | 1 gold, 3 silver, 6 bronze, 3 none, 2 skipped (placeholders). Tests and clippy pass everywhere. The failures are docs coverage, `seikika-derive`'s 418-line `Pipeline::run`, complexity up to 51, and 3 crates with no tests (`seikika-cli`, `git-files`, `scim`) |
| `maintainer-docs` | 33 silver; failures: `dashboards.md` (a false positive, F8), `maintainer-sdlc.md` with no sources, 3 docs that left the SDLC layer unnamed |
| `operator-cli` | 2 silver, 5 bronze, 2 none. `push` and `describe` aren't documented; most commands have undescribed arguments; `plan` and `token create` aren't exercised by any script |
| `user-cli` | 1 silver, 9 bronze: undescribed arguments; 7 of 9 read commands are run by no script |
| `user-mcp` | 4 silver, 7 bronze: 7 tools have undescribed inputs; **no tool is called directly over MCP in any test** (they are reached only through the CLI) |

## 6. The agent loop

*(Results of the two runs are recorded in [research-log.md](../research-log.md).)*

1. **Maintainer docs (by the authoring agent).** The agent ran `query --kind Doc --filter …` for
   failing docs, read `score doc:…` for the rule and fix, and edited the five docs. It then killed the
   workspace connector process (the supervisor restarted it with the rebuilt binary), ran
   `refresh doc:…`, and checked the scores: 4 of 5 rose to silver. The fifth failed `links-resolve` on
   a link to this document, which didn't exist yet. The portal caught a forward reference.
2. **User MCP (by a fresh agent with only the portal's interfaces).** See §6.1.

### 6.1 Fresh-agent run

A fresh general-purpose agent was given AGENTS.md, the portal URL, and the goal "raise the
`user-mcp` scorecard". It was told to use only the CLI and MCP, not to change `portal/` or `docs/`,
and not to game the checks. It had no knowledge of what failed.

| | Before (snapshot `18`) | After (snapshot `27`) |
|---|---|---|
| `user-mcp` | 4 silver, 7 bronze; every tool failed `contract-tested`, 7 failed `inputs-described` | **11 gold** |

What it changed (reviewed, then committed with this experiment):
- **Descriptions** for the 15 undescribed MCP input properties in `crates/seikika-api/src/mcp.rs`.
  Each one was taken from the REST handler's actual behaviour, with defaults.
- **`crates/seikika-api/tests/mcp.rs`**: 5 contract tests that call every built-in tool with
  `tools/call` and assert on both `structuredContent` and the Markdown:
  - `tools/list` checks names, hints and that every input is described;
  - reads, including A5 and field projection;
  - actions, matching `expected/options.json`, plus a 422;
  - `refresh_entity` end to end: the test plays a remote connector that long-polls for the command
    and returns a changed record;
  - unknown tools.

  This closes the user SDLC gap "contract test kit" for the built-in tools.

It took 56 tool calls and found a real bug while writing tests: MCP `get` advertises a `fields`
input that nothing reads.

The fresh agent's friction report produced findings F9–F14 below. The manifest fixes it justified
were applied at once:
- `documented` now requires `docs/v1-spec.md`, not any doc;
- the runner's `refresh` block declares `command` and `mcp-tool`;
- the MCP instructions list every evidence field.

## 7. Findings: what Seikika lacked, and what's now built

**Status 2026-11-20 (D66):** the core changes these findings asked for are built:

| Finding | Built |
|---|---|
| F1, F4 | `seikika report` (connectors once, recompute, md/json/html) with a Betterer-style ratchet (`--baseline`, `--update`, `--ci`). CI gates this repo on `.seikika/results.json` ([ratchets.md](./ratchets.md), [repo-portals.md](./repo-portals.md)) |
| F2, F11 | Rule `evidence:` CEL, shown by `scorecard_status`, `explain`, `scorecard_report` and `report`, and counted by the ratchet |
| F5 | `relation.dropped` diagnostics per relation; `unresolved: ignore` for candidate relations |
| F7 | `--fields` on `get` and `query`; every CLI argument has help |
| F9 | `SEIKIKA_MCP_TRACE` records observed `tools/call`s, and the runner's `tally` job reports them per tool. `contract-tested` uses observed calls when the runner ran |
| F10 | MCP `scorecard_report`, CLI `seikika scorecard`, REST `/v1/scorecards[/{name}]`; `query` filters can read `scores['<card>']` |
| F12 | `refresh_entity` reports changed fields and the changed entities over the snapshot range |
| F13 | `get` honours `fields` and lists `outbound` references; `scorecard_status` narrows `structuredContent` too |

**Still open:**
- F3: multi-hop derivation;
- F6: connector hot reload (`seikika dev`);
- the `seikika diff` half of F12.

The original findings follow.


| # | Finding | Proposal |
|---|---|---|
| F1 | **A local, single-repo mode works but is a shell script.** `report` is "run each connector once, then `derive`" | Productise as `seikika report [--static]` (one-shot connectors + recompute + scorecards), the local twin of `serve`. It is also how a developer runs fleet scorecards on their repo before pushing |
| F2 | **Rule evidence is by convention.** Fix texts name evidence fields by hand ("see `longFunctions`"), and `explain --rule` can't show *which* items fail | Add an optional rule `evidence:` CEL expression returning the failing items; show it in `scorecard_status`, `explain` and the UI |
| F3 | **No multi-hop derivation.** Computed fields see inbound entities' *stored* fields, not their computed ones. So "an MCP tool is tested if a tested CLI command calls it" can't be expressed | Evaluate computed fields in dependency order across kinds, or allow a bounded relation walk in CEL. Worked around with a separate rule |
| F4 | **No history or ratchet.** "Raise the bar" needs before and after, and CI should stop regressions. Today there are only snapshot ids | A scorecard ratchet: `seikika report --check <baseline.json>` fails if any entity drops a level; the baseline is committed and updated deliberately. The history archive (dashboards.md) for servers |
| F5 | **Dropped relation edges are silent.** Mapping every code span to a candidate id is convenient, but typos in real references vanish too | Report dropped edges per relation as `derive` diagnostics (counts, plus a sample) |
| F6 | **Connector binaries don't hot-reload.** After rebuilding a connector, the server keeps the old process | `seikika dev` (planned) should restart in-image connectors whose binary changed; meanwhile killing the child works because the supervisor restarts it |
| F7 | **CLI and MCP drift.** MCP `query` takes `fields`; the CLI has no `--fields`, and its markdown output lists ids only. The read commands' arguments mostly lack help | Generate CLI read commands from the MCP tool schemas, or test them against each other (`user-cli` + `user-mcp` now show the drift) |
| F8 | **Standards need tests too.** Two rules had false positives on the first run: a numbered `## 5. Sources` heading, and a quoted "SDLC". A time-estimate rule was dropped because market facts ("6–12 months to production") look like estimates | Keep curated fixtures and expectations per scorecard (done: `portal/tests`); treat rule changes like code changes |

| F9 | **"Contract-tested" means "mentioned", not "called".** The rule matches a `"name": "<tool>"` literal in tests and scripts. So a helper like `call("search", …)` doesn't count, any fixture JSON with `"name": "get"` does, and `#[cfg(test)]` unit tests aren't scanned | Measure *observed* calls, in keeping with "done means observed" (D49): the server records each `tools/call` (tool and outcome) to a coverage file when `SEIKIKA_MCP_TRACE` is set; a runner job runs the tests with it set and reports tools actually called and asserted |
| F10 | **No scorecard-wide view for agents.** "Which entities fail rule X?" took 11 `score` calls; scorecard results aren't visible to `query` filters; `score scorecard:user-mcp` is a 404 (the UI's `/v1/views/scorecard/{name}` exists but has no MCP tool or CLI) | An MCP tool and CLI over the scorecard view (`scorecard_report <name> [--rule R --outcome fail]`), and scorecard outcomes exposed to `query` filters |
| F11 | **`scorecard_status` gives fixes, not evidence.** The agent needed one `explain --rule` per rule per entity to see observed values | Fold the F2 `evidence:` into `scorecard_status` output for failing rules |
| F12 | **Refresh doesn't say what it changed.** `refresh mcp-tool:search` reran `seikika describe` and updated all 11 tools silently. Background resyncs advance the snapshot (18 → 23 with no action), so before and after by snapshot id mixes in unrelated changes | Refresh results list the entities whose fields changed; add `seikika diff <snapshot> <snapshot>` (the `plan` diff, over inputs instead of manifests) |
| F13 | **MCP and CLI bugs found by the tests.** MCP `get` advertises an unused `fields` input; `scorecard_status`'s `scorecard` argument filters only the Markdown, not `structuredContent`; `get` on an entity shows only inbound relations (outbound needs `graph`) | Fix in the next maintainer pass; the `user-mcp` rules can't see these, which is F9 again |
| F14 | **Common names make "documented" noisy.** `query`, `get` and `search` are "documented" by any backticked word in any doc | Fixed in the manifest: `documented` requires `docs/v1-spec.md`. Generally, rules should name the authoritative doc |

## 8. Next steps (proposed)

1. ~~Decide on F1, F2/F11, F10 and F9~~: built (D66), see §7.
2. ~~Add a ratchet baseline for the self portal in CI (F4)~~: built and gating (`.seikika/results.json`).
3. Extend the catalogue:
   - acceptance scenarios (A1–A12) as entities, with status from the scripts;
   - decisions and open questions as entities, so an agent can ask "which decisions have no owning
     design doc?";
   - coverage through a runner job (`cargo llvm-cov`) once the toolchain component is available in CI.
4. Offer the two connectors to operators and users as built-ins (`builtin:workspace`,
   `builtin:runner`), with a starter manifest for "score this repo locally".

## Sources

- The portal, connectors and script: [`portal/`](../../.seikika/README.md),
  [`crates/seikika-connector-workspace`](../../crates/seikika-connector-workspace),
  [`crates/seikika-connector-runner`](../../crates/seikika-connector-runner),
  [`scripts/self-portal.sh`](../../scripts/self-portal.sh).
- Measurements and both agent runs: [research-log.md](../research-log.md).
- Connector control and scoped refresh: [connector-control.md](./connector-control.md).
- Cyclomatic complexity as used here: McCabe, "A Complexity Measure", IEEE TSE SE-2(4), 1976
  (counted as 1 + `if`/`while`/`for`/`loop` + match arms beyond the first + `&&`/`||`).
