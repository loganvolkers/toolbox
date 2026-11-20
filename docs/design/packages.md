# Toolbox Packages: Sharing Standards, Publishing Our Own, and Running Them Across a Fleet

*Status: decided, 2026-11-20 (D89, resolving Q15). Package includes, parameters and git pinning are built; the first package is `packages/github-actions/`. Originally research and proposal. Answers the flagged open question Q15 (federation's
package registry) with a recommendation that is **not yet decided**. Covers four questions raised
while building Toolbox for Toolbox (T4T):*
1. *How should we catalogue the open-source packages we publish?*
2. *Is T4T's single 1,841-line manifest OK?*
3. *What should a shareable package of Toolbox primitives look like, and where should it live?*
4. *How do we tell what is unique to us from what is worth sharing?*

*Layers: the **operator SDLC** (sharing and importing standards) and the **maintainer SDLC** (what we
publish). **Revised by [package-author-sdlc.md](./package-author-sdlc.md) (D87):** a package also
carries connector artifacts and tool requirements, and this design was blind to binaries; read §3–4
here with §3 and §5 there. No code changes. Builds on [federation.md](./federation.md), [typescript-builder.md](./typescript-builder.md)
and [repo-portals.md](./repo-portals.md).*

---

## 1. Conclusion

**A Toolbox package should be data, not code:**
- **What it contains:** manifest fragments, fixtures, tests, rubrics and agent skills, with typed
  parameters and a version.
- **How it is identified:** by a git path and a semver tag, and pinned in a lockfile by commit and
  content digest.
- **How it is fetched:** from git by default, or from an OCI mirror carrying the *same* digest.

This is the Go-modules shape, which the configuration ecosystems we surveyed are converging on:
- **Git refs:** Renovate is deprecating npm presets in favour of git-hosted ones. GitHub Agentic
  Workflows (gh-aw) and Microsoft's Agent Package Manager (APM) install from git. Claude Code plugin
  marketplaces are git repositories.
- **OCI:** CUE, KCL, OPA, Helm and OpenTofu ship configuration as OCI artifacts where enterprises and
  air-gapped sites need them.

| Question | Answer |
|---|---|
| **What a package is** (§3) | `toolbox-package.yaml` (name, version, Toolbox compatibility, typed `parameters`, `exports`) plus fragments, fixtures, tests, rubrics and skills. A package must pass its own `toolbox test` and reach meta bronze before it can be published |
| **Where it lives** (§4) | **Recommended: option R1+, git-first with an OCI mirror.** No registry of our own, no Node requirement for YAML users, private packages use existing repository permissions. Provenance comes from GitHub immutable releases and artifact attestations |
| **The monolith** (§5) | Split T4T now with the existing `include` (no new technology), by concern and owner. Then remove its duplication with parameterised packages, not a TypeScript program. The TS builder stays for teams that want it and compiles to the same data |
| **Our public packages** (§6) | A `Package` kind and an `oss-package` scorecard anchored in OpenSSF Scorecard, the OSPS Baseline, SLSA and each ecosystem's norms. Licence (Q1) blocks bronze for everything we publish |
| **Unique or shareable** (§7) | Deterministic signals (repo-specific literals, kinds only we have, duplication across portals) plus a `shareability` rubric. Candidates graduate T4T → `packages/` in this repo → published, and T4T consumes its own packages |
| **Fleet scale** (§8) | Deterministic CI does the gating (`actions/report` with a pinned baseline package). gh-aw adds the agentic part: agents raise ratchets and write assessments, installed from a central repo with `gh aw add …@v1` |

**What needs deciding:**
1. Adopt R1+ (§4.4), which resolves Q15.
2. Split T4T with `include` (§5).
3. Package format v0, including how parameters work (§3.3).
4. ~~The project licence (Q1).~~ Decided: MIT (D88). Items 1 and 3 are decided too (D89); item 2 is next.

## 2. What the ecosystems do

Configuration-sharing ecosystems, read from primary sources where reachable (secondary sources are
marked).

