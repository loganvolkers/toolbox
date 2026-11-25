# Maintainer SDLC (Layer 1): Building Toolbox Itself

*Status: describes practice as of 2026-11-20 (M0–M4 done); design review added 2026-11-24 (D01M87PSM9YQGGW3YQ5X4XCJP63). Layer 1 of [the four SDLC
layers](../sdlc.md). Design rationale for speed and testing lives in
[developer-velocity.md](./developer-velocity.md) and [verification.md](./verification.md); this
document is how maintainers, mostly agents in harnesses, actually work.*

---

## 1. Conclusion

A maintainer agent can take a milestone from [v1-spec.md](../v1-spec.md) §3, implement it, and
*prove* it is done without a human checking by eye, because every exit criterion is an executable
check:
- the Python reference model is the oracle for derived state;
- the parity script compares `toolbox validate` against the reference validator;
- the acceptance scripts exercise each milestone on real Postgres, OpenFGA and a browser;
- CI runs all of it.

The inner loop takes seconds, so this stays cheap enough to run after every change.

## 2. The loop

| Step | What the maintainer does | Tooling |
|---|---|---|
| **Orient** | Read [AGENTS.md](../../AGENTS.md), [decisions.md](../decisions/README.md) and the milestone in [v1-spec.md](../v1-spec.md) §3 | `CLAUDE.md` → `AGENTS.md` |
| **Plan** | Turn exit criteria into checks *before* coding: a test, an acceptance-script step, or a reference-model case | §4 |
| **Build** | Small changes in a crate of the [workspace](../v1-spec.md) §4 | `cargo build` (toolchain pinned in `rust-toolchain.toml`) |
| **Test** | Unit and integration tests, the differential tests, the relevant acceptance script | §4 |
| **Review** | `cargo fmt --check`, `cargo clippy -D warnings`; read your own diff | CI |
| **Design review** (`design-review`) | For a change readers see: story probes, then a blind panel of three auditors over two reference portals, until the stop rule holds (D01M87PSM9YQGGW3YQ5X4XCJP63) | `report-ui` (`npm run probe`), [design-audit](./design-audit.md) |
| **Record** | Results in [research-log.md](../research-log.md); decisions in [decisions.md](../decisions/README.md); spec status in v1-spec §3 | Working rules in AGENTS.md |
| **Ship** | Small commits straight to `main` (D32), pulled with rebase first | `git` |
| **Monitor** | CI on every push; the scale budget and acceptance jobs catch regressions | `.github/workflows/ci.yml` |

## 3. Measured loop (4 vCPU container, warm caches)

| Action | Time |
|---|---|
| Rebuild after editing `toolbox-derive` | 2.1 s |
| Rebuild after editing `toolbox-api` | 1.4 s |
| `cargo test` (all crates, 30 tests, needs `TOOLBOX_OPENFGA_BIN`) | 3.6 s |
| `toolbox test examples/reference-portal` (20 expectations + `.fga.yaml`) | 0.2 s |
| `cargo clippy --all-targets -D warnings` | 4.7 s |
| `scripts/m4-acceptance.sh` (Postgres, OpenFGA, three connectors, Chromium) | 7.9 s |
| `npm run check` in `report-ui` (`tsc`, then the Vite bundle against the committed one) | 0.5 s |
| `npm run build-storybook` (53 stories over five fixtures) | 1.3 s |
| `npm run probe` (53 stories × 4 views = 212 checks; 4 workers) | 33 s |

The workspace has 13 crates. The deferred velocity tooling (hakari, affected-test selection, result
caching) comes back only when these numbers regress (D44, [developer-velocity.md](./developer-velocity.md)).

## 4. The check pyramid

