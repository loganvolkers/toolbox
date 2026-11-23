# GitHub Actions Standards: Security, Reliability and Supply Chain for CI

*Status: research and implementation, 2026-11-20 (D70). Toolbox for Toolbox now catalogues its own
workflows and the report action, and scores them with three scorecards. The connectors are generic,
so any portal can apply the same standards to a fleet. Related: [toolbox-for-toolbox.md](./toolbox-for-toolbox.md),
[ratchets.md](./ratchets.md), [repo-portals.md](./repo-portals.md).*

---

## 1. Conclusion

**Recommended scorecard: three cards, built on two linters and the run history.**

| Card | Kind | What it answers | Built on |
|---|---|---|---|
| `maintainer-ci-security` | Workflow (workflows and action metadata) | Can an attacker or a compromised dependency abuse this workflow? | **zizmor** (security audits), **actionlint** (correctness), and Toolbox's own parse (permissions, pinning, risky triggers) |
| `maintainer-ci-reliability` | Workflow | Does green mean good, and does red get fixed? | The **GitHub API**'s run history (first-try fail rate, failing streak, workflow state, failed runs) and the parse (timeouts, hidden failures) |
| `maintainer-ci-speed` | Workflow | Does CI answer quickly, without spending runners on superseded commits? | The parse (concurrency) and the run history (p90 time) |
| `maintainer-action-deps` | ActionDependency | Which third-party actions do we run, where, and are they pinned everywhere? | The parse, joined across workflows |

**What this repository's CI scores:**

| Card | Level | Why |
|---|---|---|
| Security | silver on `ci.yml`; `release.yml`'s template findings are waived | `ci.yml` pins every third-party `uses:` and sets `persist-credentials: false`. What remains is low or informational (`self-repository` on `./actions/report`, which actionlint 1.7.12 rejects as `$/…`, plus `adhoc-packages` and template injection). `release.yml` is dist's bytes (D98). Its high, medium and remaining zizmor findings are waived in `portal/waivers.yaml` (D01M7Y5VWNAWB8K1DNMY31KQPQ0), with the evidence kept. The first measurement, 30 findings, is in the research log |
| Reliability | none where timeouts or concurrency fail; silver where they pass and run history is unknown | `ci.yml` and `release.yml` have no `timeout-minutes` and no concurrency group, so they stay none. `codeql.yml` is bronze (no concurrency). `image.yml`, `pages.yml` and `scale-slo.yml` are silver: their structural rules pass, and the six run-history rules are `unknown` in the fast ratchet, which does not grant gold (D01M7Y5VWKW5TDS0VQWZFPRQ27N). When the GitHub API connector runs, 11 of the last 12 runs on main failed |
| Supply chain | bronze | Every third-party action is pinned to a commit in every workflow. Bronze is this card's only level |

The reliability numbers are not hypothetical. They are why CI was examined at all: acceptance had
never run on GitHub, and the scale budget fails on 2-vCPU runners ([research-log.md](../research-log.md)).

**Key design choices:**
1. **Use the linters; don't re-implement them.**
   - zizmor (42 audits) and actionlint (35+ checks) are maintained, fast, run offline, and output JSON.
   - Toolbox adds what they don't cover: an owned standard with levels, an inventory across files and
     repositories, run history, and a ratchet.
2. **Missing data is `unknown`, not `fail`.**
   - A machine without zizmor reports those rules as unknown, which is never a regression.
   - `--ci` and `--update` refuse an unknown whose `dataFrom` connector was not skipped, so CI
     must have zizmor and actionlint (`dataFrom: ci-lint`).
   - An all-unknown level is not granted. A level with a mix of passes and unknowns can be
     reached (D01M7Y5VWKW5TDS0VQWZFPRQ27N).
3. **Static facts are ratcheted; run history isn't.**
   - Fail rates change with every run, so `github-actions` is skipped in the ratchet. Those rules
     are `dataFrom: github-actions` and score `unknown` there, and real figures in a full report.
   - Security and structure are pure functions of the files, so they are in the ratchet.

## 2. Market: tools for GitHub Actions security and health

*Primary sources were read from each project's repository on 2026-11-20 unless marked. Versions are
the latest tags.*