| Ecosystem | Unit | Transport | Pinning | Composition | Provenance | Notes |
|---|---|---|---|---|---|---|
| **Renovate presets** | JSON preset | git host (`github>org/repo:sub#1.2.3`), local; **npm deprecated** | git tag | `extends`, later wins; presets take arguments | none | The project moved *away* from npm for non-JS configuration |
| **gh-aw** (public preview) | Markdown workflow; `aw.yml` package manifest (draft spec 0.1.0) | git (`gh aw add owner/repo/wf@v1.0.0`) | tag; `source:` recorded in frontmatter; remote imports cached by commit SHA | `imports:`; `gh aw update` keeps local edits | none beyond git | A central-repo pattern for organisations (versioned templates plus shared modules) |
| **APM** (Microsoft, Thoughtworks Radar "Trial") | Agent primitives (instructions, skills, prompts, agents, hooks, MCP) | git only (GitHub, GHE, GitLab, Azure DevOps); Artifactory "Agent Packages" repos | `apm.lock.yaml`: commit plus **SHA-256 per package and per deployed file**; `--frozen` | transitive dependencies | none; `apm audit`, org `apm-policy.yml`, hidden-Unicode scan | Dependabot version updates support `apm.yml` (GitHub changelog, 2026-11-04) |
| **Claude Code marketplaces** | plugin | git repository with `.claude-plugin/marketplace.json`; `github`/`git-subdir`/`url` sources | ref | — | none | Private repositories use the machine's git credentials |
| **Agent Plugins 1.0.0** | plugin package | (spec; [agent-plugins.md](./agent-plugins.md)) | — | — | — | Our operator plugin already conforms |
| **ESLint flat config** | JS module | npm | semver plus lockfile | `extends` in `defineConfig`; local entries override | npm provenance | JS-only toolchain |
| **OPA / Conftest** | policy bundle | **OCI** (OPA native since v0.40; Conftest pushes via ORAS), plus git and HTTP (go-getter) | digest | data merge | cosign (secondary source) | Policy-as-code went OCI |
| **CUE modules** (experimental) | module | **OCI** (`cue mod publish`; `CUE_REGISTRY`; central registry.cue.works) | version plus content hash | unification | — | A configuration language chose OCI |
| **KCL** | module | **OCI** (ghcr.io by default; `kcl mod push oci://…`); listed on Artifact Hub | tag | — | — | Same |
| **Pkl** | package | HTTPS URI (`package://host/path@1.2.3`), checksum per version; GitHub releases work | checksum | `amends` / `extends` | checksum | Tag must equal the metadata version |
| **OpenTofu 1.10** | module, provider | **OCI** (`oci://…?tag=`/`digest=`) next to its registry | digest | modules | — | Pitched at air-gapped use |
| **Backstage (Red Hat Developer Hub)** | dynamic plugin | **OCI** (`oci://…`, digest for integrity); community plugins moved to ghcr.io in 1.9 | digest | — | — | A *portal* ecosystem chose OCI for plugins (secondary: vendor docs) |
| **Go modules** | module | git, with proxy.golang.org as a cache | `go.sum` hashes, checked against **sumdb** (a transparency log) | — | the transparency log | Git identity plus content hash plus an untrusted mirror: the model R1+ copies |
| **Bazel Central Registry** | module | a git *index* repository; archives hosted elsewhere | SRI `integrity` in `source.json` | — | integrity reviewed on submission | An index-as-git-repo, no artifact hosting |

**Publishing trust is now available without long-lived tokens:**
- **npm trusted publishing:** OIDC is GA (2025-07-31).
- **crates.io trusted publishing:** OIDC, launched July 2025; GitLab followed in 2026.
- **GitHub immutable releases:** GA 2025-10-28. Tags can't be moved and releases are attested.
- **GitHub artifact attestations:** Sigstore-signed, and verifiable for files and `oci://` images
  with `gh attestation verify`.
- **Not available:** GitHub's *immutable actions* published to GHCR, which are still unreleased per
  `actions/publish-immutable-action`.

