# Maintainer SDLC (Layer 1): Building Toolbox Itself

*Status: describes practice as of 2026-11-20 (M0–M4 done). Layer 1 of [the three SDLC
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
| Acceptance | One script per milestone, on real Postgres 16, OpenFGA, connectors and Chromium | Exit criteria | `scripts/m2-acceptance.sh`, `m3-…`, `m4-…` |
| Scale | 100k entities / 2M relations / 100 rules ≤ 10 s | Budget §10.4 | `cargo run --release -p toolbox-derive --example scale` |
| Format | Tracked `.yaml` and `.yml` match Yamark's fixed point (D97) | One style for embedded objects | `yamark format --check` |
| Freshness | `ui/dist` matches its source | Embedded UI is current | CI `ui` job |
| Standards | Toolbox for Toolbox scorecards over crates, docs, CLI and MCP (report, not yet a gate) | Docs, tests, clippy, size and complexity, interface contracts | `scripts/self-portal.sh report`; `toolbox test portal` |

Each row is recorded with its result in [research-log.md](../research-log.md) when a milestone
closes.

## 5. Local prerequisites

| Tool | Used by | Note |
|---|---|---|
| Rust 1.97.0 | Everything | Pinned; dependencies needing a newer toolchain are isolated or rejected (D46) |
| Postgres 16 | M2–M4 acceptance | The scripts create a throwaway cluster (as the `postgres` user when run as root) |
| OpenFGA v1.22.1 (`TOOLBOX_OPENFGA_BIN`) | Authorization tests, M3–M4 acceptance | `go install github.com/openfga/openfga/cmd/openfga@v1.22.1` |
| Python 3 + `tools/requirements.txt` | Reference model, parity, mock GitHub, SCIM feeder | `cel-python`, `jsonschema`, `pyyaml` |
| yamark 0.6.1 | YAML formatting (D97) | `python3 -m pip install yamark==0.6.1` |
| Node 22 + Playwright | UI build, A11 browser check | `npm install` in `ui/` |

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
| Acceptance for M5–M6 | Planned with those milestones |
| Property tests over generated portals (P1–P13 beyond the reference portal) | Planned ([verification.md](./verification.md)) |
| Deterministic simulation | Deferred (D44) |
| Velocity tooling (hakari, affected tests, result cache) | Deferred until the loop regresses (D44) |

## Releasing (D90)

A version tag releases everything; nothing is built by hand.

1. Set `version` in the root `Cargo.toml` (`[workspace.package]`) and run `dist plan` to see the
   artifacts.
2. `git tag v<version> && git push origin v<version>`.
3. `.github/workflows/release.yml`, generated by `dist` from `dist-workspace.toml` (regenerate with
   `dist generate`, never by hand), does the rest. `dist plan` compares that file to its own
   generator, so Yamark does not format it (D98).
   - builds `toolbox` and each connector as its own app for Linux, macOS and Windows;
   - writes shell and PowerShell installers;
   - attests every artifact;
   - publishes a GitHub release.
4. `.github/workflows/image.yml` then builds `ghcr.io/<owner>/toolbox:<version>` from that release's
   Linux archives, plus checksum-verified OpenFGA, and attests it.
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
- The repository's own scorecards for this layer: [toolbox-for-toolbox.md](./toolbox-for-toolbox.md).