| Level | Check | Proves | Command |
|---|---|---|---|
| Unit | Crate tests (CEL conversion, lazy lists, CODEOWNERS parsing, hub coalescing, DSL conversion, JWT verification) | Local behaviour | `cargo test` |
| Contract | JSON Schemas for manifest, connector and executor messages; golden protocol transcripts | Wire formats | `toolbox validate`, `tools/validate.py` |
| Parity | `toolbox validate` vs `tools/validate.py` on 25 accept/reject cases | Validator equivalence | `scripts/validate-parity.py` |
| **Differential** | Rust pipeline output byte-identical to `tools/reference.py` (P2) | Derived-state semantics | `cargo test -p toolbox-derive` |
| Authorization | Reference `.fga.yaml` through the Authorizer; DSL converter vs `fga model transform` | Model semantics | `cargo test -p toolbox-authz` |
| Connector | Conformance kit against each in-image connector | Protocol behaviour | `toolbox connector test` |
| GitHub connector integration | Real GitHub connector over stdio and HTTP against a seeded Emulate instance | GitHub response mapping, stateful refresh and authentication | `npm --prefix scripts/github-emulate test` |
| Connector smoke | Real github, git-files and SCIM connectors feeding `toolbox serve` on the in-memory store; derived outputs byte-identical to the reference. No Postgres or OpenFGA, so it runs locally in about a minute; not in CI, which runs the full M4 path | Connectors end to end | `scripts/m4-connectors-smoke.sh` |
| Acceptance | One script per milestone, on real Postgres 16, OpenFGA, connectors and Chromium (M5 needs none of them) | Exit criteria | `scripts/m2-acceptance.sh`, `m3-…`, `m4-…`, `m5-…` |
| Scale | 100k entities / 2M relations / 100 rules ≤ 10 s | Budget §10.4 | `cargo run --release -p toolbox-derive --example scale` |
| Format | Tracked `.yaml` and `.yml` match Yamark's fixed point (D97) | One style for embedded objects | `yamark format --check` |
| Freshness | `ui/dist` matches its source | Embedded UI is current | CI `ui` job |
| Types and freshness | `report-ui/src` passes `tsc` (strict), and `toolbox-report.js` is its Vite bundle (D01M880QMWD7Z7FTGKJT99AHWBX) | The report reads the JSON `toolbox report` writes; embedded report is current | `npm run check` in `report-ui` |
| Story probe | Every report story over the T4T, OpenFGA and starter fixtures, at 1280 and 390 px, light and dark | No serious axe violation (WCAG 2.2 AA), no sideways scroll, no script error | `npm run probe` in `report-ui` |
| Standards | Toolbox for Toolbox scorecards over crates, docs, CLI and MCP (report, not yet a gate) | Docs, tests, clippy, size and complexity, interface contracts | `scripts/self-portal.sh report`; `toolbox test portal` |

Each row is recorded with its result in [research-log.md](../research-log.md) when a milestone
closes.

**Above the pyramid: DX Benchmarks** ([dx-benchmarks.md](./dx-benchmarks.md),
D01M82X2K0DYWTCA23MQJ4CKR0Z). The pyramid proves Toolbox behaves as specified. A DX Benchmark
asks whether that specification helps the agents that use it. Fresh agents do lifecycle work on
open-source forks, with Toolbox and with its strongest alternative, against a bar set before the
run. They aren't run on every change: they cost 0.5M–1.3M tokens each, so run one when a claim
needs proving or a release changes what a benchmark measured. Each benchmark measures one or more
of the four SDLC layers, and the catalog in `portal/benchmarks/` is scored by
`maintainer-benchmark`. They are the most effective way we have found to smooth Toolbox's rough
edges: every one so far has produced fixes.

### GitHub connector test proposal

Add a maintainer check for the central portal's GitHub ingestion path. Start a pinned Emulate
GitHub service on an ephemeral loopback port with synthetic users, private repositories and a
GitHub App. Send NDJSON refresh commands to the built `toolbox-connector-github` binary. Make
fixture changes through Emulate's REST API so a follow-up refresh must read the new state. Run
the check in its own `github-connector` CI job on pull requests and pushes to `main`.

| Scenario | Acceptance criterion |
|---|---|
| Scoped repository refresh | Reports the requested repository's archive status, default branch protection and direct admin, plus `not_found` for an unknown ID. |
| CODEOWNERS refresh | Reads `.github/CODEOWNERS` before the root and `docs/` files. Removing the preferred file changes the next refresh. A repository without CODEOWNERS returns `not_found`. |
| GitHub App authentication | Signs a JWT with an ephemeral key, exchanges it for a selected-repository installation token and uses the token to read private content. Emulate refuses access outside the installation. |
| Invalid token | A 401 produces an acknowledged, failed command with zero records. |

Emulate 0.14.0 lacks `GET /orgs/{org}/repos`, the three App inventory reads used by the connector
(`GET /orgs/{org}/installations`, `GET /user/installations/{id}/repositories` and `GET /apps/{slug}`),
and a public way to seed workflow history. Its collaborator list also ignores `permission` and
`affiliation` filters, so the Emulate tests seed only direct admins. The reference portal mock
supplies organization, App inventory and workflow responses. Recheck these gaps when upgrading Emulate.

Run the check from the repository root with Node 24 and the pinned Rust toolchain. The test
opens only a loopback listener and uses no external credentials, Postgres or OpenFGA.

```sh
cargo build --locked -p toolbox-connector-github
npm --prefix scripts/github-emulate ci --ignore-scripts
npm --prefix scripts/github-emulate test
```

## 5. Local prerequisites

