# Toolbox for Toolbox: Scoring Our Own Three SDLC Layers With Toolbox

*Status: experiment built 2026-11-20 (D65). The portal is [`portal/`](../../portal/README.md); run
it with `scripts/self-portal.sh`. It applies Toolbox to the [three SDLC layers](../sdlc.md) of
Toolbox itself. Related: [meta-catalog.md](./meta-catalog.md) (a portal cataloguing its own
configuration), [maintainer-sdlc.md](./maintainer-sdlc.md), [user-sdlc.md](./user-sdlc.md).*

---

## 1. Conclusion

**It works, and it changes what Toolbox is for.** Organisations build catalogs to measure, maintain
and raise their standards. The same holds for the Toolbox repository. A local portal can:
- catalogue Toolbox's crates, docs, scripts, tests, CLI commands and MCP tools;
- score them against one standard per SDLC layer;
- give an agent everything it needs to pick a failing rule, fix it, refresh, and see the score move.

It needs no server, no Postgres and no OpenFGA: one binary plus two local connectors.

What the experiment showed:

| Question | Answer |
|---|---|
| Can Toolbox run usefully as a local, single-repo portal? | **Yes.** In-memory store, dev auth, no authorization. A static report takes about 1 s; with tests and clippy per crate, about 2 min; a per-crate refresh, about 7 s |
| Do local connectors fit the connector model? | **Yes**, including the control channel. `refresh crate:X` routes to both connectors through the records behind the entity; the runner reruns only that crate's jobs |
| Can an agent raise a standard using only the portal? | **Yes** (§6): a fresh agent with only the portal's interfaces took all 11 MCP tools to gold on the user SDLC scorecard, and I raised the maintainer docs scorecard |
| Did it find real problems? | **Yes**, on its first run (§5). Examples: none of the 11 MCP tools had a direct contract test, several CLI arguments had no help, a 418-line function, and three docs (one mine, written minutes earlier) with an SDLC layer left unnamed or no sources |
| Did it find gaps in Toolbox itself? | **Yes** (§7): rule-level evidence, multi-hop derivation, score history and ratchets, connector hot reload, and a productised local mode |

**The recursion.** Each SDLC layer of Toolbox becomes a catalog that Toolbox scores:

| Layer | What is catalogued | Scorecards here |
|---|---|---|
| 1. Maintainer | Crates, docs, scripts and tests of this repository | `maintainer-crate`, `maintainer-docs` |
| 2. Operator | The commands and docs operators use to build portals (and, through the [meta catalog](./meta-catalog.md), a portal's own configuration) | `operator-cli` |
| 3. User | The contracts apps and agents rely on: MCP tools and read commands | `user-mcp`, `user-cli` |

**The generalisation.** Nothing in the two connectors is specific to Toolbox: Toolbox-specific
behaviour lives in the manifest's config and rules. So any team can run the same scorecards their
fleet portal applies, locally, on one repository, before they push ("shift-left scorecards"). That
is a layer-3 use of Toolbox: developers building on its context.

## 2. Why

- **Standards are the point of a catalog.** A catalog without standards is an inventory. What
  organisations want is to measure, maintain and raise their engineering standards across every part
  of their software lifecycle.
- **We should hold ourselves to it.** AGENTS.md states standards (docs, verification, naming the
  SDLC layer, sources). Until now they were checked by reading. Now they are rules with evidence and
  fixes.
- **Open source means local.** Nobody will host a Toolbox server for the Toolbox project. So the
  self portal must run on a laptop and in CI from the repository alone. That is also a strong test of
  charter goal G1 at its smallest size.

## 3. Design

```
scripts/self-portal.sh serve|report|snapshot
  └─ toolbox serve portal/      (in-memory store, dev auth, no OpenFGA)
       ├─ workspace connector   cargo metadata + syn → crate; markdown → doc; scripts/tests → file
       └─ runner connector      cargo test -p / cargo clippy -p → test, clippy;  toolbox describe → command, mcp-tool
```

| Piece | What it does | Generic? |
|---|---|---|
| [`toolbox-connector-workspace`](../../crates/toolbox-connector-workspace) | `crate` records: `cargo metadata` plus a `syn` parse of each crate. Reports every function (lines, cyclomatic complexity), every undocumented public item, unit and integration test counts, unwraps outside tests, and in-workspace dependencies. `doc` records: title, status banner, sources section, broken relative links, inline code spans. `file` records: lines. Docs and files also get **configurable regex `captures` and `counts`** | Any Rust workspace and markdown docs |
| [`toolbox-connector-runner`](../../crates/toolbox-connector-runner) | Runs configured jobs, per crate or once, and parses `cargo test`, clippy JSON, or passes through NDJSON records from any command. Supports refresh by external id | Any local command |
| `toolbox describe` | Prints Toolbox's CLI commands (from clap) and built-in MCP tools as connector records: help, undocumented arguments, structured output, the MCP tool each read command calls | Toolbox-specific, and doubles as a contract snapshot for the user SDLC |
| [`portal/toolbox.yaml`](../../portal/toolbox.yaml) | Kinds `Crate`, `Doc`, `File`, `Command`, `McpTool`. Relations from docs and files to commands, tools and crates come from captures. Five scorecards; one policy (append-only docs exempt from wording rules) | — |
| [`portal/tests`](../../portal/tests/standards.test.yaml) + fixtures | 16 expectations that pin what each rule means, over a trimmed snapshot of the repo | — |

**Design choices**

- **Connectors report facts; the manifest holds the standard.** The workspace connector applies no
  thresholds. It lists every function, and `maintainer-crate` decides that more than 80 lines is
  long (gold) and more than 150 is huge (silver). Raising the bar is a reviewed manifest change,
  with `toolbox plan` showing which entities drop a level.
- **Mentions become relations through captures.** A doc mentions an MCP tool when one of its code
  spans equals the tool's name. A script exercises a command when a capture regex finds
  `toolbox <command>`. The mapping emits candidate ids, and the pipeline drops those that don't
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
| `user-mcp` | Described; annotated | Inputs described; documented | Contract-tested over MCP (not just via the CLI) |

The rules are deliberately simple. Their value is in the evidence fields an agent can act on, such as
`undocumented`, `longFunctions`, `complexFunctions`, `clippy.messages`, `tests.output`,
`undocumentedArgs`, `undescribedProperties`, `documentedIn`, `exercisedBy` and `viaCli`.

## 5. First run (2026-11-20, before any fixes)

| Scorecard | Result |
|---|---|
| `maintainer-crate` | 1 gold, 3 silver, 6 bronze, 3 none, 2 skipped (placeholders). Tests and clippy pass everywhere. The failures are docs coverage, `toolbox-derive`'s 418-line `Pipeline::run`, complexity up to 51, and 3 crates with no tests (`toolbox-cli`, `git-files`, `scim`) |
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
- **Descriptions** for the 15 undescribed MCP input properties in `crates/toolbox-api/src/mcp.rs`.
  Each one was taken from the REST handler's actual behaviour, with defaults.
- **`crates/toolbox-api/tests/mcp.rs`**: 5 contract tests that call every built-in tool with
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

## 7. Findings: what Toolbox lacked, and what's now built

**Status 2026-11-20 (D66):** the core changes these findings asked for are built:

| Finding | Built |
|---|---|
| F1, F4 | `toolbox report` (connectors once, recompute, md/json/html) with a Betterer-style ratchet (`--baseline`, `--update`, `--ci`). CI gates this repo on `portal/results.json` ([ratchets.md](./ratchets.md), [repo-portals.md](./repo-portals.md)) |
| F2, F11 | Rule `evidence:` CEL, shown by `scorecard_status`, `explain`, `scorecard_report` and `report`, and counted by the ratchet |
| F5 | `relation.dropped` diagnostics per relation; `unresolved: ignore` for candidate relations |
| F7 | `--fields` on `get` and `query`; every CLI argument has help |
| F9 | `TOOLBOX_MCP_TRACE` records observed `tools/call`s, and the runner's `tally` job reports them per tool. `contract-tested` uses observed calls when the runner ran |
| F10 | MCP `scorecard_report`, CLI `toolbox scorecard`, REST `/v1/scorecards[/{name}]`; `query` filters can read `scores['<card>']` |
| F12 | `refresh_entity` reports changed fields and the changed entities over the snapshot range |
| F13 | `get` honours `fields` and lists `outbound` references; `scorecard_status` narrows `structuredContent` too |

**Still open:**
- F3: multi-hop derivation;
- F6: connector hot reload (`toolbox dev`);
- the `toolbox diff` half of F12.

The original findings follow.


| # | Finding | Proposal |
|---|---|---|
| F1 | **A local, single-repo mode works but is a shell script.** `report` is "run each connector once, then `derive`" | Productise as `toolbox report [--static]` (one-shot connectors + recompute + scorecards), the local twin of `serve`. It is also how a developer runs fleet scorecards on their repo before pushing |
| F2 | **Rule evidence is by convention.** Fix texts name evidence fields by hand ("see `longFunctions`"), and `explain --rule` can't show *which* items fail | Add an optional rule `evidence:` CEL expression returning the failing items; show it in `scorecard_status`, `explain` and the UI |
| F3 | **No multi-hop derivation.** Computed fields see inbound entities' *stored* fields, not their computed ones. So "an MCP tool is tested if a tested CLI command calls it" can't be expressed | Evaluate computed fields in dependency order across kinds, or allow a bounded relation walk in CEL. Worked around with a separate rule |
| F4 | **No history or ratchet.** "Raise the bar" needs before and after, and CI should stop regressions. Today there are only snapshot ids | A scorecard ratchet: `toolbox report --check <baseline.json>` fails if any entity drops a level; the baseline is committed and updated deliberately. The history archive (dashboards.md) for servers |
| F5 | **Dropped relation edges are silent.** Mapping every code span to a candidate id is convenient, but typos in real references vanish too | Report dropped edges per relation as `derive` diagnostics (counts, plus a sample) |
| F6 | **Connector binaries don't hot-reload.** After rebuilding a connector, the server keeps the old process | `toolbox dev` (planned) should restart in-image connectors whose binary changed; meanwhile killing the child works because the supervisor restarts it |
| F7 | **CLI and MCP drift.** MCP `query` takes `fields`; the CLI has no `--fields`, and its markdown output lists ids only. The read commands' arguments mostly lack help | Generate CLI read commands from the MCP tool schemas, or test them against each other (`user-cli` + `user-mcp` now show the drift) |
| F8 | **Standards need tests too.** Two rules had false positives on the first run: a numbered `## 5. Sources` heading, and a quoted "SDLC". A time-estimate rule was dropped because market facts ("6–12 months to production") look like estimates | Keep curated fixtures and expectations per scorecard (done: `portal/tests`); treat rule changes like code changes |

| F9 | **"Contract-tested" means "mentioned", not "called".** The rule matches a `"name": "<tool>"` literal in tests and scripts. So a helper like `call("search", …)` doesn't count, any fixture JSON with `"name": "get"` does, and `#[cfg(test)]` unit tests aren't scanned | Measure *observed* calls, in keeping with "done means observed" (D49): the server records each `tools/call` (tool and outcome) to a coverage file when `TOOLBOX_MCP_TRACE` is set; a runner job runs the tests with it set and reports tools actually called and asserted |
| F10 | **No scorecard-wide view for agents.** "Which entities fail rule X?" took 11 `score` calls; scorecard results aren't visible to `query` filters; `score scorecard:user-mcp` is a 404 (the UI's `/v1/views/scorecard/{name}` exists but has no MCP tool or CLI) | An MCP tool and CLI over the scorecard view (`scorecard_report <name> [--rule R --outcome fail]`), and scorecard outcomes exposed to `query` filters |
| F11 | **`scorecard_status` gives fixes, not evidence.** The agent needed one `explain --rule` per rule per entity to see observed values | Fold the F2 `evidence:` into `scorecard_status` output for failing rules |
| F12 | **Refresh doesn't say what it changed.** `refresh mcp-tool:search` reran `toolbox describe` and updated all 11 tools silently. Background resyncs advance the snapshot (18 → 23 with no action), so before and after by snapshot id mixes in unrelated changes | Refresh results list the entities whose fields changed; add `toolbox diff <snapshot> <snapshot>` (the `plan` diff, over inputs instead of manifests) |
| F13 | **MCP and CLI bugs found by the tests.** MCP `get` advertises an unused `fields` input; `scorecard_status`'s `scorecard` argument filters only the Markdown, not `structuredContent`; `get` on an entity shows only inbound relations (outbound needs `graph`) | Fix in the next maintainer pass; the `user-mcp` rules can't see these, which is F9 again |
| F14 | **Common names make "documented" noisy.** `query`, `get` and `search` are "documented" by any backticked word in any doc | Fixed in the manifest: `documented` requires `docs/v1-spec.md`. Generally, rules should name the authoritative doc |

