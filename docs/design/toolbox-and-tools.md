# The Box and the Tools

*Status: vision and review, 2026-11-21 (D01M7ZRAJT47W11X5X3FEJ6QTX8). It sets the vocabulary every
design doc uses from now on: what the box does, what a tool does, and what Toolbox builds as a tool.
It applies to every SDLC layer: maintainers keep the box small, package authors carry tools in
it, operators fill a portal's box, and users take tools out.*

---

## 1. Conclusion

**Toolbox is a box for tools, not a wrapper around them and not a factory for them.**
- A tool does the work: a linter, a formatter, a static analyser, a test runner. It stays useful
  without the box.
- The box gives tools a home and a standard way to be carried, pinned, run and read together. It
  remembers what they found (the baseline), tracks the jobs left (campaigns), and labels the lid
  (the report).
- You can take any tool out of the box and use it directly, with the same command, the same
  configuration and the same findings.

The A/B Toolbox just lost mostly measured the wrong half: building a tool (a line counter, a
pattern counter) with Toolbox's primitives against writing one in Python (§5). Of the 33 friction
entries from its Toolbox arms, 24 are about the box, 7 about building tools, and 2 about the
experiment. The box was never compared with what teams do without one, which is CI glue around
several tools. That comparison is next (§7).

Toolbox for Toolbox fails the same test (§6). Most of its scorecards read facts from about 6,600
lines of home-made extractors in one connector. Many of those re-implement existing tools, and
none can be used outside Toolbox.

## 2. The analogy

| In a workshop | In Toolbox | Example |
|---|---|---|
| **A tool** | A program that checks or changes code, with its own CLI, configuration and output. Useful without the box | zizmor, actionlint, semgrep, clippy, ESLint and its plugins, ruff |
| **The box** | Holds tools for a repository or a fleet: which tools, at which versions, run how, and what their findings mean to this team | `toolbox.yaml` (connectors, `tools:`), `toolbox doctor`, `toolbox report` |
| **The box's shape** | A standard slot: any tool that fits it goes in without new code | SARIF 2.1.0 for findings; NDJSON records for facts and measures; JUnit for tests |
| **A fitting** | A small adapter for a tool that doesn't fit a standard slot. A fitting is a stopgap: prefer the tool's standard output | `parse: zizmor`, `parse: actionlint`, `parse: semgrep` today |
| **The label on the lid** | What is in the box and whether it's all there | `toolbox describe`, `toolbox doctor` (D90) |
| **The notebook in the lid** | What the tools found last time, so you know if it got worse | The baseline (`results.json`) and the ratchet |
| **The job list** | What still needs doing, and how far along it is | Campaigns, paydown from a recorded start |
| **A kit** | A set of tools with their settings and the rules for using them, which another workshop can take as it is | A package (D89): `github-actions` carries zizmor and actionlint with their pins and scorecards |
| **A jig** | A quick tool made on the bench for one job, from the box's own primitives | A `counts` regex on the workspace connector with a CEL rule; `file-size`, `pattern-count` |

**What a toolbox is not:**
- **A wrapper.** A wrapper has to be gone through to use the tool, and it changes what the tool
  does. If the box adds ignores, flags or filters the tool doesn't know about, running the tool
  directly gives different answers. Then the tool has been wrapped, not boxed.
- **A factory.** Toolbox shouldn't try to be the best way to build every tool. The web ecosystem
  moved its dev tools from JavaScript to Go and Rust (esbuild, swc, Biome, Oxc, Ruff for Python)
  because tools that run on every repository, over millions of files, have to be fast. Those tools
  are built by their communities. The box carries them; it doesn't compete with them.

## 3. What the box promises

Each promise is testable, and each has a check to add (§7).

| # | Promise | Holds today? |
|---|---|---|
| B1 | **Take it out.** The command the box runs is a command you can run, and the tool's configuration lives in the tool's own file | No: the `github-actions` package passes actionlint three `-ignore` flags and a runner-label ignore in `toolbox.yaml`. actionlint 1.7.12's own `.github/actionlint.yaml` (`self-hosted-runner.labels`, `paths.<glob>.ignore`) holds the same settings; checked, exit 0 |
| B2 | **The box doesn't change the findings.** It may decide what they mean (levels, owners, waivers), not what they are | Partly: rules read the tool's findings, but per-tool fittings decide what counts as "suppressed" (the semgrep fitting reads comments) |
| B3 | **Standard slots first.** A tool that emits SARIF, NDJSON or JUnit fits without new Toolbox code | No: there is no SARIF slot. zizmor 1.32.0 emits SARIF 2.1.0, and semgrep 1.163.0's SARIF marks inline suppressions as `suppressions: [{kind: inSource}]` (both checked). The semgrep fitting built this round infers the same from comments |
| B4 | **Pinned and checkable.** Every tool's version is declared, and the box can tell when one is missing or wrong | Yes: `tools:`, `toolbox doctor`, mise (D90) |
| B5 | **Kits travel whole.** A package carries tools, their configuration, and the rules for using them | Partly: packages carry rules and parameters, but not the tool's own config file, a CI snippet or a STANDARD.md (the rerun's phase 4 hand-copied all three) |
| B6 | **Any tool, any version.** Swapping a tool for a better one changes a slot's contents, not the box | Partly: rules name a tool's record fields (`entity.counts.high`), so a new tool means new rules |