| Tool | What it is | Coverage | Output / fit for Toolbox | Source |
|---|---|---|---|---|
| **zizmor** 1.32.0 | Static security analysis for GitHub Actions, Dependabot and pre-commit (Rust) | **42 audits**, e.g. template-injection, dangerous-triggers, excessive-permissions, unpinned-uses, artipacked, cache-poisoning, impostor-commit, known-vulnerable-actions, secrets-inherit. Severity and confidence per finding; personas regular, pedantic and auditor; offline mode; safe and unsafe auto-fixes; experimental workflow-lockfile support since 1.32.0 | `--format json` (versioned v1), SARIF, GitHub annotations; exit codes 11–14 by highest severity. **Adopted** | `zizmorcore/zizmor` `docs/audits.md`, `docs/usage.md` |
| **actionlint** 1.7.12 | Workflow linter (Go) | Syntax, expression type checking, contexts, `needs`, matrix, webhook events, cron, runner labels, local action inputs, popular action inputs, script injection, shellcheck and pyflakes integration | `-format '{{json .}}'`; exit 1 on problems. Lints workflows, and checks local actions through the workflows that use them. **Adopted** | `rhysd/actionlint` `docs/checks.md`, `docs/usage.md` |
| **OpenSSF Scorecard** | Repository security posture | 19 default checks, of which Dangerous-Workflow (Critical), Token-Permissions (High), Pinned-Dependencies (Medium; immutable-release tags count as pinned) and CI-Tests are CI-related. "Scans more than 1.3 million repositories every week" | Per repository; REST API and BigQuery. A **complement** (repository-level). Toolbox's rules mirror its CI checks | `ossf/scorecard` `README.md`, `docs/checks.md` |
| **poutine** (BoostSecurity) | Build-pipeline security scanner | Rules: injection, untrusted checkout exec, default permissions on risky events, unverified creators, unpinnable actions, self-hosted runners on PRs, known vulnerabilities in build components | Overlaps zizmor; a candidate second opinion | `boostsecurityio/poutine` `docs/content/en/rules` |
| **ghalint** | Policy linter for workflows and actions | 13 policies: job permissions, deny read-all/write-all, deny inherit secrets, full-length SHA, **job timeout-minutes required**, persist-credentials false, GitHub App token scoping | Narrower; its timeout policy is adopted as a Toolbox rule | `suzuki-shunsuke/ghalint` `docs/policies` |
| **pinact** | Rewrites `uses:` to full SHAs (with the tag in a comment) | Fixing tool, not a check | The `pinned-to-sha` fix | `suzuki-shunsuke/pinact` README |
| **StepSecurity Harden-Runner** | Runtime egress monitoring and blocking on runners | Detected the tj-actions/changed-files compromise (CVE-2025-30066) and a compromised axios release, per its README | Runtime, not static; an optional gold-level control (not adopted: a third-party agent in every job) | `step-security/harden-runner` README |
| **GitHub workflow dependency lockfiles** | `.github/workflows/actions.lock`, generated by `gh actions-lock`; pins every action, including transitive ones, and verifies the source repository's identity at run time | github.com and GHEC, not yet GHES. Doesn't lock reusable workflow calls or composite action definitions | The modern alternative to SHA pinning. **Proposed** as an equivalent of `pinned-to-sha` (§7) | `github/docs` `content/actions/concepts/security/workflow-dependency-lockfiles.md`, `data/features/actions-workflow-lockfiles.yml` |
| **GitHub code scanning (CodeQL for Actions)** | Security queries for workflow files, alerts in the Security tab | Mentioned in GitHub's secure-use guide | Server-side; not runnable offline in a repo portal | `github/docs` `secure-use.md` (query set not verified here) |
| **GitHub Actions performance metrics** | Org and repository dashboards: average run time, queue time, **failure rates** by workflow, job, OS and runner type | Built in; read-only UI | The reliability card uses the same measures from the REST API | `github/docs` `data/reusables/actions/about-actions-performance-metrics.md` |
| CI analytics vendors (Datadog CI Visibility, Trunk, BuildPulse) | Flaky-test detection, pipeline analytics | — | Secondary, not verified; flakiness is approximated here by the re-run rate | — |

**What the landscape says:**
- **Security tooling has converged.** zizmor and actionlint are the de facto open-source pair. GitHub's
  guide recommends SHA pinning or the new lockfiles, CODEOWNERS on workflow files, least-privilege
  tokens, care with `pull_request_target`, and Dependabot for action updates.
