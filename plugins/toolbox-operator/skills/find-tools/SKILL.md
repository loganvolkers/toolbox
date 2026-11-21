---
name: find-tools
description: >-
  Finds the tools a repository already uses (linters, scanners, formatters, its own check scripts) and decides, candidate by candidate, which belong in the box and through which slot. Use when setting up a repo portal, before writing a new standard or connector extractor, or when asked what Toolbox could carry for a repository.
---

# Find the tools a repository already uses

`toolbox tools find` gives evidence; you give the verdict. Its inventory is deterministic (config files, CI and hook commands, package manifests, matched against a catalog), so it can't tell a tool in use from a stale config, and it can't see a check the repository wrote itself. Check every candidate before anything goes in the box.

## 1. Inventory

```
toolbox tools find . --format json
```

Each tool has a `status`: `in box` (a portal connector runs it), `pinned` (in `tools:`, nothing runs it) or `candidate`. It also has `evidence`, a `slot` (`sarif`, `fitting: …`, `connector: …` or `none known`), a SARIF command, and `checked`: where that command was last run. `scripts` lists the repository's own scripts that CI and hooks run.

## 2. Check each candidate

For each `candidate` and `pinned` tool, in this order:

1. **In use?** Open each evidence line. A CI step that runs it, or a config file a hook reads, counts. A config with no caller, a commented-out step, or a mention in prose doesn't. A tool used once in a release job is not a standard.
2. **Installed at what version?** Run the tool's `--version`, or read the version CI installs. That becomes its pin.
3. **The SARIF command works?** If `checked` is empty, the command comes from the tool's docs. Run it on one or two files yourself. Confirm stdout parses as SARIF 2.1.0 with `runs[0].results`, that it exits 0 when there are findings, and that it reads the repository's own config file. Fix the command if any of these fail. If `checked` is set, still run it once at the repository's version.
4. **Same findings alone?** The command must give the same findings with no Toolbox around (take-it-out). Settings go in the tool's config file, not in the command.

## 3. Look where the inventory can't

- **Each script under `scripts`:** read it. A check that reports per file (sizes, banned patterns, naming) is a tool. Can it print SARIF? Is it one of the existing kits (`file-size`, `pattern-count`)? A pass/fail gate (an acceptance test, a release step) stays in CI.
- **Makefile and justfile targets, CONTRIBUTING.md and AGENTS.md:** the commands contributors are told to run.
- **Tools used with no config file:** e.g. `npx <tool>` in CI. Grep CI for `npx`, `uvx` and `pipx run`.

## 4. Decide and record

| Finding | Do |
|---|---|
| In use, SARIF works | Put it in the box with the `sarif` kit, or the tool's own kit (`next` names it); pin it in `tools:` with `install` ids |
| In use, no SARIF, structured output | Name the fitting it needs; don't write one without saying why no slot fits |
| Formatter or pass/fail gate | Leave it in CI; a box rule would only repeat CI's verdict |
| Not in use | Leave it out, and say so |

Before proposing a new connector extractor, say why no existing tool, framework plugin or standard slot (SARIF, NDJSON, JUnit) covers the fact. Report a table with each tool, its evidence, how you checked it, and the decision. Then run `toolbox validate`, `toolbox doctor` and `toolbox report` for each tool you put in the box.
