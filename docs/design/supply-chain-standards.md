# Supply-Chain Standards: The Files That Turn GitHub Advanced Security On

*Status: deterministic catalog and scorecard built, 2026-11-20 (D85). Toolbox for Toolbox scores the
repository record with `maintainer-supply-chain`. Alert counts are not a gate. Related:
[github-actions-standards.md](./github-actions-standards.md),
[repo-level.md](./repo-level.md),
[toolbox-for-toolbox.md](./toolbox-for-toolbox.md).*

---

## 1. Conclusion

**GitHub Advanced Security is on when the files that enable it exist.**

The maintainer SDLC already scores workflow text with zizmor and actionlint (D70). Dependabot,
code scanning and a security policy are files in the checkout, so they can be scored without the
GitHub API. Counts of alerts need that API and stay unknown.

| Question | What answers it | In this repo |
|---|---|---|
| Are Dependabot, code scanning and a security policy configured? | `dependabot.yml` for cargo, npm and github-actions; a workflow that uses `github/codeql-action`; `SECURITY.md` | bronze |
| Does someone own the CI workflow? | The last CODEOWNERS rule that matches `.github/workflows/ci.yml` | silver |
| Is the code-scanning action pinned to a commit? | `github/codeql-action/…@` plus 40 hex digits | gold |

This checkout now has the bronze files, a CODEOWNERS rule for `.github/`, and
`github/codeql-action` pinned to a commit, so the live entity is gold.
D01M7Y5VWNAWB8K1DNMY31KQPQ0 adopts those files as the standard. D85 left the entity at none until
they existed, and is not rewritten.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **GitHub Dependabot** | `package-ecosystem` entries in `.github/dependabot.yml` | The three ecosystems this repo builds with: cargo, npm, github-actions |
| **GitHub code scanning** | A workflow that uses `github/codeql-action` | Presence of that action. Pinning it to a commit is gold, the same shape as other action pins (D70) |
| **A security policy** | `SECURITY.md` at the root, under `.github/`, or under `docs/` | The file's existence. Its prose is not scored |
| **CODEOWNERS** | The last matching rule wins | A rule that covers `.github/workflows/ci.yml` and names an owner |

Secret scanning, push protection and alert counts stay ungated.

## 3. The catalog

One `repo` record per checkout, id `repo:<name>`. The facts are `dependabotGaps`, `codeScanning`,
`codeScanningPinned`, `securityPolicy` and `workflowOwners`.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `dependabot` | No gap among cargo, npm and github-actions |
| bronze | `code-scanning` | A workflow contains `github/codeql-action` |
| bronze | `security-policy` | A `SECURITY.md` exists in one of the three places |
| silver | `workflows-have-owners` | CODEOWNERS covers `.github/workflows/ci.yml` |
| gold | `code-scanning-pinned` | That action is pinned to a 40-character commit |

## 4. Where this repository lands

| Repository | Level | Notes |
|---|---|---|
| This checkout | gold | Dependabot for cargo, npm and github-actions; `codeql.yml` pins `github/codeql-action`; `SECURITY.md`; CODEOWNERS covers `.github/workflows/ci.yml` |

Fixtures: `repo:acme/focused` at gold, `repo:acme/partial` failing the pin and one ecosystem,
`repo:acme/unsafe` at none. Every rule passes on one fixture and fails on another (D71).

## Sources

- GitHub Dependabot configuration: `package-ecosystem` in `.github/dependabot.yml`.
- GitHub code scanning: the `github/codeql-action` action.
- [github-actions-standards.md](./github-actions-standards.md), which already gates workflow text
  and action pins (D70).