## 4. Building tools

Toolbox should help build a tool only where none exists. In order of preference:

| # | When | Build |
|---|---|---|
| 1 | A tool already checks it | Nothing: put the tool in the box (a slot, or a fitting until it has standard output) |
| 2 | A framework checks things like it | A plugin or rule for that framework: an ESLint rule, a semgrep or ast-grep rule, a clippy lint. It runs without Toolbox, in editors and in CI |
| 3 | Nothing checks it, and it will be reused | A standalone tool with standard output (SARIF or NDJSON), in a language that scales, usually Rust. Toolbox may supply the parts (globs, file walking, record shapes) as a library, but the tool must run and be useful without Toolbox |
| 4 | A quick proof of concept, in one repository | A jig: the workspace connector's `files`, `counts`, `captures` and a CEL rule. When a jig is reused, graduate it to step 2 or 3 |

**Rules for the tool-building primitives:**
- **A tool built with them must not need the box.** Its output is SARIF or NDJSON that any CI or
  editor can read, and its configuration is its own file.
- **They aren't the product.** The `file-size` and `pattern-count` packages are jigs. A file-size
  standard is a line count (`wc -l`, or tokei or scc for languages). A pattern standard is better
  as a semgrep or ast-grep rule, which matches syntax, not text. The rerun's reader caught the
  synthetic commit that "fixed" six matches by renaming them; a syntax-aware rule needn't be
  fooled that way.

## 5. The A/B friction, re-sorted

**Layers:**
- **Box:** carrying, running, remembering, reporting, distributing.
- **Tool building:** making a tool with Toolbox's primitives.
- **Fitting:** making a tool fit the box.
- **Experiment:** the test's own setup.

Items are from round 2 (R2) and the rerun (B′) of
[experiments/standards-ab](../../experiments/standards-ab/README.md).

| Friction | From | Layer | Note |
|---|---|---|---|
| The ratchet compared evidence counts, not a number | R2 1 | Box | Fixed: `measure`. Any tool's number fits, not only the box's own |
| A new failing entity wasn't a regression by default | R2 2 | Box | A kit sets `newEntitiesAtLeast` |
| No flat failure list; `jq` over internal JSON | R2 3, B′ | Box | `-o failures` added; a shape that names the tool's own fields is still missing |
| Campaign status needs a server; buried in `-o json`; percent misleading | R2 4, B′ | Box | Paydown added; `toolbox campaign` still needs a server |
| No frozen campaign start | R2 5 | Box | Fixed |
| No way to accept an intended change | R2 6 | Box | Fixed: `--accept`; it doesn't separate newly failing from shifted |
| No package for a size or pattern standard | R2 7 | Tool building | Built as jigs. By §4 these are better as tools: a line counter and a semgrep or ast-grep rule |
| No reason on an exemption | R2 8 | Box | The box's waivers and exemptions should also read the tool's own suppressions (`nosemgrep`, SARIF `suppressions`) |
| Scorecards had no title or description | R2 9 | Box | Fixed |
| The baseline records every entity; churn on clean files | R2 10, B′ | Box | Open: store failing entities only |
| No reference for the workspace connector | R2 11 | Tool building | Fixed |
| No starter for a written standard | R2 12 | Box | Fixed |
| `root:` and the action's install path undocumented; no copy-paste CI workflow | R2 13, B′ | Box | Open |
| No guidance on changing a standard mid-campaign | R2 14, B′ | Box | In `campaigns.md`; not yet in kits |
| Broken pipe panics; percent rounding | R2 15, 16 | Box | Fixed |
| `#` inside a folded CEL block | R2 17 | Box | CEL is the box's policy language; unverified |
| 36 s per PostHog run | R2 18 | Tool building | The workspace connector's `lastChange` (one `git log` per file). Fixed |
| A per-line wrapper script; a `[1..200]` CEL hack to count matches | R2 | Tool building | The jig lacked a fact. A real tool (`rg -c`, semgrep) has it |
| Including a package offline means copying it in | B′ | Box | Open: vendoring |
| The default Markdown lists every passing entity | B′ | Box | Open |
| `--ci` fails on an unrecorded improvement | B′ | Box | Open: two modes, documented |
| `tighten` isn't enforced without a default | B′ | Box | Open bug |
| An exempted file shows as fixed | B′ | Box | Open |
| Glob semantics undocumented | B′ | Tool building | Open |
| STANDARD.md and the workflow hand-copied | B′ | Box | Open: kits should carry them (B5) |
| No self-test for a consumer's gate | B′ | Box | Open |
| No guidance on generated files | B′ | Tool building | Open: a tool's own excludes |
| The CI install docs were outside the rerun's read list | B′ | Experiment | |
| A usage limit cut off a run | B′ | Experiment | |
| `AGENTS.md` doesn't point to the tooling | Both | Box | Open: the box should say where it is |
| The gate compares with the baseline, not the last commit | Both | Box | Open |
| A textual pattern can be gamed | Both | Tool building | Use a syntax-aware tool |
| The standard already existed as a semgrep rule, and both arms rebuilt it | R2 | Box | The box's first job is finding the tools a repository already has |

