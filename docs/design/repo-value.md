# Proving Repo-Level Value

*Status: plan, 2026-11-21 (D01M7Y46VDDBK2SW6H96ZSG89MP). All effort goes here until the exit in §6 is met. This is the
operator SDLC applied to one repository (a [repo portal](./repo-portals.md)), built under the
maintainer SDLC, with packages from the package author SDLC ([sdlc.md](../sdlc.md)).*

---

## 1. Conclusion

**We prove that one repository gets enough from Toolbox to keep it, before we build anything that
governs many.** A repo portal is a `toolbox.yaml`, a committed baseline and one CI step
([repo-portals.md](./repo-portals.md)). If maintainers and their coding agents get value from it,
federation is the same thing at fleet scale ([federation.md](./federation.md)). If they don't, a
central portal won't fix that.

The bet has five parts. Each has an experiment and a result that would stop us (§3).

| # | Hypothesis |
|---|---|
| R1 | Adopting Toolbox in a repository takes one pull request and no infrastructure |
| R2 | Its findings are true and worth fixing |
| R3 | Coding agents raise a repository's standards further with Toolbox's evidence and ratchet than with the same standards written down |
| R4 | The ratchet catches real regressions on real pull requests without becoming a chore |
| R5 | Standards travel between repositories as packages, with parameters and no edits |

## 2. What is paused and what continues

| Area | State |
|---|---|
| Central portal: Cloud Run deployment, `plan --remote`, preview, `verify` (M6) | **Paused** |
| Self-service actions (M5, already paused by D64) | **Paused** |
| Federation, including static federation ([federation.md](./federation.md), [repo-portals.md](./repo-portals.md) §5) | **Paused** until §6 |
| Apps built on a portal's API (user SDLC beyond reading a report) | **Paused** |
| `serve`, REST, MCP, UI, OpenFGA, Postgres (M2–M4) | **Kept green, not extended.** CI still runs their tests and acceptance scripts; bugs are fixed. `serve` and `mcp` on a laptop remain part of the repo-level agent loop |
| `toolbox report`, the ratchet and verdict, `actions/report`, the starter, packages, assessments, campaigns | **Focus** |

## 3. Hypotheses, experiments and stop conditions

| # | Experiment | Measure | Continue if | Stop or rethink if |
|---|---|---|---|---|
| R1 | Install on each installed pilot (§4) from the starter, from scratch | Steps and edits to the starter; CI time added on a cold and a warm run; questions the maintainer had to ask | One pull request; no edits beyond parameters; warm overhead small enough that nobody asks to turn it off | A pilot needs local changes to Toolbox, or its maintainer removes the step |
| R2 | Out-of-box runs on repositories we didn't write (§4, read-only), then triage of every failing rule | Each failure as true and worth fixing, true but not worth it, or false | At least 80% true, and at least half of those worth fixing | More than 20% false: the standards, not adoption, are the problem |
| R3 | A/B on the installed pilots: the same coding agent and budget, asked to raise the repository's standards. Arm A gets the standards as a written checklist; arm B gets `toolbox report` with evidence and the ratchet | Rules newly passing (scored afterwards by Toolbox and spot-checked by a person), regressions introduced, tokens | Arm B fixes at least 15 percentage points more at equal tokens (the bar E2 sets in [adversarial-critique.md](../adversarial-critique.md) §10) | No difference: the value is the standards text, so ship packages of written standards instead of a tool |
| R4 | The installed pilots keep the action on for their normal pull requests | Real regressions caught; intended changes flagged as worse; pull requests that had to commit a baseline update | At least one real regression caught, and maintainers keep the step | Maintainers disable it or routinely `--update` past it |
| R5 | Every pilot uses the same packages (`packages/github-actions` and the ones §5 adds) | Local overrides and forks per pilot | Parameters only (`with:`) | Each pilot forks a package: federation's package model needs rethinking first |

## 4. Pilots

| Kind | Repositories | Proves | What it changes |
|---|---|---|---|
| **Out-of-box, read-only** | Public projects in several languages, for example the account's forks of OpenFGA (Go), Codex (Rust and TypeScript), PostHog (Python and TypeScript) and gh-aw (Go) | R2, language coverage | Nothing: runs happen in this session, and results go to the research log |
| **Installed** | Active repositories with maintainers, for example `json-editor/json-editor` (a public JavaScript library) and the account's private projects | R1, R3, R4, R5 | A pull request adding `.toolbox/` and one workflow; needs each owner's agreement |

At least one installed pilot must have maintainers other than us.

## 5. Gaps to close before the pilots

| Gap | Why it matters | Work |
|---|---|---|
| Missing data scored as failure | `maintainer-crate` shows 13 of 15 crates at `none` partly because the ratchet skips the runner, so `tests-pass` and `clippy-clean` read as failures. On pilots this would count as false positives under R2 | Rules with no data are `unknown`, reported and never compared |
| Cold start | The action builds Toolbox from source on a pilot's first run | Prebuilt binaries for pilots ([package-author-sdlc.md](./package-author-sdlc.md) §3); needs a pre-release |
| A thin starter | It checks a README, CODEOWNERS, a licence and doc links | Package the standards in Toolbox for Toolbox that aren't specific to Toolbox: agent instruction files, supply-chain basics, npm manifests, docs. Each is tested in its own `tests/` portal, as `packages/github-actions` is |
| Language coverage | The workspace connector knows Rust crates, npm manifests and Markdown | Measured by the out-of-box runs; add what R2's triage shows is missing |
| Pull request experience | Annotations and a job summary, but no published report | Publish the one-page report with each run; a summary comment is optional |

## 6. Exit

**Repo-level value is proven when R1 to R5 all continue on at least three installed pilots, one of
them maintained by people other than us.** Federation (D68) then resumes, starting from the packages
that R5 showed travel.

## 7. Order of work

1. Missing data as `unknown` (§5).
2. Out-of-box runs and their triage (R2).
3. Package the generic standards (§5, R5).
4. Pre-release binaries for pilots, if agreed.
5. Install the pilots (R1, R4).
6. The A/B (R3).
7. Review the pilot period (R4, R5) and decide on federation.

## Sources

- [repo-portals.md](./repo-portals.md), [federation.md](./federation.md), [packages.md](./packages.md)
- [adversarial-critique.md](../adversarial-critique.md) §10: kill criteria E1 and E2
- [toolbox-for-toolbox.md](./toolbox-for-toolbox.md): the first repo portal
