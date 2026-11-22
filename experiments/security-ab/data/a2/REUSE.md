<!-- Saved by the experiment's adjudicator, verbatim from the A2 agent's hand-back (see ../../prompts.md, Deviations). -->
# REUSE.md

Ran unchanged: yes (no crash, 25 s, overall 5.1), but several results were wrong for this repository. 12 general fixes to `scorecard.py`; overall now 6.4 / 10. The regression suite still passes 11/11, and the empty fixture still runs without crashing.

| Check | Before | After | What drives it now |
|---|---|---|---|
| ci-workflow-audit | 1.1 | 1.4 | 19 zizmor template-injection (high), 9 github-env (high severity, low confidence), dangerous triggers in `cla.yml`, `secrets: inherit` |
| workflow-permissions | 1.0 | 4.5 | 10 workflows with undeclared token permissions; write scopes on `cla.yml` (pull_request_target) and two `issues` workflows |
| dependency-pinning | 9.8 | 9.8 | 3 base images not pinned by digest |
| secrets | 4.4 | 7.9 | only test RSA keys and test tokens |
| static-analysis | 4.5 | 7.5 | ruff subprocess and `tarfile.extractall` in Python scripts |
| dependency-management | 8.9 | 7.4 | npm, pip and docker not covered by Dependabot; 2 dead Dependabot entries |
| known-vulnerabilities | null | null | needs an advisory DB |
| security-tooling-in-ci | 2.0 | 2.5 | only cargo-deny; no SAST, secret scanning or CI linting |
| container-hardening | 5.3 | 8.0 | `curl \| sh` in dev-container images; no `.dockerignore` |
| security-policy | 10.0 | 9.0 | CODEOWNERS doesn't cover `.github/workflows` |
| release-integrity | 4.0 | 6.0 | cosign signing; provenance only for PyPI and npm; no SBOM |

## Changes to the scorecard, and why

1. **zizmor audits only workflows that run.** It scanned `.`, and so audited `codex-rs/vendor/bubblewrap/.github/workflows/check.yml`, which GitHub never runs. Results are now kept only for root `.github/workflows/*`, in-repo `action.yml` files (outside vendor and skipped directories) and the Dependabot config.
2. **Unknown runner labels no longer cost points.** actionlint's `runner-label` flags larger or self-hosted runner labels such as `codex-linux-x64`, which isn't a security problem. The finding stays, at weight 0.
3. **Missing workflow permissions are judged properly.** "No top-level `permissions:`" is flagged only when some job also sets none, and never for `workflow_call`-only reusable workflows, which get their token from the caller. The message names the jobs that set none. Before, it flagged `issue-labeler`, `issue-deduplicator` and `python-sdk-release`, where every job sets its own permissions, and three reusable workflows. zizmor's matching "no permissions: block" findings are no longer double-counted for the same file, and the finding counts at most 5 times, like the other rules.
4. **Write scopes on untrusted triggers are weighted by impact.** `issues`, `pull-requests`, `statuses`, `checks` and `discussions` write weigh .08. Scopes that can push code, change CI or publish (`contents`, `actions`, `id-token`, …) keep .20. The old message claimed "repository takeover" for an issue labeller with `issues: write`.
5. **Rust test code is recognised.** Test file names (`*_test.rs`, `*_tests.rs`, `test.rs`, `tests.rs`) and inline `#[cfg(test)] mod … {` blocks, from there to the end of the file, count as test code in both the secrets and static-analysis checks. Before, test RSA keys and `user:pass@` URLs in Rust unit tests were scored as production secrets, and semgrep hits in `shell_snapshot_tests.rs` as production code.
6. **Noisy bandit rules weigh less.** ruff S105–S107, S108, S310 and S606 join the low-weight set. They fire on any `urlopen`, any `/tmp` path, or constants like `ARGS_TOKEN = "..."`, and S105–S107 duplicate the secrets check.
7. **Composite actions count as CI.** Local `action.yml` files now count when the tooling and release checks look for tools; Codex's cosign signing, for example, lives in `.github/actions/linux-code-sign`.
8. **Container scanning is required only when CI builds images.** It was required whenever any Dockerfile existed. Now it needs a Dockerfile outside `.devcontainer/` and an image build in CI (`docker build`, buildx, build-push-action, ko, jib, kaniko, goreleaser `dockers:`). Codex has dev-container and build-environment Dockerfiles but publishes no images.
9. **Dev-container images weigh less.** Findings in `.devcontainer/` Dockerfiles are weighted ×0.3, like test code, and labelled: they are developer environments, not shipped images.
10. **CODEOWNERS must cover the workflows.** Any `.github…` rule used to earn the 2 points, so `/.github/CODEOWNERS` alone was enough. Now covering `.github/` or `.github/workflows` earns 1, and covering the CODEOWNERS file itself earns 1.
11. **Release provenance from trusted publishing is credited.** `pypa/gh-action-pypi-publish`, or `npm publish` in a workflow with `id-token: write`, produces registry provenance attestations. That earns 2 of 4 points when GitHub release binaries are also published without attestations, and 4 otherwise.
12. **Dependabot entries that update nothing no longer count.** The parser now reads `directories:` lists, including entries without a leading `/`, and an entry counts only if one of its directories holds a manifest for its ecosystem. Here the `bun` entry points at a missing directory, and it was the only thing "covering" npm. The `docker` entry points at `codex-cli/`, which has no Dockerfile. npm and docker are now correctly reported as uncovered, along with pip, which was already reported.

## Results that may still be wrong

- **ci-workflow-audit (1.4) is probably too harsh.** Most template-injection hits put `${{ inputs.* }}` or `matrix.*` into `run:` inside `workflow_call` workflows, whose in-repo callers likely pass constants. github-env hits are in composite actions, where zizmor has low confidence. The scorecard trusts zizmor's severity and confidence; I didn't recalibrate them.
- **actionlint `expression` errors are false positives.** `runs-on:` gets a `{group, labels}` object from the matrix, which is valid but which actionlint rejects. They still cost .03 each.
- **The Rust test-module rule assumes the test module ends the file.** Any code after it is treated as test code.
- **Trusted-publishing provenance assumes versions I can't check offline:** npm 11.5 or later, and pypi-publish 1.11 or later (the pinned SHA is commented `v1.13.0`).
- **Some settings are invisible from the tree.** The org's default token permissions, GitHub code-scanning default setup, and secret scanning or push protection could all change workflow-permissions and security-tooling-in-ci. If the org default token is read-only, the missing-permissions findings overstate the risk.
- **Secrets (7.9) still loses about 2 points** for two RSA keys in test code. That is by design: test findings weigh less but are not ignored.
- **known-vulnerabilities stays null.** `Cargo.lock` could be checked with `cargo audit` given an advisory database.
- **`.dockerignore` for `Dockerfile.bazel` may be a false finding.** Its build context is unknown.
