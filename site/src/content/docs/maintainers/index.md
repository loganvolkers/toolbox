---
title: Build Toolbox
description: A maintainer changes this repository and proves the change against the reference portal and the scorecards.
sidebar:
  order: 1
---

A maintainer changes Toolbox itself: the Rust workspace, the connectors shipped in the binary, the schemas, and this site. The contract is the v1 spec and the reference portal. A portal team's manifest is the [operator](/toolbox/operators/) loop. A scorecard published for other repositories to include is the [package author](/toolbox/authors/) loop, including when that package lives in this repository.

The maintainer SDLC is how a change lands here. Every milestone exit is a command, so a change is done when the command passes.

## Prove a derived-state change

`examples/reference-portal` is the oracle. Its `expected/` directory is the derived catalog the pipeline must reproduce byte for byte: entities, scores, tuples, options. `toolbox derive` writes that catalog. `cargo test -p toolbox-derive` compares it. `scripts/validate-parity.py` compares `toolbox validate` with the Python validator on the same accept and reject cases.

```bash
cargo build
toolbox validate examples/reference-portal
toolbox test examples/reference-portal
cargo test -p toolbox-derive
python3 scripts/validate-parity.py
```

A change that moves one field in the output fails the byte compare. Regenerate `expected/` only after reading the diff.

## Benchmark a claim about developer experience

Tests prove that Toolbox behaves as specified. A DX Benchmark asks whether that helps the agents that use it. Fresh agents do lifecycle work on open-source forks, with Toolbox and with its strongest alternative, against a bar written before the run. A claim that Toolbox makes something easier needs one. Each benchmark has an entry in `portal/benchmarks/`, which `maintainer-benchmark` scores, and is archived once it has served its value. See [DX Benchmarks](/toolbox/concepts/dx-benchmarks/).

## Keep the inner loop short

On a 4 vCPU container with warm caches, after milestone M4, the recorded times are: rebuild after editing `toolbox-derive` 2.1 s, `cargo test` 3.6 s, `toolbox test examples/reference-portal` 0.2 s, `cargo clippy --all-targets -- -D warnings` 4.7 s. The acceptance script that drives Postgres, OpenFGA, the connectors and a browser (`scripts/m4-acceptance.sh`) is 7.9 s.

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
toolbox test portal
```

`toolbox test portal` runs this repository's own expectations: the rules, scored against fixture entities, both the passing cases and the failing ones. Acceptance scripts for milestones M2, M3 and M4 need Postgres 16. M3 and M4 also need OpenFGA (`TOOLBOX_OPENFGA_BIN`). M4 needs Node and Playwright for the browser check.

The toolchain is pinned in `rust-toolchain.toml`. YAML in the repository is formatted with Yamark 0.6.1 (`yamark format --check`). The public [CLI reference](/toolbox/start/cli/) is an Astro page built from the live binary (`toolbox describe`); `npm run check:cli-docs` in the pages workflow fails if the committed fingerprint is stale.

## Stop a standard from regressing

This repository is also a portal. `portal/` declares the standards for crates, docs, workflows, skills, this site's pages, and the four SDLC layers.

```bash
toolbox report portal --skip runner,github-actions --baseline portal/results.json --ci
```

`--ci` fails when an entity drops a level, when a failing rule's evidence grows, when a new entity is below the floor (bronze for these standards), and when the baseline is missing an improvement. `--update` rewrites `portal/results.json` after you have looked at the diff. The HTML report published on this site is the same run; CI compares the committed files with a fresh one.

The test runner and the GitHub API are skipped in that fast check. A rule that needed them is unknown, and it names the connector that was skipped. The full report, including those connectors, is the slow path.

[This repository, scored](/toolbox/demo/report/) is that report, in the site's navigation.

## Record the decision, then change the contract

A decision is one file, `docs/decisions/<id>.md`. The id is `D` plus a ULID, minted with `python3 scripts/decision-id.py`. Filename order is time order. An existing decision stays as it was written. A change is a new file, and the design doc it points at is updated in the same change.

The v1 spec says what the binary must do. The design docs say why. When they disagree, the decision file wins, then the spec.

## Where the loop stands

| Step | What you run | State |
|---|---|---|
| Decide | A new file in `docs/decisions/` | Built |
| Build | `cargo build`, `npm --prefix ui run build`, `npm --prefix site run build` | Built |
| Test | `cargo test`, clippy, `toolbox test`, the parity script | Built |
| Verify a milestone | `scripts/m2-acceptance.sh`, `m3-acceptance.sh`, `m4-acceptance.sh` | Built through M4. M5 (actions) is paused |
| Release | `dist plan`, tag `v0.1.0`, the release and image workflows | First installable release: [v0.1.0](https://github.com/loganvolkers/toolbox/releases/tag/v0.1.0). See [docs/RELEASING.md](https://github.com/loganvolkers/toolbox/blob/main/docs/RELEASING.md) |
| Watch this repository | `toolbox report portal` and the ratchet | Built |

[Run Toolbox locally](/toolbox/start/run/) is the short form of the build. [The four SDLC layers](/toolbox/concepts/layers/) says which loop a sentence is about.
