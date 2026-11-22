<!-- Saved by the experiment's adjudicator, verbatim from the A1 agent's hand-back: the harness blocked the agent's own write of this file. -->
# Security Scorecard — Report

**Result for `repo` (OpenFGA, Go): 7.9 / 10** (mean of 10 evaluated
checks; `known-vulnerabilities` is null offline).

**Top finding:** `.github/workflows/claude-code-review.yaml` triggers on
`issue_comment` and `pull_request_review_comment`, which any GitHub user
can fire on a public repo. It calls a third-party reusable workflow
pinned to the mutable `@main` (`auth0/auth0-ai-pr-analyzer-gh-action`),
and grants it `contents`, `issues` and `pull-requests` write plus
`id-token: write`. Anyone who can push to that upstream branch gets a
write token and an OIDC token in this repository.

Usage: `scorecard/run <repo-path>` prints JSON;
`scorecard/tests/test.sh` runs the regression tests.

## Checks and why

Each check scores 0–10 (10 = nothing found; null = nothing applicable).
Most use decay scoring: each finding removes a share of the remaining
score, weighted by severity, with at most 5–10 findings per rule
counted. One noisy rule can't sink a check, and a few serious findings
still hurt.

| Check                    | Tool                                                             | Why it matters                                                                                                        | Score here |
| ------------------------ | ---------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------- |
| `ci-workflow-audit`      | zizmor, actionlint, custom "pwn request" rule                    | CI holds release credentials; injection, dangerous triggers and cache poisoning are the main supply-chain entry point | 4.2        |
| `workflow-permissions`   | custom + zizmor                                                  | Least-privilege `GITHUB_TOKEN` limits blast radius; write scopes on untrusted triggers are the worst case             | 7.1        |
| `dependency-pinning`     | custom                                                           | Actions, reusable workflows and base images pinned to SHA or digest; tags can be moved (tj-actions)                   | 9.9        |
| `secrets`                | custom regexes + entropy                                         | Committed keys and tokens, credential files, credentials in URLs; test paths down-weighted                            | 9.6        |
| `static-analysis`        | semgrep (20 local rules: Go/JS/TS/Rust/shell), ruff `S` (Python) | TLS verification off, string-built SQL, shell injection, servers without timeouts, XSS sinks                          | 7.2        |
| `dependency-management`  | custom + zizmor                                                  | Lockfiles, Dependabot/Renovate covering every ecosystem, stale update config                                          | 8.4        |
| `known-vulnerabilities`  | —                                                                | Needs an advisory database; returns null rather than guessing                                                         | null       |
| `security-tooling-in-ci` | custom                                                           | Continuous SAST, dependency CVE scanning, secret scanning, CI linting, image scanning                                 | 8.0        |
| `container-hardening`    | custom                                                           | Non-root final stage, no `ADD <url>` or `curl \| sh`, no secrets in ENV/ARG, `.dockerignore`, no privileged compose   | 10.0       |
| `security-policy`        | custom                                                           | SECURITY.md disclosure channel; CODEOWNERS covering `.github/`                                                        | 5.0        |
| `release-integrity`      | custom                                                           | Provenance (SLSA), signing (cosign), SBOM                                                                             | 10.0       |

These cover how repositories actually get compromised: through CI and
the supply chain (the first three checks and dependencies), through code
(static analysis and secrets), through what ships (containers and
releases), and through how reports are handled (policy). Everything is
judged from files in the tree, so it works offline and on any repo.

## Most important findings

1. **Write token on a comment trigger, with a third-party workflow pinned to `@main`**
   (`claude-code-review.yaml:2-18`, described above). Flagged by three
   checks: zizmor `dangerous-triggers`, the untrusted-trigger-with-write
   rule, and the only real pinning failure.
2. **zizmor template injection (high):** three hits in
   `create-release-tag.yaml:27,62` and `create-helm-release-pr.yaml:27`,
   interpolating `pull_request.user.login` and `merge_commit_sha` into
   `run:`. Neither value is meaningfully attacker-controlled, so the
   practical risk is low. They should still move to `env:`, because
   those workflows hold `GORELEASER_GITHUB_TOKEN`.
3. **Cache poisoning (high, low confidence):** goreleaser runs in
   release workflows with `setup-go` caching enabled (`release.yaml`,
   `release-nightly.yaml`, `pull_request.yaml:183`).
4. **No Slowloris protection:** four `http.Server`s in `cmd/run/run.go`
   (784, 898, 1012, 1033) set no `ReadHeaderTimeout`: the HTTP API,
   playground, pprof and metrics servers.
5. **`InsecureSkipVerify: true` for the unix-socket gRPC dial**
   (`cmd/run/run.go:699`). It is justified in a code comment and worth a
   `nosemgrep` with a reason. The TCP variant verifies manually and is
   correctly not flagged.
6. **Hygiene:** artipacked (21 `actions/checkout` steps without
   `persist-credentials: false`); Dependabot has no cooldown, and its
   `/tools` gomod entry points at a missing directory, so it updates
   nothing; no secret scanning in CI; no SECURITY.md in the repo (an
   org-level policy may exist but isn't visible).

**Strengths:** nearly all actions and every image are SHA- or
digest-pinned. The runtime image is distroless and non-root (Chainguard
static). Releases have cosign signing, syft SBOMs and SLSA provenance.
CI runs CodeQL, govulncheck, Snyk and OSSF Scorecard. No real secrets
were found: the 8 secrets findings are `postgres:password@` dev
defaults, with negligible weight.

**Calibration:** `slsa-github-generator` is exempt from SHA pinning,
since its docs require a tag reference. `InsecureSkipVerify` together
with `VerifyPeerCertificate` isn't flagged. zizmor runs with
`--no-config`, so every repo is judged by the same standard; inline
`# zizmor: ignore` still applies. Regression tests: `tests/fixtures/bad`
is a planted-problem repo, and `tests/test.sh` asserts every check fires
on it (11/11 pass). An empty repo yields only nulls and zeros, with no
crash.

## With more time

- **Known vulnerabilities:** `govulncheck`, `osv-scanner` or
  `cargo audit` against a mirrored advisory DB. Go analysis was blocked
  here: local Go is 1.24, the repo needs ≥1.25, and there is no network.
- **Remote state** via the GitHub API: branch protection and rulesets,
  required reviews, the default `GITHUB_TOKEN` permission, whether
  secret scanning and push protection are on, the org SECURITY.md.
- **Deeper workflow semantics:** gate on whether security jobs block
  (Snyk here is `continue-on-error: true`, release-only); follow
  reusable workflows across repositories.
- **Git history:** secret scanning across all commits, not just the
  tree; signed commits and tags.
- **More rules:** gosec-equivalent semgrep coverage; Terraform and
  Kubernetes manifests (privileged pods, public buckets); Java and Ruby
  rules.
- **Calibration:** tune weights on a corpus of known-good and known-bad
  repos; SARIF output for code scanning.
