# Dependency Standards: Cataloguing and Scoring the Libraries We Ship Against

*Status: deterministic catalog and scorecard built, 2026-11-20 (D74). Seikika for Seikika catalogues
every direct Cargo dependency and scores it with `maintainer-dependency`. Advisory databases and
licence allow-lists are not gated. Related: [seikika-for-seikika.md](./seikika-for-seikika.md).*

---

## 1. Conclusion

**Catalogue each direct library, and gate the facts the lockfile and the manifests already hold.**

A direct dependency is a crate a workspace member names, other than another member. Dev-dependencies
are out of this catalog: they do not ship in the binary. The questions worth separating:

| Question | What answers it | In this repo |
|---|---|---|
| Is the requirement bounded, and does it come from a registry? | The `req` cargo metadata reports, and `Cargo.lock`'s `source` | bronze |
| Does the lockfile resolve it, and to one version? | `Cargo.lock` | silver |
| If several crates use it, is the requirement declared once? | `[workspace.dependencies]` | gold |
| Does it have a known vulnerability, or a licence we refuse? | RustSec, `cargo deny` | **Not gated.** Both need a database, and a licence policy is a decision we have not made (Q1 is our own licence) |

`workspace` emits one `dependency` record per direct crate when `"dependencies": true`. Thresholds
stay in the manifest. A dependency added after this baseline must reach bronze.

**What we did not adopt:**

| Proposal | Why it is not the gate |
|---|---|
| OpenSSF Scorecard `Vulnerabilities` | It queries OSV. The result changes when the database changes, and a missing network would look like a failure. The same reason zizmor is `unknown` when it is not installed: a database check is a later optional runner, not a fact of the tree we can recompute offline |
| `cargo deny` licence allow-list | Right tool, once this repository decides which licences it accepts. Q1 is still open for Seikika's own licence. Failing a dependency for its licence before that decision would invent a policy |
| Pinning every requirement to `=x.y.z` | Cargo's reproducibility mechanism is `Cargo.lock`, which OpenSSF Scorecard already accepts as the pin for Rust. Exact requirements fight `[workspace.dependencies]` and the caret ranges the ecosystem publishes |
| Transitive crates as entities | The lockfile has 384 packages. The direct set is what a maintainer chose. Transitive duplicates show up as extra versions of a direct crate (`sha2`, `syn`), which is the fact the silver rule reads |

## 2. What the field measures

| Source | What it decides | Deterministic here | Needs a database or a run |
|---|---|---|---|
| **Cargo lockfile** | The resolved version and source of every crate the build uses | Parsed from `Cargo.lock` | Nothing |
| **Cargo workspace inheritance** | One requirement in `[workspace.dependencies]`, inherited by members, so two crates cannot drift | The root manifest's table, and who names the crate | Nothing |
| **OpenSSF Scorecard** `Pinned-Dependencies` | A lockfile pins Rust (and Go, npm). GitHub Actions are pinned separately, by SHA | The lockfile's presence for a direct crate | The Scorecard probe's network |
| **cargo-deny** | Advisories, licences, bans (duplicate versions, wildcards), sources (registries and git) | Wildcard requirements and git sources, which `[bans]` and `[sources]` also flag | Advisory and licence checks |
| **RustSec advisory-db** | Yanked and vulnerable crates | Nothing, until a snapshot of the database is vendored | The database |
| **Dependabot / Renovate** | Update PRs. They do not decide whether today's tree is acceptable | Not a property of one revision | The bot |

## 3. The catalog

Opt-in `"dependencies": true`. Id `dependency:cargo:<name>`. `cargo metadata --no-deps` supplies the
requirement and the using crates. `Cargo.lock` supplies versions and sources. Only normal
dependencies; `kind: dev` and `kind: build` are not records.

| Fact | What it is |
|---|---|
| `requirements` | Distinct `req` strings. Cargo rewrites `1` as `^1` |
| `usedBy` | Workspace members that name it |
| `versions`, `sources` | Every `Cargo.lock` package of that name. A direct `^0.11` still lists `0.10` if something else pulled it in |
| `git` | Any source starts with `git+` |
| `wildcard` | A requirement is empty, `*`, or starts with `>` (no ceiling) |
| `inherited` | The name is a key of `[workspace.dependencies]` |

## 4. The scorecard

`maintainer-dependency`, kind `Dependency`. Maintainer SDLC: these are libraries the binary is built
against.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `version-bounded` | At least one requirement, and none of them is a wildcard |
| bronze | `from-a-registry` | No git source |
| silver | `locked` | `Cargo.lock` has the crate |
| silver | `one-version` | The lockfile has exactly one version of it |
| gold | `shared-is-inherited` | It is used by one crate, or `[workspace.dependencies]` declares it |

## 5. This repository

Measured with `seikika report portal --skip runner,github-actions`. Twenty direct crates.

| Level | Crates |
|---|---|
| gold | 16, including `tokio`, `serde`, `axum`, `sqlx` |
| silver | `regex` (four crates, not in `[workspace.dependencies]`), `walkdir` (two crates, same) |
| bronze | `sha2` (`0.10.9` and `0.11.1`), `syn` (`2.0.119` and `3.0.10`) |

`proc-macro2` is gold: only the workspace connector names it, so a workspace entry is not required.
No direct dependency is a git source or a wildcard. `anyhow` is in `[workspace.dependencies]` and
unused by any member; it is not an entity.

Fixtures: `dependency:cargo:focused` at gold, `partial` at bronze (two versions, shared, not
inherited), `unsafe` at none (a star requirement and a git source). Every rule passes on one
fixture and fails on another (D71).

## Sources

- Cargo reference, "Inheriting a dependency from a workspace",
  `doc.rust-lang.org/cargo/reference/specifying-dependencies.html`. The lockfile format is the
  `[[package]]` tables in `Cargo.lock`, name / version / source.
- OpenSSF Scorecard, `ossf/scorecard` `docs/checks.md`, check `Pinned-Dependencies`: a checked-in
  lockfile is the Rust pin. `Vulnerabilities` queries OSV and is not adopted here.
- `EmbarkStudios/cargo-deny` README: `[bans]` (wildcards, duplicates), `[sources]` (git and
  unknown registries), `[advisories]`, `[licenses]`. The first two are the bronze and silver rules.
  Advisories and licences are the follow-on.
- RustSec advisory database, `rustsec/advisory-db`. Not vendored in this change.
