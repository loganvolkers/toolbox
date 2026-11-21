# Finding the Tools a Repository Already Uses

*Status: built, 2026-11-21 (D01M801A9QNVCSBBJATYV721TCS). About the box
([toolbox-and-tools.md](./toolbox-and-tools.md)): it reports which tools a repository carries
and which are in the box. It decides nothing. It serves the operator SDLC: setting up a repo
portal, or a new standard.*

---

## 1. Conclusion

**Finding tools takes two passes: a deterministic inventory, then an agent that checks it.**
- **`toolbox tools find`** reads config files, CI, hook and task-runner commands, package
  manifests and installer files, and matches them against a catalog of tools. For each tool it
  prints:
  - the evidence, as file and line;
  - its status: `in box` (a portal connector runs it), `pinned` (in `tools:` only) or `candidate`;
  - the slot that would carry it, with the SARIF command and where that command was checked;
  - the kit that puts it in the box.

  It also lists the repository's own scripts that CI runs, which no catalog knows.
- **The `find-tools` agent skill** checks each candidate before anything goes in the box. Is the
  tool really in use, at what version? Does the SARIF command print SARIF, exit 0 on findings,
  and read the repository's own config? Do the findings match when the tool runs alone? It also
  looks where the inventory can't: the scripts it lists, Makefile targets, contributor docs.

The split is deliberate. Determinism is cheap and repeatable, so it does the reading. It can't
tell a tool in use from a stale config, or tell whether a script is a per-file check or a gate.
Those judgements are the agent's, as with assessments (D79).

## 2. On four repositories

| Repository | Tools found (status) | Scripts CI runs | Time (release build) |
|---|---|---|---|
| This one (T4T) | ruff, zizmor, actionlint, clippy (in box); rustfmt, yamark, codeql (candidates) | 10 | 0.4 s (debug) |
| PostHog fork | 13: ruff, actionlint, semgrep, golangci-lint, eslint, clippy, rustfmt, shellcheck, hadolint, markdownlint, mypy, prettier, codeql | 15 | 0.12 s |
| Codex fork | 7: ruff, eslint, clippy, rustfmt, markdownlint, prettier, cargo-deny | 13 | 0.23 s (debug) |
| gh-aw fork | 8: zizmor, actionlint, semgrep, golangci-lint, shellcheck, markdownlint, prettier, codeql | 16 | 1.06 s, over 25 MB of workflow YAML |

**This repository, checked as the skill says:**
- **rustfmt and yamark:** formatters that CI gates pass or fail.
- **codeql:** already checked by `maintainer-supply-chain`.
- **The ten scripts:** gates and utilities (acceptance, parity, scale, release, ids), not
  per-file checks.

So nothing new enters this box. That is the expected result for a repository whose box was
built by hand: the inventory agrees with it.

**Noise found and fixed in the first runs:**
- Comment lines counted as commands.
- Python scripts under `scripts/` mentioned tools in code, e.g. `take-it-out.py`'s
  `if via == "semgrep"`. Only shell scripts there are now read as commands.
- gh-aw's agentic workflows are Markdown in `.github/workflows/`, and their prose counted. Only
  YAML workflows are read.
- One regex pass per tool over gh-aw's workflows took 20 s in a debug build. A `RegexSet` reads
  each line once.

## 3. The catalog

`crates/toolbox-cli/src/tool-catalog.yaml` holds 24 tools. Each has config globs, a CI pattern,
package names, and its slot: a SARIF command, a fitting, or a built-in connector.

**`checked` says where a SARIF command was run and its output read.** Where it's empty, the
command comes from the tool's docs or help, and the skill tells the agent to run it first.

| Checked | Tools |
|---|---|
| In this repository's standards | ruff 0.14.0, zizmor 1.32.0, actionlint 1.7.12 (fitting) |
| In a kit on a fork | semgrep (PostHog) |
| By hand here | golangci-lint 2.5.0 (`--output.sarif.path stdout --show-stats=false --issues-exit-code=0`; without `--show-stats=false` a summary follows the SARIF on stdout) |
| Not checked | eslint (needs `@microsoft/eslint-formatter-sarif`), hadolint, gitleaks, trivy, tflint, biome |
| No SARIF slot | clippy (the workspace connector), rustfmt, prettier, yamark, shellcheck, markdownlint, yamllint, mypy, cargo-deny, codeql |

## 4. Next

| # | Item |
|---|---|
| F1 | Check the unchecked SARIF commands where the tools can be installed, and fill in `checked` |
| F2 | Let a portal extend the catalog (`--catalog <file>`) for tools only one organisation uses |
| F3 | A formatter slot: the files a formatter's check mode would change, one record per file, so a formatter can be ratcheted instead of gated |

## Sources

- [toolbox-and-tools.md](./toolbox-and-tools.md): slots, fittings and take-it-out
- [tool-installation.md](./tool-installation.md): the `install` ids in the catalog
- `golangci-lint run --help` (2.5.0), run here
- The A/B forks of PostHog, Codex and gh-aw ([experiments/standards-ab](../../experiments/standards-ab))