- **Every tool scores one repository.** None owns a standard across a fleet, tracks it over time, or
  tells you which repositories use a compromised action. That is Toolbox's slot again, as with
  baselines ([ratchets.md](./ratchets.md) §6).
- **Reliability measures exist in GitHub's own UI but not as standards.** Nobody turns "failure rate"
  into "this workflow is below the bar".

## 3. Best practices the rules encode

| Practice | Source | Rule |
|---|---|---|
| Pin actions to a full-length commit SHA (or lock workflow dependencies) | GitHub secure use, "Using third-party actions"; Scorecard Pinned-Dependencies; zizmor `unpinned-uses` | `pinned-to-sha` (bronze), whose evidence names each unpinned `uses:` and its line, and `pinned-everywhere` (deps), which names the SHA pin the repository already writes elsewhere, version comment included. `zizmor-clean` skips `unpinned-uses`, so the defect fails one rule (D01M84CCDH1ZWF9726SVDDM1JDX, D01M84EPMMR9740K4N25G7A6FAY) |
| Least-privilege `GITHUB_TOKEN`: read-only at the top, writes per job | Scorecard Token-Permissions; zizmor `excessive-permissions`; ghalint deny write-all | `least-privilege-token` (bronze) |
| No untrusted checkout or write tokens under `pull_request_target`, `workflow_run` or `issue_comment` | Scorecard Dangerous-Workflow; GitHub secure use, "untrusted code checkout"; zizmor `dangerous-triggers` | `no-high-findings` (bronze), `no-write-tokens-on-risky-triggers` (silver) |
| No script injection from untrusted inputs | GitHub secure use; zizmor `template-injection`; actionlint script injection check | `no-high-findings` / `actionlint-clean` |
| Don't persist checkout credentials | zizmor `artipacked`; ghalint `checkout_persist_credentials_should_be_false` | `no-medium-findings` (silver) |
| Set `timeout-minutes` on every job (the default is 360) | ghalint `job_timeout_minutes_is_required` | `every-job-has-timeout` (bronze, reliability) |
| Cancel superseded runs | GitHub `concurrency` (secondary: common practice) | `cancels-superseded-runs` (bronze, speed) |
| Keep the default branch green; fix failures before features | GitHub performance metrics expose failure rate per workflow; thresholds are **Toolbox's proposal** (20 / 10 / 5%, 3 failures in a row), not a published standard | `main-not-stuck-red`, `fail-rate-under-*` |
| Treat re-runs as flakiness | Proxy chosen here (secondary: flaky-test vendors measure retries): a run that passed only on a re-run counts as failed in the fail rate | `fail-rate-under-*` (first-try rate) |
| Fast feedback | Toolbox's proposal: p90 ≤ 15 minutes | `fast-feedback` (silver, speed) |

## 4. How it works

| Connector | New | Records |
|---|---|---|
| `workspace` | `workflows` globs: parses workflows and action metadata with serde (no linter) | `workflow`: triggers, risky triggers, top-level and per-job permissions, write scopes, jobs without a timeout, concurrency, every `uses:` with its pin (`sha`, `tag`, `branch`, `local`, `docker-tag`, `docker-digest`). `action-ref`: per external action, every ref used and the files using it |
| `runner` | `parse: zizmor` and `parse: actionlint`, with `files` globs and `optional` | One record per covered file (clean files included): findings with rule, severity and 1-based line, and counts per severity. An optional tool that isn't installed is skipped and logged |
| `github` | `actions: {repos, branch, runs}` | `workflow-runs` per workflow: `runs::stats` over the last N runs on the branch. A run counts as failed when its conclusion is failure, timed_out or startup_failure; cancelled and skipped runs are ignored |

**Determinism.** zizmor runs `--offline`, and actionlint runs with `-shellcheck= -pyflakes=`: the CI
image has shellcheck and a laptop may not, and enabling it on one side only would change results.
Both are pinned (1.32.0, 1.7.12) in CI and in the docs, and `portal/results.json` was made with those
versions.

**Unknown, not fail.** zizmor and actionlint rules reference the linter's record without a guard,
and they set `dataFrom: ci-lint`. When the tool is missing, the reference fails, the rule is
`unknown`, and the ratchet treats that as "not compared":
- the default mode reports it;
- `--ci` and `--update` refuse it, because CI does not skip `ci-lint`.