## 3. What a Toolbox package looks like

### 3.1 Three shapes considered

| Shape | Example | For | Against |
|---|---|---|---|
| **P1. Data package**: manifest fragments plus metadata | Renovate preset, OPA bundle, Spectral ruleset | The binary validates it; language-neutral; diffable; works in repo portals without Node; `plan` can show an upgrade's impact | Reuse needs a parameter mechanism (§3.3) |
| **P2. Code package**: TS functions that emit fragments | CDK constructs, ESLint shareable configs | Full abstraction power; typed | Requires Node everywhere; the output is only known after running code; can't be inspected without executing; npm supply-chain exposure |
| **P3. Typed configuration language**: CUE, KCL or Pkl modules | CUE module | CUE unification *is* "children may tighten, never weaken" (federation §3) | A new language for every operator, breaking the sprawl budget (D45); a second schema next to JSON Schema |

**Recommended: P1.** The TS builder (P2) stays as an *authoring* option that compiles to P1, as D53
already makes it compile to the canonical manifest. P3's lattice semantics are worth borrowing as
validation rules (monotonic tightening), not as a language.

### 3.2 Layout

```
toolbox-package.yaml        # identity, compatibility, parameters, exports, dependencies
fragments/*.yaml            # kinds, fields, scorecards, rubrics, connector configs (manifest schema)
fixtures/*.ndjson           # records that exercise every exported rule both ways
tests/*.test.yaml           # the package's own expectations (meta-rule `tested` holds inside the package)
skills/<name>/SKILL.md      # how an agent raises these standards (Agent Skills format)
rubrics are fragments; assessments are never shipped (they judge a consumer's entities)
README.md  CHANGELOG.md  LICENSE
```

```yaml
# toolbox-package.yaml
apiVersion: toolbox.dev/package/v0
name: github.com/acme/toolbox-standards/ci   # identity = git path (+ subdirectory)
version: 1.4.0                               # must equal the release tag (as in Pkl)
toolbox: ">=0.2 <0.4"                         # compatible Toolbox versions
description: CI security and reliability standards built on zizmor and actionlint.
parameters:
  failRateBronze: {type: integer, default: 20, description: "Max % failed runs on main for bronze."}
  stage: {enum: [experimental, advisory, mandatory], default: advisory}
exports:
  kinds: [Workflow, ActionDependency]          # kinds it defines (namespaced on import: ci/Workflow)
  scorecards: [ci-security, ci-reliability]
  rubrics: [workflow-quality]
requires:                                     # kinds and connector records it reads, not defines
  records: [workspace/workflow, github-actions/workflow-runs]
dependencies:
  - github.com/acme/toolbox-standards/core@^2
```

The consumer side extends D54's `include`:
```yaml
include:
  - package: github.com/acme/toolbox-standards/ci@^1
    with: {failRateBronze: 15, stage: mandatory}   # parameters may only tighten (validated)
```
`toolbox.lock` (committed) records, per package:
- the resolved version and the commit SHA;
- the **SHA-256 of the canonical package archive**;
- the source it was fetched from.

### 3.3 Parameters, not templates

The duplication in T4T is parameter-shaped: four `review-*` scorecards and four sets of
`reviewState`/`reviewFailing`/`reviewUnsure` fields differ only in a kind, a relation and a rubric.
Three ways to remove it:

1. **Typed parameters substituted before validation** (`${param:failRateBronze}`), like Renovate's
   preset arguments. Small, inspectable, validated.
2. **Instantiation:** a package exports a *scorecard template* that a consumer instantiates per kind
   (`instances: [{kind: Rule, rubric: rule-quality}, …]`). Removes the review-* duplication.
3. **Code** (the TS builder). Unlimited, but P2's costs.

**Recommended:** 1 now and 2 when a second package needs it. 3 remains available through the
builder.

**Rule for parameters:** a parameter declares its direction (`tighten: lower`, as for
`failRateBronze`). `toolbox validate` rejects a `with:` value that weakens the package's default
when the package is included as `mandatory`. This is federation's monotonic tightening, applied at
the parameter rather than the rule.