**What it means:**
- **The cost Toolbox lost on was mostly tool building.** Learning to make a jig costs more than
  writing a script, and a script is a fine proof of concept.
- **The box's friction was real, but the box's value wasn't measured.** No phase had a team with
  several tools and the CI glue they need. Toolbox's claim lies there: one place for every tool,
  its version, its baseline, its campaign and its report.
- **The most expensive miss was a box failure.** PostHog already had the standard as a semgrep
  rule in CI, and neither arm found it. A box that lists the tools a repository already runs
  would have started there.

## 6. Toolbox for Toolbox's tools, critically

**What T4T carries:**
- Tools in the box: cargo test, clippy, zizmor, actionlint, `toolbox describe`.
- Almost everything else comes from the workspace connector's built-in extractors, which are
  facts produced inside the box. They are jigs that grew.

| Extractor | What it reads | Existing tools (candidates, not yet tried here) | Verdict |
|---|---|---|---|
| `rust.rs` (311 lines) | Doc coverage, function length, complexity, `unwrap`s per crate | clippy lints `missing_docs_in_private_items`, `too_many_lines`, `cognitive_complexity`, `unwrap_used`; rust-code-analysis | **Replace**: clippy already runs in T4T; take its JSON per lint |
| `workflows.rs` (221) | Triggers, permissions, timeouts, `uses:` pinning | zizmor (`unpinned-uses`, `excessive-permissions`), actionlint | **Shrink**: keep only what neither reports (concurrency, timeouts) |
| `deps.rs` (231) | Direct Cargo dependencies, workspace declarations | cargo-deny, cargo-machete, `cargo metadata` | **Replace** where cargo-deny covers it |
| `site.rs`, docs links in `main.rs` | Broken relative links, sidebar reach, secret shapes | lychee (links), gitleaks (secrets) | **Replace** links and secrets; keep sidebar reach (Starlight-specific) |
| `plugins.rs` (294), `specs.rs` (188) | plugin.json and mcp.json against the 1.0.0 schemas; schema dialects | A JSON Schema validator (check-jsonschema) with the official schemas | **Replace** the validation; keep the facts the schemas don't express |
| `skills.rs` (731) | Agent Skills frontmatter, word counts | `skills-ref validate` (official) for the spec part | **Replace** the spec part; extract the counts as a skill linter usable without Toolbox |
| `agentsmd.rs` (241) | AGENTS.md links, commands, size | lychee for links | **Extract** as a standalone AGENTS.md linter: no existing tool does this well |
| `npm.rs` (415), `js_workspace.rs` (566), `expo.rs` (268), `storybook.rs` (304) | package.json, pnpm workspace, Expo and Storybook facts | `expo doctor`, syncpack, knip, Storybook's own checks | **Fit** existing tools first; keep only workspace-wide facts no tool reports |
| `repo.rs` (278) | README, LICENSE, CODEOWNERS | OpenSSF Scorecard (repository checks), codeowners validators | **Fit** OpenSSF Scorecard; keep CODEOWNERS parsing (ownership is the box's) |
| `connectors.rs` (474), `examples.rs`, `data.rs`, assessments | Toolbox's own connectors, write-ups, declared data, agent judgements | None: they are facts about the box itself | **Keep in the box** |
| `betterer.rs` (114) | Betterer's results file | Betterer itself | **Keep**: a fitting, the right kind of code |
| `files`, `counts`, `captures` | Lines and regex matches per file | `wc`, tokei, scc, ripgrep, ast-grep, semgrep | **Keep as the jig bench**; don't grow it into a linter |

T4T should be the box's best example, so its own standards should pass B1: an agent should be
able to run each tool behind a T4T score directly and get the same finding.

## 7. Recommendation: what to improve next

**Make the box a box before building more tools.** In order:

1. **A SARIF slot** (`parse: sarif` in the runner). Read results, rule ids, locations and
   `suppressions` per file, so most linters and security tools fit with no Toolbox code. That
   includes zizmor, semgrep, ruff, ESLint with a SARIF formatter, gitleaks, trivy and Checkov; the
   first two are checked, the rest are claims to verify. Move the `github-actions` and `semgrep`
   kits onto it, and keep the native fittings only where SARIF loses information. **Check:** a
   kit's findings are identical through the slot and through the tool's own SARIF file.
2. **Take-it-out (B1) as a test.**
   - Move the actionlint ignores into `.github/actionlint.yaml`.
   - Make each kit's tool config a file the kit carries (`.zizmor.yml`, `.semgrep/`).
   - Have `-o failures` say which tool and command produced each finding.
   - **Check:** for every kit, running the documented tool command gives the same findings as
     `toolbox report`.
3. **Find the tools a repository already has.** `toolbox describe` (or a starter step) should
   list the tools the CI already runs (semgrep, ESLint, ruff, Betterer) and suggest the kit for
   each, before suggesting a jig. The PostHog miss was exactly this.
4. **The box's open friction** (§5, all Box layer):
   - vendoring kits offline;
   - a quiet default report;
   - baselines that store failing entities only;
   - `toolbox campaign` without a server;
   - a copy-paste CI workflow;
   - kits that carry a STANDARD.md and a CI snippet;
   - a per-commit comparison.
5. **A box A/B** (for R4 and R5, not R3): a repository with several tools already in CI. Tasks:
   - add a tool;
   - upgrade one with a ratchet;
   - swap a tool for a better one;
   - have a new agent run one tool directly.

   Compare CI glue written by hand with the same tools in a box. Measure the glue a team
   maintains and tokens, and require take-it-out parity (B1) of both arms.
6. **Shrink T4T's home-made tools** per §6, starting with `rust.rs` (clippy already runs) and the
   link checks (lychee). Extract the AGENTS.md and skill linters as standalone tools only if
   someone outside Toolbox would use them.

**Not next:** more jigs, more record kinds in the workspace connector, or more extractors for
facts that an existing tool reports.

## 8. Writing rule

Every design doc and package README says which side of the line it is on:
- **The box:** carrying, pinning, running, remembering, reporting, distributing.
- **A tool**, and whether it is an existing tool in the box, a fitting, or a tool built with
  Toolbox's primitives (a jig, or a standalone tool).

New extractors in a Toolbox connector need a sentence on why no existing tool, plugin or slot
covers the fact ([AGENTS.md](../../AGENTS.md), *Writing style*).

## Sources

- [experiments/standards-ab](../../experiments/standards-ab/README.md): the friction tables (round 2 and the rerun)
- [crates/toolbox-connector-workspace/src/](../../crates/toolbox-connector-workspace/src/): the extractors in §6 and their line counts
- [packages/github-actions/fragments/github-actions.yaml](../../packages/github-actions/fragments/github-actions.yaml): the actionlint flags in B1
- Checked here, 2026-11-21:
  - actionlint 1.7.12 honours `.github/actionlint.yaml` `self-hosted-runner.labels` and `paths.<glob>.ignore`;
  - `zizmor 1.32.0 --format sarif` emits SARIF 2.1.0;
  - semgrep 1.163.0 `--sarif` marks `nosemgrep` findings `suppressions: [{kind: inSource}]`. On PostHog's `posthog/management/commands` its SARIF has 22 results, where the JSON run with `--disable-nosem` had 19; the difference is not yet explained.
- Existing tools in §4 and §6 (tokei, scc, ast-grep, lychee, gitleaks, cargo-deny, cargo-machete, check-jsonschema, OpenSSF Scorecard, `expo doctor`, syncpack, knip, trivy, Checkov), and SARIF formatters for ruff and ESLint: named from their projects, not yet run here