Run-history rules set `dataFrom: github-actions` and read `entity.runs` without a `has()` guard.
In the fast ratchet, where that connector is skipped, they are `unknown` ("no data from connector
github-actions"). That unknown is excused, so `--ci` can update the baseline. A level whose rules
are all unknown is not granted.

## 5. Verification

- **Unit tests:**
  - zizmor and actionlint parsers, using the documented JSON shapes;
  - workflow parsing: pins, risky triggers, permissions, reusable-workflow jobs;
  - run statistics, including this repository's real history (92% failing, an 11-run streak);
  - cancelled runs and re-runs;
  - unknown outcomes in the ratchet.
- **Live:**
  - on this repository, zizmor through the runner reproduces zizmor's direct output (27 findings in
    `ci.yml` and 3 in `action.yml`, 30 in total);
  - actionlint reports 0 errors;
  - `workflow-runs` passes end to end against the mock GitHub API (two new endpoints in
    `tools/mock_github.py`).
- **`toolbox test portal`:** 31/31. Nine of the expectations are new and cover each card, including
  unknown-not-fail and the supply-chain view.
- **The ratchet:**
  - with both tools installed, `--ci` matches the committed baseline;
  - without them, the default mode passes with 7 unknown rules reported, and `--ci` and `--update`
    refuse.
- **Not verified here:**
  - the `github-actions` connector against the real API (no direct API access from this session);
    its statistics are computed from the run data read through GitHub's MCP tools;
  - CodeQL for Actions.

## 6. Generalising to a fleet

The three cards need nothing Toolbox-specific: point `workspace` (with `workflows`), `ci-lint` and
`github` (with `actions`) at every repository, and the portal answers fleet questions:
- "which repositories use `tj-actions/changed-files`, and at which refs?" (ActionDependency `usedBy`
  across repos: the incident question behind CVE-2025-30066);
- "which workflows run `pull_request_target` with write tokens?";
- "which teams' main branches have been red for 3+ runs?".

In the repo-portal starter, these become a natural stage-2 standard under federation's graduation
path ([federation.md](./federation.md) §6).

## 7. Open items

| # | Item | Proposal |
|---|---|---|
| G1 | Workflow lockfiles | Detect `.github/workflows/actions.lock`, and count workflows onboarded to it as pinned. zizmor's own support is experimental |
| G2 | CODEOWNERS for `.github/workflows` | Repo-level rule (GitHub secure use), in `repo-hygiene` when a portal has a Repository entity |
| G3 | Dependabot for the `github-actions` ecosystem | Repo-level rule (Scorecard Dependency-Update-Tool); zizmor's `dependabot-cooldown` audit covers its configuration |
| G4 | shellcheck | Enable it on both sides (install it locally, or run the linters only in CI) to regain its checks |
| G5 | Job-level history | `workflow-runs` is per workflow. Per-job failure rates would name the flaky job, which the GitHub API supports via the runs' jobs |
| G6 | Thresholds | The fail-rate, re-run and p90 thresholds are proposals. Revisit with fleet data |

## Sources

- zizmor: `zizmorcore/zizmor` (`README.md`, `docs/audits.md`, `docs/usage.md`), tag v1.32.0; read and
  run 2026-11-20.
- actionlint: `rhysd/actionlint` (`docs/checks.md`, `docs/usage.md`, `error.go`), tag v1.7.12; read
  and run 2026-11-20.
- OpenSSF Scorecard: `ossf/scorecard` `README.md`, `docs/checks.md`.
- GitHub docs (`github/docs`):
  - `content/actions/reference/security/secure-use.md`;
  - `content/actions/concepts/security/workflow-dependency-lockfiles.md`;
  - `data/features/actions-workflow-lockfiles.yml`;
  - `content/actions/concepts/metrics.md`;
  - `data/reusables/actions/about-actions-performance-metrics.md`.
- poutine (`boostsecurityio/poutine` rules), ghalint (`suzuki-shunsuke/ghalint` policies), pinact
  (`suzuki-shunsuke/pinact`), Harden-Runner (`step-security/harden-runner`) READMEs.
- This repository's workflow runs on `main`, via GitHub's MCP tools, 2026-11-20.