## 4. Where packages live

### 4.1 Requirements

From federation §3, Q15 and charter goals G1–G3:
- pin by digest;
- provenance;
- private hosting inside enterprises;
- air-gapped use;
- fetchable by the `toolbox` binary with no extra toolchain;
- update bots (Dependabot, Renovate) can propose upgrades;
- no registry of our own;
- discovery.

### 4.2 Options

| Option | Pin and integrity | Provenance | Private and air-gapped | Toolchain | Update bots | Discovery | Verdict |
|---|---|---|---|---|---|---|---|
| **R1. Git refs** (tag → commit → archive) | commit SHA plus our archive hash in `toolbox.lock` | immutable releases (GA) plus `gh attestation verify` on the release asset | repository permissions; mirror the repository | `git` or HTTPS only | Renovate (regex or custom manager on `toolbox.lock`); a built-in `toolbox update` (as `gh aw update`) | GitHub search, topics | Matches every fleet tool in §2 |
| **R2. OCI artifacts** (ORAS media type on GHCR, Artifactory, Harbor, ECR, ACR) | digest, native | Sigstore or GitHub attestations stored as OCI referrers | enterprise registries exist; air-gap by registry mirroring; can be baked into the one image (G1) | the binary needs an OCI client (e.g. the `oci-client` crate), not the `oras` CLI | Renovate's docker datasource; Dependabot doesn't bump OCI refs in custom files | Artifact Hub | Best for air-gapped sites and the server image; weaker for repo portals |
| **R3. npm** | lockfile integrity | trusted publishing plus provenance (GA) | private npm registries | Node for every consumer, against D53's "Node only for the builder" | native in both bots | npmjs.com | Fits only TS-builder users; Renovate's npm-preset deprecation is a signal against it |
| **R4. APM as the carrier** | per-file SHA-256 lock | none | git or Artifactory | the `apm` CLI | Dependabot (`apm.yml`) | APM | Carries skills and prompts well; fragments aren't an APM primitive and its install model writes into agent harness folders. Use it for the *agent part* only |
| **R5. Our own registry** | — | — | — | — | — | — | Rejected: the request was to piggyback |

### 4.3 Why not one transport only

- **Git alone** fails air-gapped sites and leaves the server image fetching from GitHub at start-up.
- **OCI alone** puts a second permission system (registry packages) next to the repositories that
  repo portals live in, and none of the fleet tools in §2 (gh-aw, APM, Renovate presets,
  marketplaces) reads OCI.
- **npm** couples configuration to a JS toolchain.

### 4.4 Recommendation: R1+, git-first with an OCI mirror on the same digest

1. **Identity is git** (`github.com/acme/toolbox-standards/ci`), **versions are tags** (immutable
   releases required for `mandatory` packages), and the canonical artifact is a **deterministic
   archive** attached to the release.
2. **`toolbox.lock` pins the archive's SHA-256.** The same archive can be pushed as an OCI artifact
   (`application/vnd.toolbox.package.v0+tar`). A mirror setting
   (`TOOLBOX_PACKAGE_MIRROR=oci://registry.internal/toolbox`) makes the resolver fetch by digest from
   OCI instead of git, which covers enterprise and air-gapped use. The digest check makes the mirror
   untrusted, as Go's proxy is.
3. **Provenance:** packages are built and released by a reusable workflow with `actions/attest`.
   `toolbox` verifies attestations when a package is `mandatory` (SLSA Build L2; L3 with a
   hardened reusable workflow).
4. **`toolbox build` (D53) vendors resolved packages into the canonical manifest,** so the server
   image never fetches at runtime.
5. **Agent pieces:** a package's `skills/` are plain Agent Skills, so `apm install` and plugin
   marketplaces can carry them too. We add an `apm.yml` only if adopters ask.

**Pros:**
- No infrastructure and no registry of our own.
- Private packages use repository permissions.
- Works in repo portals with no new toolchain.
- The same unit runs from git, OCI or a vendored image.
- It matches how gh-aw, APM, Renovate and Claude marketplaces distribute.

