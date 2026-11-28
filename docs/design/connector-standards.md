# Connector Standards: Cataloguing and Scoring the Connectors We Ship

*Status: deterministic catalog and scorecard built, 2026-11-20 (D73). Seikika for Seikika catalogues
every first-party connector crate and scores it with `maintainer-connector`. Runtime health (lag,
error rate, permission probes) stays in the meta-catalog design and is not gated. Related:
[connector-control.md](./connector-control.md), [meta-catalog.md](./meta-catalog.md),
[seikika-for-seikika.md](./seikika-for-seikika.md).*

---

## 1. Conclusion

**Catalogue every connector we ship, and gate the checks a reader of the source can make.**

A connector is a process that speaks Seikika's control channel and emits records of named kinds.
The questions worth separating:

| Question | What answers it | In this repo |
|---|---|---|
| Can an operator find it and learn how to configure it? | A public setup guide, package description, crate docs, and a documented config | `maintainer-connector` bronze |
| Does a resync close, and does a refresh admit when an id is gone? | The conformance kit's message rules, visible as source shapes | silver |
| Does CI actually run it, and is a credential pasted into the tree? | A `connector test` line, or an acceptance script for a push connector; secret shapes | gold |
| Is it healthy in production? | Lag, error rate, a permission probe | **Not gated.** That is `connector-health` in [meta-catalog.md](./meta-catalog.md), and it needs a live portal |

The workspace connector emits one `connector` record per `crates/seikika-connector-*/Cargo.toml`.
It reports facts. `.seikika/standards/connectors.yaml` holds the thresholds. A connector added after this baseline
must reach bronze.

**What we did not adopt, and why:**

| Proposal | Why it is not the gate |
|---|---|
| Airbyte's connection specification and acceptance-test credentials | Airbyte's spec is a JSON Schema a user fills in, and the acceptance suite needs sandbox credentials. Seikika connectors are configured by the portal manifest. The part we take is the same idea: one protocol, one test suite, secrets not pasted into the tree |
| Singer's SCHEMA message and `key_properties` | A tap describes streams at runtime. Our kinds are declared in `capabilities()` and checked by `seikika connector test` against the manifest. Requiring a second schema document would duplicate that |
| Ocean's `spec.yaml` plus `@ocean.on_resync` | The same contract (kinds, config, a resync per kind), expressed in Python. We check the Rust shape of that contract instead of adopting Ocean's files |
| Meta-catalog `connector-health` bronze through gold as written | Owner, digest pin, lag, permission probe, identity-link coverage and a 7-day error rate are runtime facts. This portal has no running connector to measure. The gold line "contract tests pass in CI" is the part that is a property of the tree, so it is the gold rule here |

## 2. What the field measures

Read 2026-11-20. "Primary" means the repository or the spec text.

| Source | What it decides | What is deterministic | What needs a run |
|---|---|---|---|
| **Seikika conformance kit** (`seikika connector test`) | Register first. Capabilities name only kinds the manifest declares. A resync of each kind emits records, `snapshot_complete` and a completion. A refresh returns exactly the requested ids. An unknown id is `not_found` | The source either contains those calls or it does not. CI either invokes the binary or it does not | The kit itself, against a built binary |
| **Seikika control channel** | `run_stdio` for an in-image connector, `run_remote` for long-poll. Push connectors register without refresh or resync. A refresh never deletes by omission | Which entry point the crate calls; whether the crate doc says push-only | Delivery, coalescing, deadlines |
| **Airbyte protocol and acceptance tests** | `connectionSpecification` is JSON Schema. Spec tests check backward compatibility and that secret fields are marked `airbyte_secret`. CI runs the acceptance suite when credentials exist | Spec file present; secret fields marked | `check`, `discover`, `read` against a sandbox |
| **Singer spec 0.3.0** | A tap writes SCHEMA then RECORD. SCHEMA requires `stream`, `schema` and `key_properties`. Discovery mode writes a catalog of streams | The message shape | A sync against the source |
| **Port Ocean** | `.port/spec.yaml` lists kinds and config (secrets marked `sensitive`). Each kind has an `@ocean.on_resync` handler | Spec and handlers agree | `ocean sail` against the API |
| **Meta-catalog `connector-health`** | Owner, pin, freshness, least privilege, error rate, contract tests | The contract-test line | Lag and error rate |

The same split shows up everywhere. Static checks decide whether the package is identifiable,
configured in the open, and shaped like the protocol. A run decides whether the upstream answers.
This scorecard is the static half. The conformance kit remains the run, and CI already executes it
for the pull connectors.

## 3. The catalog

`workspace` grows an opt-in `connectors` glob (this portal: `crates/seikika-connector-*/Cargo.toml`).
One `connector` record per manifest. The id is `connector:<crate directory>`.