| Tool | Used by | Note |
|---|---|---|
| Rust 1.97.0 | Everything | Pinned; dependencies needing a newer toolchain are isolated or rejected (D46) |
| Postgres 16 | M2–M4 acceptance | The scripts create a throwaway cluster (as the `postgres` user when run as root) |
| OpenFGA v1.22.1 (`TOOLBOX_OPENFGA_BIN`) | Authorization tests, M3–M4 acceptance | `go install github.com/openfga/openfga/cmd/openfga@v1.22.1` |
| Python 3 + `tools/requirements.txt` | Reference model, parity, mock GitHub, SCIM feeder | `cel-python`, `jsonschema`, `pyyaml` |
| yamark 0.6.1 | YAML formatting (D97) | `python3 -m pip install yamark==0.6.1` |
| Node 22 + Playwright | UI build, A11 browser check, report stories and probes | `npm install` in `ui/`; `npm ci` and `npx playwright install chromium` in `report-ui/` |

## 6. Standards

- **Every claim is checked.** No "should work": a test, an acceptance step or a measurement backs
  each statement in a commit or document.
- **The reference model leads.** Semantics change in `tools/reference.py` and the expectations
  first; the Rust implementation follows until the outputs are identical.
- **Decisions are logged, never rewritten** ([decisions.md](../decisions/README.md)).
- **No time estimates** (D25). Measurements of the software are welcome.
- **Commits are small and scoped**, so parallel agents rarely conflict.

## 7. Implemented vs. planned

| Capability | State |
|---|---|
| Workspace, pinned toolchain, fmt/clippy gates | ✅ |
| Reference-model differential tests, parity, conformance kit | ✅ |
| Acceptance scripts M2–M4, scale budget, UI freshness in CI | ⚠️ First observed 2026-11-20: acceptance had never run (no Postgres 16 on the runner; fixed); the scale budget fails on 2-vCPU runners (24.4 s vs 10 s; open decision). See [research-log.md](../research-log.md) |
| Acceptance for M5 | ✅ `scripts/m5-acceptance.sh` (no network, no Postgres) and `scripts/gitlab-acceptance.sh` (a real GitLab), both in CI |
| Acceptance for M6 | Planned with that milestone |
| Property tests over generated portals (P1–P13 beyond the reference portal) | Planned ([verification.md](./verification.md)) |
| Deterministic simulation | Deferred (D44) |
| Velocity tooling (hakari, affected tests, result cache) | Deferred until the loop regresses (D44) |

## Releasing (D90)

A version tag releases everything; nothing is built by hand.

1. Set `version` in the root `Cargo.toml` (`[workspace.package]`) and run `dist plan` to see the
   artifacts.
2. `git tag v<version> && git push origin v<version>`.
3. `.github/workflows/release.yml`, generated by `dist` from `dist-workspace.toml`, does the rest.
   It carries hand patches (job-level permissions, SHA-pinned installers, the image job), which
   `allow-dirty = ["ci"]` keeps: `dist plan` no longer compares the file to the generator, and
   `dist generate` would drop the patches, so reapply them after regenerating. Yamark does not
   format it (D98).
   - builds `toolbox` and each connector as its own app for Linux, macOS and Windows;
   - writes shell and PowerShell installers;
   - attests every artifact;
   - publishes a GitHub release.
4. The same run's last job, `custom-image` (dist's `post-announce-jobs`), calls
   `.github/workflows/image.yml`. It builds `ghcr.io/<owner>/toolbox:<version>` from that release's
   Linux archives, plus checksum-verified OpenFGA, and attests it. A `release: published` trigger
   would never fire: dist creates the release with `GITHUB_TOKEN`, and events from that token start
   no workflows. `gh workflow run image.yml -f tag=v<version>` rebuilds the image for an existing tag.
5. Consumers stop compiling: `actions/report` takes `toolbox-version: <version>` and downloads
   verified binaries.

**Checks:**
- Every action in both workflows is pinned to a commit SHA through `github-action-commits` in
  `dist-workspace.toml`.
- Verify an artifact with `gh attestation verify <file> --repo <owner>/toolbox`.

## Sources

- Measurements and acceptance results: [research-log.md](../research-log.md) (M0–M4 rows and the
  maintainer loop timings).
- Milestones and exit criteria: [v1-spec.md](../v1-spec.md) §3; properties §11.3.
- CI: [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml); acceptance scripts in
  [`scripts/`](../../scripts).
- GitHub integration dependency and route coverage in [Emulate v0.14.0](https://github.com/vercel-labs/emulate/tree/v0.14.0/packages/%40emulators/github/src).
- The repository's own scorecards for this layer: [toolbox-for-toolbox.md](./toolbox-for-toolbox.md).