**Cons and risks:**
- We write a small resolver: git fetch, OCI fetch, lockfile, semver range.
- Dependabot needs Renovate or our own `toolbox update` until GitHub supports the format.
- There is no transparency log like Go's sumdb. Attestations and immutable releases are the
  substitute.
- Moving tags (`@v1`) are convenient but mutable. `mandatory` includes therefore require exact
  versions in the lockfile.

## 5. The T4T monolith

T4T's `portal/toolbox.yaml` is 1,841 lines: 5 connectors, 21 kinds, 26 scorecards with 143 rules, and
4 rubrics. Scorecards are 965 lines and kinds 665.

| Option | For | Against |
|---|---|---|
| Keep one file | One place to search | Every T4T change conflicts in one file (seven changes, D72–D78, landed in it in one morning); CODEOWNERS can't split ownership; fragments can't become packages |
| **Split with `include` (D54, built)** | No new technology; per-file CODEOWNERS; each file becomes a candidate package (§7) | Duplication remains |
| TS builder (Pulumi-style) | Functions remove duplication | Node in this repo's CI; T4T's value is being the canonical *data* example operators copy; the builder is still a prototype |

**Recommended:** split now, along the lines packages will follow:

```
portal/toolbox.yaml                     # portal, policies, mcp, include list
portal/connectors.yaml
portal/standards/maintainer/{crates,docs,ci,skills,connectors,deps,agent-docs,packages,schemas,examples}.yaml
portal/standards/operator/cli.yaml
portal/standards/user/{cli,mcp}.yaml
portal/meta/{rules,model,assessments}.yaml   # meta-*, review-*, rubrics
```

Then let the second user of a pattern trigger a package with parameters (§3.3). The review-*
scorecards and their computed fields are the first candidate.

## 6. Cataloguing what we publish

What Toolbox publishes, or will publish:
- crates;
- the `actions/report` action;
- the `plugins/toolbox-operator` agent plugin;
- `@toolbox/manifest`;
- JSON Schemas;
- container images;
- Toolbox packages.

Proposed: a `Package` kind (record per published unit: ecosystem, name, path, release and publish
facts), and an **`oss-package`** scorecard. Its anchors are OpenSSF Scorecard's checks (18 checks;
e.g. `Signed-Releases`, `Packaging`, `Pinned-Dependencies`, `Token-Permissions`), the OpenSSF
**OSPS Baseline** (three maturity levels across access control, build and release, documentation,
governance, legal, quality and vulnerability management; designed to map to the EU CRA and NIST
SSDF), and SLSA build levels.

| Level | Rules | Anchor |
|---|---|---|
| bronze | licence declared (blocked by Q1); README with install and usage; security policy with private reporting; versioned releases from tags; owner | OSPS L1 (documentation, legal, vulnerability management) |
| silver | trusted publishing, no long-lived tokens (npm, crates.io, GHCR); provenance attestation on every release; immutable releases; changelog; least-privilege release workflow | SLSA Build L2, Scorecard `Signed-Releases`, `Token-Permissions`, OSPS L2 build and release |
| gold | builds in a hardened reusable workflow; OpenSSF Scorecard ≥ 7; reproducible archive; compatibility checked on every release | SLSA Build L3, OSPS L3 |

Ecosystem rules sit under `appliesTo`:

| Ecosystem | Ecosystem rules |
|---|---|
| Crates | `rust-version` (MSRV) declared; `cargo publish --dry-run` in CI; `cargo-semver-checks`; docs.rs builds; no workspace-only paths in `include` |
| GitHub Actions | `action.yml` metadata complete (`description`, `branding`, every input described); a major-version tag; immutable releases; its own `uses:` pinned by SHA (already scored by `maintainer-ci-security`) |
| Agent plugins | `plugin.json` validates against Agent Plugins 1.0.0; skills pass `skills-ref`; no hidden Unicode (APM's check); listed in a marketplace |
| Toolbox packages | `toolbox test` passes on the package; every exported rule at meta bronze; parameters documented; `toolbox` compatibility range; the archive is deterministic |

**Source:** the workspace connector already reads `Cargo.toml`, `package.json`, `action.yml`, plugin
and schema files. Release facts (tags, attestations, immutable releases) come from the GitHub
connector. One new `package` record kind joins them.

## 7. Unique to us, or worth sharing?

| Signal | How | What it says |
|---|---|---|
| Repo-specific literals in CEL | The CEL analysis already lists string literals; flag paths, org and repository names | Only **5 of T4T's 143 rules** mention anything specific to this repo: specificity lives in connector config and kind mappings, not in the standards |
| Kinds only we have | A rule reads fields of kinds no builtin connector produces | Coupled to our model; package the kind with it or keep it local |
| Duplication across portals | The same rule shape in T4T, the starter and the reference portal (`describe portal` content hashes) | Two portals → a package candidate |
| Thresholds | Literal numbers in checks | Parameter candidates |
| Prior art | The rubric asks for an external standard (OpenSSF, OSPS, Betterer) | Anchored standards travel better |

A `shareability` rubric would join the `review-*` mechanism (D79):
- Would another organisation adopt it unchanged after setting parameters?
- Is its intent general rather than a fix for one of our incidents?
- Does an external standard back it?
- Does it read only builtin-connector records?

Graduation:
1. A rule starts in T4T.
2. It moves to `packages/<name>/` in this repository.
3. T4T `include`s its own package (dogfood).
4. It is published by tag with attestations.
5. Others include it.

These are federation's stages (experimental → advisory → mandatory) applied to our own standards.
Likely first packages:
- `ci` (zizmor, actionlint, run history);
- `rust-crate`;
- `agent-docs` and `skills`;
- `meta` (scorecards for scorecards, assessments).

## 8. Fleet scale with GitHub Agentic Workflows

Gating stays deterministic; agents do the improving.

1. **Baseline.** A company publishes `github.com/acme/toolbox-standards@v3` (R1+). Each repository
   carries a small `.toolbox/toolbox.yaml` that `include`s it, plus `.toolbox/results.json`, as the
   starter (D69) does today.
2. **Gate.** `actions/report` (plain Actions, deterministic) runs the ratchet on every PR. Rollout
   across repositories is a PR per repository: org rulesets or required workflows, or a gh-aw
   CentralRepoOps workflow that opens them.
3. **Improve.** An agentic workflow installed from a central repository
   (`gh aw add acme/toolbox-fleet/raise-the-bar@v1`):
   - reads the report's failing rules and evidence;
   - fixes one rule and runs `toolbox report --update`;
   - opens a PR;
   - writes assessments (D79) for the rubric-based cards.

   gh-aw's own guidance fits this split: versioned templates in a central repository, shared modules
   through `imports:`, GitHub Apps rather than PATs for cross-repository access.
4. **Results up.** The root portal reads each repository's committed `results.json` (federation F2).
   Package versions per repository appear as a federation adoption view.

**Caveats:**
- gh-aw is in public preview, and its organisation guides warn about API limits beyond roughly 50
  repositories (secondary: their blog).
- Agentic runs cost tokens per repository, so the improvement workflow should be scheduled and
  bounded (one rule, one PR).

## 9. Open questions

| # | Question |
|---|---|
| PQ1 | Package names: full git paths (Go-style, unambiguous) or short aliases resolved through a git *index* repository (as the Bazel Central Registry does)? |
| PQ2 | Namespacing on import (`ci/Workflow`) versus federation FQ2 (extending a parent's kinds) |
| PQ3 | Should `toolbox` verify attestations itself or shell out to `gh attestation verify`? Offline verification needs bundles shipped with the archive |
| PQ4 | Do we publish an `apm.yml` for packages' skills? Only if adopters use APM |
| PQ5 | The licence (Q1): nothing in §6 reaches bronze, and no package can be published, until it is decided |

## Sources

Primary unless marked. Pages were read on 2026-11-20 through web search summaries. Some hosts
(microsoft.github.io, github.github.com) failed DNS from this session, so APM's manifest schema and
gh-aw's dependency page were read only through search excerpts.

- Renovate shareable config presets, including the deprecation of npm presets: https://docs.renovatebot.com/config-presets/
- gh-aw: reusing workflows and imports https://github.github.com/gh-aw/guides/packaging-imports/ ;
  aw.yml package manifest https://github.github.com/gh-aw/reference/aw-yml-package-manifest/ ;
  using at scale https://github.github.com/gh-aw/guides/using-at-scale/ ;
  public preview status https://docs.github.com/en/copilot/how-tos/github-agentic-workflows/creating-github-agentic-workflows
- APM README (read directly): https://raw.githubusercontent.com/microsoft/apm/main/README.md ;
  manifest schema https://microsoft.github.io/apm/reference/manifest-schema/ ; Dependabot support
  https://github.blog/changelog/2026-11-04-dependabot-version-updates-support-agent-package-manager ;
  JFrog Agent Packages repositories https://docs.jfrog.com/artifactory/docs/agent-packages-repositories
- Claude Code plugin marketplaces: https://code.claude.com/docs/en/plugin-marketplaces
- ESLint flat config `extends`: https://eslint.org/blog/2025/03/flat-config-extends-define-config-global-ignores/
- Conftest sharing policies: https://conftest.dev/sharing ; OPA bundles (OCI since v0.40): https://openpolicyagent.org/docs/bundles , https://github.com/open-policy-agent/opa/issues/4518
- CUE modules and registries: https://cuelang.org/docs/reference/modules , https://cuelang.org/docs/tutorial/working-with-a-custom-module-registry/
- KCL OCI registries: https://www.kcl-lang.io/docs/user_docs/guides/package-management/how-to/kpm_oci
- Pkl package pages (secondary: generated docs, a GitHub discussion): https://github.com/apple/pkl/discussions/85
- OpenTofu 1.10 OCI registries: https://opentofu.org/docs/cli/oci_registries/ , https://opentofu.org/blog/opentofu-1-10-0/
- Red Hat Developer Hub dynamic plugins as OCI (vendor docs, secondary for Backstage upstream): https://docs.redhat.com/en/documentation/red_hat_developer_hub/1.9/html/dynamic_plugins_reference/ref-community-plugins-migration_title-plugins-rhdh-about
- Go checksum database design: https://golang.org/design/25530-sumdb ; module mirror: https://go.dev/blog/module-mirror-launch
- Bazel registries: https://bazel.build/external/registry
- npm trusted publishing GA: https://github.blog/changelog/2025-07-31-npm-trusted-publishing-with-oidc-is-generally-available/ ;
  crates.io trusted publishing: https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07 , https://rust-lang.github.io/rfcs/3691-trusted-publishing-cratesio.html
- GitHub immutable releases GA: https://github.blog/changelog/2025-10-28-immutable-releases-are-now-generally-available/ ;
  immutable actions not released: https://github.com/actions/publish-immutable-action ;
  artifact attestations: https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations , https://cli.github.com/manual/gh_attestation_verify
- Dependabot supported ecosystems: https://docs.github.com/en/code-security/reference/supply-chain-security/supported-ecosystems-and-repositories ;
  Helm support https://github.blog/changelog/2025-04-09-dependabot-version-updates-now-support-helm/
- OpenSSF Scorecard checks: https://github.com/ossf/scorecard/blob/main/docs/checks.md ;
  OSPS Baseline: https://baseline.openssf.org/versions/2026-08-28
- This repo: [federation.md](./federation.md), [typescript-builder.md](./typescript-builder.md),
  [repo-portals.md](./repo-portals.md), [agent-plugins.md](./agent-plugins.md),
  [scorecards-for-scorecards.md](./scorecards-for-scorecards.md), [decisions.md](../decisions.md) D53–D54, D68–D69, D79, Q1, Q15.