| Fact | What it is |
|---|---|
| `name`, `description`, `descriptionChars`, `path`, `dir` | Package name and the `description` field of `[package]` |
| `crateDoc`, `configDocumented` | A `//!` block on `src/main.rs` (or `src/lib.rs`). Config is documented when that block names `SEIKIKA_CONNECTOR_CONFIG`, `--seikika` or `--listen` |
| `publicGuide` | The public `/connectors/<name>/` route, or null if it cannot be verified. The crate's `seikika-connector-` suffix names its guide. The workspace collector checks for a titled and described directory and guide, and sidebar visibility. |
| `transport` | `stdio` if the source calls `run_stdio`, `remote` if it calls `run_remote`, `both` or `none` |
| `pushOnly` | The crate doc says push-only, or the crate calls `run_remote` and not `run_stdio` |
| `documentedKinds`, `genericKinds` | Kinds from `record("…")`, `snapshot_complete("…")`, `"kind": "…"`, and doc bullets. `genericKinds` is set when the doc says kinds come from job config (`kind \`<\`` or `declares their \`kinds\``), which is how the runner works |
| `snapshotComplete`, `reportsNotFound` | The source contains those calls. A push connector is not required to |
| `tests` | Count of `#[test]` (including `#[tokio::test]`) in the crate's `.rs` files |
| `conformanceMentions`, `ciMentions` | Lines under `scripts/` or `.github/workflows/` that name the package and `connector test`, and files there that name the package at all |
| `secretHits` | High-precision shapes only (`AKIA…`, `ghp_…`, `github_pat_…`, `sk-ant-` / `sk-proj-` / `sk-live-`, `xox[baprs]-`), as `file:line kind`, with the secret left out |

## 4. The scorecard

`maintainer-connector`, kind `Connector`, levels bronze / silver / gold. Maintainer SDLC: we are
the authors of these connectors.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `described` | A non-empty Cargo description and a crate doc |
| bronze | `config-documented` | The crate doc names the config env or the remote CLI flags |
| bronze | `public-guide` | The connector's guide appears in the public connector directory and sidebar |
| bronze | `speaks-the-protocol` | `transport` is `stdio`, `remote` or `both` |
| silver | `names-its-kinds` | At least one documented kind, or the runner's generic-kinds note |
| silver | `closes-snapshots` | The source sends `snapshot_complete`, or the connector is push-only |
| silver | `reports-missing-ids` | The source sends `not_found`, or the connector is push-only |
| silver | `has-tests` | At least one `#[test]` |
| gold | `exercised-in-ci` | A CI line runs `connector test` on the binary, or the connector is push-only and an acceptance script starts it |
| gold | `no-hardcoded-secrets` | No secret shape in the crate sources |

A new connector below bronze is a ratchet regression. Add an operator landing page under
`npm/site/src/content/docs/connectors/`. Give the page a title and description. The directory and
sidebar pick up published landing pages automatically. Put additional pages under the connector's
path and link them from the landing page. Use the [documentation style guide](../../npm/site/src/content/docs/maintainers/writing.md)
to review setup, record examples, verification, and recovery steps. The public guide rule checks
presence and discoverability. A guide may link to more pages when the connector needs them.
See [connector directory research](../audits/connector-directory.md) and
[the public guide decision](../decisions/D01M8CHMD6TH2AXRWT9E49G4V6V.md).

## 5. This repository

On November 26, 2026, `seikika report .seikika --skip runner,github-actions` found that all six
first-party connectors passed `public-guide` and reached gold. The directory links to the guides
for GitHub, GitLab, Git files, Workspace, Runner, and SCIM. [Connector directory](../../npm/site/src/content/docs/connectors/index.mdx)

`seikika test .seikika` pins four fixture connectors: `crates/demo/focused` at gold,
`crates/demo/partial` at bronze, `crates/demo/unsafe` and `crates/demo/unlisted` at none. The
unlisted fixture has crate docs and fails the public guide rule. Every rule passes on one fixture and
fails on another, so the new rules and the scorecard clear `meta-rule` / `meta-scorecard` from
this commit (D71).

## Sources

- Seikika conformance kit, `crates/seikika-connect/src/conformance.rs` (the five checks). Control
  channel: [connector-control.md](./connector-control.md). The runtime scorecard this one does not
  implement: [meta-catalog.md](./meta-catalog.md) §7.2.
- Airbyte protocol, `docs.airbyte.com/platform/understanding-airbyte/airbyte-protocol`, and the
  acceptance-test reference, `docs.airbyte.com/platform/connector-development/testing-connectors/connector-acceptance-tests-reference`,
  fetched 2026-11-20. Spec tests cover backward compatibility and `airbyte_secret` markers.
- Singer specification 0.3.0, `singer-io/getting-started` `docs/SPEC.md` and `docs/DISCOVERY_MODE.md`,
  read 2026-11-20. SCHEMA requires `stream`, `schema` and `key_properties`.
- Port Ocean custom-integration docs (`spec.yaml` kinds and configuration, `@ocean.on_resync`),
  `docs.port.io` pages indexed 2026-11-20. The site is often blocked here; the excerpts are from
  the indexed pages, and the local review is [ocean.md](../ocean.md).
