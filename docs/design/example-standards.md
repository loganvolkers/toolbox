# Example Standards: Cataloguing the Write-ups Agents Copy

*Status: deterministic catalog and scorecard built, 2026-11-20 (D78). Toolbox for Toolbox catalogues
each example and experiment write-up and scores it with `maintainer-example`. The reference
portal's `expected/` outputs are not regenerated. Related:
[schema-standards.md](./schema-standards.md),
[toolbox-for-toolbox.md](./toolbox-for-toolbox.md).*

---

## 1. Conclusion

**An example is a procedure someone can follow, not a second copy of the design.**

The reference portal is the oracle for v1: `tools/validate.py`, `tools/reference.py --check` and
`fga model test` are the checks, and `expected/` is what they must match. Experiments record a
question, a command and a result. A specification that names no command is still a document, and
it stops below the level of a write-up a reader can run. The checks that do not need to run the
example:

| Question | What answers it | In this repo |
|---|---|---|
| Does it have a heading, and do its relative links exist? | The first heading, the link targets | bronze |
| Does it name a command that checks it? | `cargo`, `npm`, `python`, `fga`, `toolbox validate`/`test`/`report`, or `./run` | silver |
| Does it point at the design or a decision, and is there no secret in it? | `docs/`, `decisions.md`, a `D` number, secret shapes | gold |

The product name "Toolbox" in a sentence is not a command. A directory link counts when the
directory exists.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **This repository's reference portal** | Correctness for v1 is reproducing `expected/` via the three checks named in its README | The README is gold: it names `toolbox validate`, `tools/reference.py --check` and `fga model test`, and its links resolve. The outputs are left byte for byte |
| **The experiments** | Each README states a question, a command and the result, and points at the design doc that cites it | A command line and a design link are the silver and gold rules |
| **Worked examples** | `docs/examples/approved-github-apps.md` is the narrative; the runnable form is the reference portal | The narrative links at the portal and at the design, so it reaches gold. The executor specification names no check command, so it stays bronze |

## 3. The catalog

Globs: `examples/**/README.md`, `experiments/**/README.md`, `docs/examples/*.md`. Id
`example:<path>`. The same file is also a `doc` when the docs glob matches it.

| Fact | What it is |
|---|---|
| `title`, `lines` | The level-1 heading and the length |
| `missingFiles` | Relative link targets that are neither a file nor a directory |
| `commandLines` | Lines that invoke `cargo`, `npm`, `pnpm`, `yarn`, `pytest`, `python`, `fga`, `toolbox` with a subcommand, or `./run` |
| `citesDesign` | The body contains `docs/`, `decisions.md`, or a decision id such as `D40` |
| `secretHits` | High-precision secret shapes |

## 4. The scorecard

`maintainer-example`, kind `Example`. Maintainer SDLC: these write-ups are how this repository
shows its own contracts.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `has-a-heading` | The file starts with a level-1 heading |
| bronze | `links-resolve` | No missing relative target |
| silver | `says-how-to-check` | At least one command line |
| gold | `cites-the-design` | The body points at a design doc or a decision |
| gold | `no-hardcoded-secrets` | No secret shape |

## 5. This repository

| Write-up | Level | Notes |
|---|---|---|
| `examples/reference-portal/README.md` | gold | Names the three checks and links `expected/` |
| `examples/repo-portal-starter/README.md` | gold | Names `toolbox report` and `toolbox test` |
| `examples/reference-portal/ui/install-approved-app/README.md` | gold | Names `npm run build` and links the spec |
| `examples/reference-portal/executors/github-apps/README.md` | bronze | Links resolve and it cites D40. It names no command that checks the executor |
| `experiments/README.md` and the four experiment READMEs | gold | Each names a run command and a design doc |
| `docs/examples/approved-github-apps.md` | gold | The narrative points at the reference portal and the design |

Fixtures: `example:examples/demo/focused/README.md` at gold,
`example:examples/demo/partial/README.md` at bronze (no command),
`example:examples/demo/unsafe/README.md` at none (no heading, a broken link, no design citation, a
secret). Every rule passes on one fixture and fails on another (D71).

## Sources

- `examples/reference-portal/README.md`, the three checks and `expected/`.
- `experiments/README.md` and the experiment READMEs, each with a run command.
- `docs/examples/approved-github-apps.md`, the worked example whose runnable form is the reference portal.