## 8. Next steps (proposed)

1. ~~Decide on F1, F2/F11, F10 and F9~~: built (D66), see §7.
2. ~~Add a ratchet baseline for the self portal in CI (F4)~~: built and gating (`portal/results.json`).
3. Extend the catalogue:
   - acceptance scenarios (A1–A12) as entities, with status from the scripts;
   - decisions and open questions as entities, so an agent can ask "which decisions have no owning
     design doc?";
   - coverage through a runner job (`cargo llvm-cov`) once the toolchain component is available in CI.
4. Offer the two connectors to operators and users as built-ins (`builtin:workspace`,
   `builtin:runner`), with a starter manifest for "score this repo locally".

## Sources

- The portal, connectors and script: [`portal/`](../../portal/README.md),
  [`crates/toolbox-connector-workspace`](../../crates/toolbox-connector-workspace),
  [`crates/toolbox-connector-runner`](../../crates/toolbox-connector-runner),
  [`scripts/self-portal.sh`](../../scripts/self-portal.sh).
- Measurements and both agent runs: [research-log.md](../research-log.md).
- Connector control and scoped refresh: [connector-control.md](./connector-control.md).
- Cyclomatic complexity as used here: McCabe, "A Complexity Measure", IEEE TSE SE-2(4), 1976
  (counted as 1 + `if`/`while`/`for`/`loop` + match arms beyond the first + `&&`/`||`).
