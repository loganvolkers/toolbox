# Package Author SDLC: Shipping Standards, Connectors and the Binaries They Need

*Status: design, 2026-11-20 (D87); distribution decided (D90, resolving Q16), implementation in progress. Adds a fourth SDLC layer, the **package author SDLC**, to the
maintainer, operator and user SDLCs ([sdlc.md](../sdlc.md)). It revises
[packages.md](./packages.md), which treated a package as data alone. Every layer's stages are now
catalogued and scored in Toolbox for Toolbox (`portal/sdlc/`, scorecards `sdlc-stage`, `sdlc-layer`
and `review-sdlc-stage`).*

---

## 1. Conclusion

**[packages.md](./packages.md) was blind to binaries.** A package that ships GitHub Actions
standards is not just YAML:
- **Connector binaries:** its records come from them (`workspace` parses workflows; `runner` runs the
  linters).
- **Upstream tools:** its rules need zizmor 1.32.0 and actionlint 1.7.12 at pinned versions.
- **Toolbox itself:** it needs a Toolbox binary, which today nobody can install: every consumer,
  including our own `actions/report`, compiles the workspace from source.

Three couplings make the gap structural:

| Coupling | Today | Consequence |
|---|---|---|
| **Connectors ↔ core crates** | In-image connectors are `builtin:<name>`, crates in this repository | Only maintainers can write an in-image connector. Operators and package authors can only push remotely |
| **Toolbox ↔ source builds** | No releases, installers or image | Every repository rebuilds Toolbox (minutes on a cold runner). Packages can't name a Toolbox version to test against |
| **Packages ↔ tools** | zizmor and actionlint are installed by hand (CI steps and README prose) | A package can't say what it needs, so its rules come out `unknown` (D70) on a machine without them |

**Recommendations:**
1. **A fourth layer, the package author SDLC** (§2). It follows the maintainer SDLC until *test*,
   then forks:
   - its test runs against **released** Toolbox versions and platforms, not the workspace;
   - it ends in **distribute** and **adopt**, not merge.
2. **Release Toolbox (maintainer/release) with `dist`.** It produces archives for each platform plus
   shell, PowerShell, Homebrew and npm installers, an MSI and GitHub attestations. We add
   cargo-binstall metadata, an aqua registry entry (which also reaches mise) and a container image.
   apt comes later (§3.1).
3. **Decouple connectors from core by making the protocol the product** (§3.2):
   - **builtin** (shipped in the image);
   - **executable** (any language, released by its author, pinned by digest per platform);
   - **WASM component** (one sandboxed artifact; the long-term default for third-party in-image
     connectors);
   - **remote** (works today).

   The conformance kit is the gate for all four.
4. **Packages declare their tools; mise and aqua install them** (§3.3). Toolbox never installs zizmor
   itself. It checks versions and reports missing ones (`toolbox doctor`). A package emits a
   `mise.toml` fragment with pinned, checksummed tools, and CI uses the same file.
5. **Score the SDLCs** (§4). `portal/sdlc/` declares each layer's stages, with their exit criteria,
   commands, checks, status and gaps. The first run confirms the critique: `maintainer/release`,
   `package-author/package` and `package-author/distribute` are **missing**, and every
   `build-connector` stage is **partial** for the reason above.

## 2. Four layers

| Layer | Who | What they ship | To whom | Ends in |
|---|---|---|---|---|
| 1. Maintainer | This repository's maintainers and agents | Toolbox: binary, image, builtin connectors, schemas, SDKs | Everyone below | A **release** |
| 2. **Package author** (new) | Anyone, inside or outside a company: a platform team, a vendor, us | Packages: standards as data, connector artifacts, tool requirements | Operators and other authors | A **distributed, attested package** |
| 3. Operator | Platform teams running a portal | A portal: manifest plus image | Users | A **deploy** |
| 4. User | Developers and agents building on a portal | Apps, agents, CI checks | Their own users | Their own release |

**Why a separate layer:** a package author is neither a maintainer nor an operator.
- **Not a maintainer:** they can't change Toolbox's crates and must work against released
  versions.
- **Not an operator:** they don't run a portal; their consumers do, across a fleet.
- **Their loop:**
  - **author → build-connector → test** looks like a maintainer's;
  - **package → distribute → adopt** looks like a library publisher's.

  The fork is at *test*. A maintainer tests the whole system at one commit; a package author tests
  one package against a *matrix* of Toolbox releases and platforms, plus the consumer's `plan`.

**We are package authors too.** Our own standards (the GitHub Actions, crate and meta scorecards,
D70–D80) should leave T4T as packages (packages.md §7). Our builtin connectors should be released the
way any author's would, so maintainers live in the package author SDLC as well.

## 3. Binaries

### 3.1 Toolbox itself (maintainer/release)

Popular CLIs offer five or so install paths. What each costs us:

| Channel | Mechanism | Cost to us | Notes |
|---|---|---|---|
| GitHub Releases (binary download) | `dist` builds per-platform archives with checksums | Generated workflow | The source for every channel below |
| `curl \| sh`, PowerShell | `dist` installers | None beyond `dist` | |
| Homebrew | `dist` writes a formula to our tap | A tap repository | `homebrew-core` later, by request |
| npm | `dist` npm installer, or the platform-package pattern (one package per os/cpu, selected by `optionalDependencies`; esbuild, Biome, turbo) | npm trusted publishing | For TS-builder users; known lockfile pitfalls when lockfiles lack other platforms (npm/cli issues) |
| cargo-binstall | `[package.metadata.binstall]` pointing at release assets | Metadata only | Rust users |
| aqua / mise | A PR to the aqua registry (mise's default backend uses it) | A registry PR per format change | Checksums, cosign and SLSA verification come with it |
| apt / rpm | `cargo-deb` or nfpm plus a hosted repository | A repository host and signing key | Defer until asked; the image covers servers |
| Container image | `ghcr.io/<org>/toolbox:<version>` with attestations | Image workflow | Charter G1: operators build `FROM` it |

`dist` generates all of this from `Cargo.toml` metadata, attestations included
(`github-attestations = true`).
- **The risk:** the latest release we found is 0.30 (September 2025).
- **The mitigation:** the generated workflow is ours to keep if upstream stalls.

**What the release unlocks:**
- `actions/report` downloads a pinned binary instead of compiling;
- packages can declare `toolbox: ">=0.3 <0.5"` and test against it;
- federation's fleet rollout stops costing a Rust build per repository.

### 3.2 Connectors outside core

The connector protocol is already language-neutral:
- NDJSON over stdio, or long-poll for remote connectors;
- message schemas in `schema/connector-messages.schema.json`;
- `toolbox connector test` exercises any executable.

The coupling is in **resolution**: `package: builtin:<name>` is the only way to name an in-image
connector.

| Form | How it's referenced | Built by | Isolation | When |
|---|---|---|---|---|
| **builtin** | `builtin:workspace` | Maintainers, in the image | Process | Today |
| **executable** | `github:acme/toolbox-connector-jira@1.2.0`, with per-platform assets and SHA-256 in `toolbox.lock` | Any author, any language (`dist`, GoReleaser, …) | Process; the D40 executor isolation (Unix user, scrubbed environment) applies | Next: the smallest change that frees operators and authors |
| **WASM component** | `oci://ghcr.io/acme/jira-connector@sha256:…` (one artifact for every platform) | Any language with a WASI 0.2 toolchain | Deny-by-default capabilities: the host grants only the network hosts and paths the connector declares | Later: answers Q12 for third-party code; Wasmtime is the mature host (Extism declined the component model) |
| **remote** | `runtime: remote`, pushes to `/v1/ingest` | Anyone, anywhere | Network boundary | Today |

**What makes them first-class:**
- `toolbox connector new <lang>` scaffolds a connector;
- the Rust SDK (`toolbox-connect`) is published to crates.io;
- thin TypeScript and Python SDKs follow, since the protocol is small.

The conformance kit is the **only** gate for the executable and WASM forms. A package that ships a
connector must pass it in its own CI against each supported Toolbox release.

### 3.3 Upstream tools (zizmor, actionlint, …)

| Option | For | Against |
|---|---|---|
| Toolbox installs tools itself | One command | Rebuilds aqua or mise inside Toolbox: per-platform URL templates, checksums, signature checks. Sprawl (D45) |
| Bundle tools inside connector artifacts | Self-contained | Redistribution and licensing per tool; stale copies; enormous artifacts |
| **Declare, and let mise or aqua install (recommended)** | aqua's registry already covers the common tools, verified by checksums, cosign and SLSA; mise uses aqua and records URLs and checksums in `mise.lock`; one file serves laptops, CI (`mise-action`) and the image's Dockerfile | Two tools for consumers to know of; tools outside the registry need a `github:` backend entry |

A package declares, for example:
```yaml
tools:
  zizmor: {version: 1.32.0, mise: "aqua:zizmorcore/zizmor"}   # or "pipx:zizmor"
  actionlint: {version: 1.7.12, mise: "aqua:rhysd/actionlint"}
```
Then:
- `toolbox tools --mise` prints the merged `mise.toml` for the included packages;
- `toolbox doctor` reports missing or wrong versions;
- the rules that need a missing tool stay `unknown`, which is D70's honest-missing-data rule.

Whether zizmor has an aqua entry is unverified; zizmor's own docs list Homebrew, PyPI, cargo,
conda-forge and a GHCR image.

## 4. The SDLCs in Toolbox for Toolbox

`portal/sdlc/layers.yaml` and `portal/sdlc/stages/<layer>.yaml` declare each stage. A stage has:
- intent and exit criterion;
- the commands that carry it out;
- the checks (CI, scripts, tests) that verify it;
- its docs;
- its status (`built`, `partial`, `designed`, `missing`);
- its open gaps.

The workspace connector's new generic **`data`** source reads them. It adds a content hash, for
assessments, and `missingPaths` for docs and checks that don't exist.

| Scorecard | Kind | Rules |
|---|---|---|
| `sdlc-stage` | SdlcStage | bronze: a checkable exit criterion, documented (paths exist); silver: agent-operable (has a command), automated (has a check); gold: built, no open gaps |
| `sdlc-layer` | SdlcLayer | bronze: every stage the layer needs is declared; silver: no stage missing; gold: every stage built |
| `review-sdlc-stage` (advisory) | SdlcStage | Assessments against the `sdlc-stage-quality` rubric: executable exit, an agent can complete it, reuses released artifacts, decoupled from core |

The `reuses-artifacts` and `decoupled-from-core` criteria are the two failures you named, written as
questions an agent answers per stage.

## 5. Revised recommendation for packages

The recommendation in [packages.md](./packages.md) §4 (git-first identity, OCI mirror on the same
digest, data packages with typed parameters) stands. A package now has three parts:

| Part | Format | Pinned in |
|---|---|---|
| Standards | fragments, fixtures, tests, rubrics, skills (data) | `toolbox.lock` (archive SHA-256) |
| Connectors | builtin, executable (per-platform assets) or WASM component | `toolbox.lock` (one digest per platform or component) |
| Tools | declared versions and mise/aqua identifiers | `mise.lock` (the consumer's) |

**Implementation status (D90):**
- **Step 1, built but not yet released:** `dist` config and `release.yml` (actions pinned by SHA), the image workflow, and `actions/report` `toolbox-version`. Internal iteration uses caches, not tags: `Swatinem/rust-cache`, an incremental Cargo cache in the report action, and tools downloaded by checksum (`scripts/ci-tools.sh`).
- **Steps 2–3, built:**
  - `package: github:owner/repo[/app]@version`;
  - `toolbox install` (pins per-platform SHA-256 in `toolbox.lock`, refuses mismatches, `--frozen` for CI);
  - `tools:` and `toolbox doctor` (`--mise`);
  - covered end to end by `crates/toolbox-cli/tests/install.rs`.

- **Step 4, built:**
  - `include: [{package, with}]` with typed parameters that may only tighten;
  - git packages cloned and pinned by content hash in `toolbox.lock`, verified on every load;
  - the first package, `packages/github-actions/` (D70's standards, with connectors and tools), extracted from T4T with an identical ratchet;
  - the package tests itself in `packages/github-actions/tests`.
- **Not built:** WASM connectors (step 5), deterministic archives and the OCI mirror, Homebrew, npm and aqua channels, and a first release (deliberately not cut; internal iteration uses caches).

**Build order**, because each step unblocks the next stage in the catalogue:
1. maintainer/release (`dist`, image, attestations);
2. executable connectors resolved from `toolbox.lock`, plus a published Rust SDK and scaffold;
3. `tools:` and `toolbox doctor`;
4. the package format and `include: package:`;
5. WASM components.

## Sources

Read on 2026-11-20 through web search summaries; secondary sources are marked.

- `dist` (formerly cargo-dist): README https://raw.githubusercontent.com/axodotdev/cargo-dist/main/README.md ;
  GitHub attestations https://axodotdev.github.io/cargo-dist/book/supplychain-security/attestations/github.html ;
  changelog https://axodotdev.github.io/cargo-dist/changelog
- aqua: cosign and SLSA verification https://aquaproj.github.io/docs/reference/security/cosign-slsa
- mise: backends https://mise.jdx.dev/dev-tools/backend_architecture.html , aqua backend https://mise.jdx.dev/dev-tools/backends/aqua.html , registry https://mise.jdx.dev/registry.html
- npm platform packages (secondary): https://blog.sentry.io/publishing-binaries-on-npm/ ; npm RFC https://github.com/npm/rfcs/issues/438 ; lockfile issue https://github.com/npm/cli/issues/4828
- WASI 0.2 https://bytecodealliance.org/articles/WASI-0.2 ; Extism and the component model https://github.com/extism/extism/issues/666 ; Wasmtime plugins and OCI (Plumber's Summit) https://bytecodealliance.org/articles/plumbers-day-1
- zizmor installation https://docs.zizmor.sh/installation/ ; actionlint installation https://github.com/rhysd/actionlint/blob/main/docs/install.md
- This repo: [packages.md](./packages.md), [connector-control.md](./connector-control.md),
  [sdlc.md](../sdlc.md), [maintainer-sdlc.md](./maintainer-sdlc.md),
  [toolbox-for-toolbox.md](./toolbox-for-toolbox.md), [decisions.md](../decisions/README.md) D40, D45, D63,
  D70, D87, Q12, Q15.
