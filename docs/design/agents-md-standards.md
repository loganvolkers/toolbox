# AGENTS.md Standards: Cataloguing the Instructions Agents Load

*Status: deterministic catalog and scorecard built, 2026-11-20 (D75). Alias globs widened and check
commands joined to workflows, 2026-11-20 (D82, D83). Seikika for Seikika catalogues `AGENTS.md` and
the aliases harnesses still read, and scores them with `maintainer-agent-doc`. Related:
[agent-plugins.md](./agent-plugins.md), [plugin-standards.md](./plugin-standards.md),
[seikika-for-seikika.md](./seikika-for-seikika.md).*

---

## 1. Conclusion

**One canonical instruction file, and aliases that point at it.**

`AGENTS.md` is the filename coding agents look for. `CLAUDE.md` and `GEMINI.md` are aliases from
harnesses that predate that name. A repository that keeps a second, divergent copy will be followed
by one agent and ignored by another. The checks that do not need a model:

| Question | What answers it | In this repo |
|---|---|---|
| Is there one source of instructions, and do its links resolve? | The filename, a mention of `AGENTS.md`, relative links | bronze |
| Does the canonical file say how to test, and is it short enough to load? | Command words, a 500-line ceiling | silver |
| Does it point at the decision log, and do its check commands run in CI? | A `docs/decisions` link; backtick invocations whose tool and subcommand appear in a workflow | gold |

The 500-line ceiling is the agents.md v1.1 proposal's guideline ("aim for under 500 lines"), not a
field the current format requires. The format has no required headings. Frontmatter is optional, so
a missing `description` is not a failure. A judgement of whether the prose is wise is the same class
of problem as an LLM assessment: useful, and not a gate.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **agents.md** (agentsmd/agents.md README) | A Markdown file named `AGENTS.md`. The example has dev-environment tips, testing instructions and PR instructions. No required schema | The filename, and that a canonical file names a command |
| **v1.1 proposal** (agentsmd/agents.md issue 135) | Jurisdiction by directory, nearest file wins, optional frontmatter (`description`, `tags`), aim for under 500 lines. Not ratified | The line ceiling, labelled as a proposal. Frontmatter stays optional |
| **Alias practice** | Harnesses read `CLAUDE.md` and `GEMINI.md`. The common fix is a symlink or a pointer at `AGENTS.md` | An alias must mention `AGENTS.md` and is not required to repeat the commands |
| **This repository** | `AGENTS.md` is the maintainer working rules and points at `docs/decisions/`. `CLAUDE.md` is five lines that link there | The decision-log link is the gold rule, because the file is not the decision log |

## 3. The catalog

`agentDocs` globs: `**/AGENTS.md`, `**/CLAUDE.md`, `**/GEMINI.md`, `**/.cursorrules`,
`**/.github/copilot-instructions.md`, `**/.github/instructions/*.instructions.md`,
`**/.cursor/rules/**/*.md`, `**/.cursor/rules/**/*.mdc`, `**/.claude/rules/**/*.md` (D82). Id
`agent-doc:<path>`. The same
file is also a `doc` when the docs glob matches it. The doc scorecard checks links and which SDLC
layer a document names. This scorecard checks the instruction-file contract.

| Fact | What it is |
|---|---|
| `canonical` | The filename is `AGENTS.md` |
| `pointsAtAgents` | The body contains `AGENTS.md` |
| `commandLines` | Lines containing `cargo`, `npm`, `pnpm`, `yarn`, `pytest` or `seikika` |
| `lines` | Line count |
| `mentionsDecisions` | The body contains `docs/decisions` (the directory, which also matches the old `docs/decisions.md` path) |
| `ciMissingCommands` | Backtick invocations (`cargo test`, `seikika validate`, `python scripts/…`) whose tool and subcommand do not appear in a workflow. Lines that are markdown table rows are the track index, not the check procedure (D83) |
| `missingFiles` | Relative link targets that are not files |
| `hiddenChars`, `secretHits` | The same hidden-character and high-precision secret checks as skills, on this file only |

## 4. The scorecard

`maintainer-agent-doc`, kind `AgentDoc`. Maintainer SDLC: these files tell agents how to change
this repository.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `single-source` | The file is `AGENTS.md`, or it points at one |
| bronze | `links-resolve` | No missing relative target |
| bronze | `no-hardcoded-secrets` | No secret shape |
| bronze | `no-hidden-instructions` | No hidden or bidi character |
| silver | `has-commands` | A canonical file names a build or test command. An alias does not have to repeat it |
| silver | `within-budget` | At most 500 lines |
| gold | `names-the-decision-log` | A canonical file mentions `docs/decisions`. An alias does not have to |
| gold | `checks-run-in-ci` | A canonical file's backtick check commands appear in a workflow. An alias does not have to repeat them |

## 5. This repository

| File | Level | Notes |
|---|---|---|
| `AGENTS.md` | gold | names `cargo test`, links to `docs/decisions/` |
| `CLAUDE.md` | gold | Five lines. It links to `AGENTS.md` and does not restate the rules |

Fixtures: `agent-doc:agents/demo/focused/AGENTS.md` at gold, `agent-doc:agents/demo/partial/AGENTS.md`
at bronze (600 lines, no command, no decision link), `agent-doc:agents/demo/unsafe/CLAUDE.md` at
none (an alias that does not point at `AGENTS.md`, a broken link, a secret, a hidden character).
Every rule passes on one fixture and fails on another (D71).

## Sources

- agentsmd/agents.md README, the minimal example (dev environment, testing, PR instructions), read
  2026-11-20.
- agentsmd/agents.md issue 135, "AGENTS.md v1.1" proposal: directory jurisdiction, optional
  frontmatter, "aim for under 500 lines". A proposal, not the current format.
- This repository's `AGENTS.md` and `CLAUDE.md`.
