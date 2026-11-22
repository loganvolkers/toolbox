<!-- Saved by the experiment's adjudicator, verbatim from the A3 agent's hand-back: the harness blocked the agent's own write of this file. -->
# Security Scorecard — Report

**Result: 5.6 / 10 overall across 16 checks.** CI is the weakest area:
broad workflow token permissions, and AI agents fed attacker-controlled
issue text. Basics are strong: every action pinned to a commit, all
manifests locked, no live secrets found.

## Checks and why
Each check starts at 10 and loses points per finding by severity, capped
per rule so one noisy rule can't zero a check. Overall = weighted mean
of applicable checks (weight 3 = direct compromise path, 2 = major
control, 1 = hygiene).

| Check                  | Weight | Score | What it judges, and why                                                                                                              |
| ---------------------- | ------ | ----- | ------------------------------------------------------------------------------------------------------------------------------------ |
| ci-workflow-audit      | 3      | 0.0   | zizmor over GitHub-executed workflows; CI holds signing and publishing secrets                                                       |
| secrets                | 3      | 9.0   | Token, private-key and password patterns, plus key stores and `.env` files; test-path hits capped at 1 point                         |
| action-pinning         | 2      | 10.0  | Share of `uses:` pinned to a 40-character commit SHA, since tags are mutable                                                         |
| workflow-permissions   | 2      | 3.8   | Share of workflows that explicitly limit the `GITHUB_TOKEN`                                                                          |
| ai-agent-workflows     | 2      | 2.0   | AI agents run on issues or PRs: attacker text in prompts, `allow-users: "*"`, write permissions. No existing tool covers this        |
| dependency-lockfiles   | 2      | 10.0  | Every manifest has a committed lockfile                                                                                              |
| security-tooling-in-ci | 2      | 3.2   | CI runs dependency audit, SAST, a workflow linter and a secret scanner; dead workflows flagged                                       |
| sast                   | 2      | 8.5   | 18 local semgrep rules (JS/TS, Rust, Go, shell, Dockerfiles): TLS verification off, eval, shell strings, formatted SQL, `curl \| sh` |
| release-integrity      | 2      | 6.0   | Release workflows sign artifacts, attest provenance, publish via OIDC and ship checksums                                             |
| dependency-updates     | 1.5    | 3.0   | Dependabot or Renovate covers each manifest directory; entries that update nothing are penalised                                     |
| dependency-sources     | 1      | 6.1   | Lockfile entries from git, non-default registries or plain http                                                                      |
| workflow-lint          | 1      | 8.5   | actionlint correctness; custom runner labels reported, not scored                                                                    |
| policy-and-ownership   | 1      | 7.0   | `SECURITY.md` with a reporting channel; CODEOWNERS covering CI                                                                       |
| python-security-lint   | 1      | 6.2   | ruff bandit rules (`S`) on non-test Python                                                                                           |
| rust-unsafe            | 1      | 0.9   | Share of `unsafe` sites with a `// SAFETY:` justification                                                                            |
| container-hardening    | 1      | 7.0   | Digest-pinned base images, non-root `USER`, no `ADD <url>`, no secrets in `ENV`/`ARG`                                                |

## Most important findings
1. **Prompt injection into CI agents.** `issue-labeler.yml` lines 85 and
   88 interpolate the issue title and body directly into an
   `openai/codex-action` prompt. It sets `allow-users: "*"`, and the
   agent's output feeds a job with `issues: write`.
   `issue-deduplicator.yml` has the same exposure through fetched issue
   text. The agent is read-only and drops sudo, but anyone can still
   steer labelling and duplicate detection.
2. **Workflow audit: 112 zizmor findings.** `cla.yml` runs an archived,
   unmaintained third-party action on
   `pull_request_target`/`issue_comment` with write access to contents,
   actions, pull requests and statuses; it doesn't check out PR code. 13
   of 21 workflows have no `permissions:` block. Six calls pass
   `secrets: inherit`. Two cache-poisoning risks are in
   `rust-release.yml`. The 19 template-injection hits mostly come from
   `inputs.*` and `matrix.*` in reusable workflows: real, but hard to
   exploit without a malicious caller.
3. **A security workflow that never runs.**
   `codex-rs/.github/workflows/cargo-audit.yml` sits in a nested
   `.github` directory, which GitHub ignores (cargo-deny does run from
   the root workflows). Apart from that, CI has no SAST (only clippy),
   no workflow linter (zizmor or actionlint) and no secret scanner.
   GitHub's built-in secret scanning can't be verified from a checkout.
4. **Dependabot gaps.** npm in 5 manifest directories gets no updates,
   because its only entry points at `.github/actions/codex`, which
   doesn't exist. Python (`scripts`, `sdk/python`, `sdk/python-runtime`)
   gets no updates. The Docker entry targets `codex-cli`, which has no
   Dockerfile.
5. **Unexplained `unsafe` in Rust.** 501 of 548 non-test `unsafe` sites
   have no SAFETY comment, mostly libc and Win32 FFI in the sandbox
   crates, which are the product's security boundary. The clippy lint
   `undocumented_unsafe_blocks` is off.
6. **Smaller gaps.** Releases are signed with cosign and the Python SDK
   publishes over OIDC, but there is no build-provenance attestation.
   CODEOWNERS doesn't cover `.github/workflows`. 12 crates come from git
   forks (crossterm, ratatui, tokio-tungstenite and others), outside
   crates.io advisories. The devcontainer Dockerfiles pipe `curl` into a
   shell and use base images not pinned by digest.

## What I would add with more time
- Taint-aware CI scoring: follow reusable-workflow `inputs` back to
  their callers, so a template injection is scored by whether an
  attacker controls the value.
- Offline vulnerability matching of `Cargo.lock`, `pnpm-lock.yaml` and
  `uv.lock` against a vendored RustSec and OSV snapshot.
- Repository settings through the GitHub API: branch protection,
  required reviews, signed commits, built-in secret scanning, and
  release environment protection.
- Agent hardening beyond prompts: an agent's tool and network sandbox,
  and whether its output is validated before a step that writes.
- A per-repository config for weights, ignores and runner labels; SARIF
  output; weights calibrated on a corpus of repositories so scores
  compare across teams.
