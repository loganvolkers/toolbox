# Deploy A/B: Toolbox and Backstage on a laptop

**Question:** can a fresh agent deploy a usable portal, with Toolbox and with Backstage, when the
only services on the machine are a local GitHub and Postgres?

**Status: blocked, not run.** The bar below is the bar. It changes only if the design changes
before the run, and then the change is recorded. Results are not written yet.

This tests the box's deploy contract, not a tool
([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md)). The environment is
[local-simulation.md](../../docs/design/local-simulation.md) (Q19). Operator SDLC deploy and
verify, and user SDLC discover.

## 1. Arms

Both arms get the same machine: Docker, a local registry, Postgres, and the façade URL from the
design. The portal is not started for them. Neither arm gets a GCP, AWS or Azure account. Neither
arm gets the probe source, this file's expected-value section (there is none: values are computed
at score time), or `docs/research-log.md`.

| | Toolbox | Backstage |
|---|---|---|
| Given | The `toolbox` CLI, the Score file, the Helm chart, and the operator docs | `@backstage/create-app` at the pin recorded when the run is unblocked, Node and Yarn at the pins in [spec-ab](../spec-ab/README.md) §2, and Backstage's deployment docs |
| Leaves | A base URL whose process is the portal image | A base URL whose process is the Backstage backend image |
| Catalog | github and git-files connectors aimed at the façade, SCIM aimed at the feed | The GitHub catalog provider, or locations, aimed at the same façade. Sample `examples/entities.yaml` does not satisfy the probes |

The harness stops the agent before scoring.

## 2. What is scored

Shared probes S1–S3 and Toolbox probes T1–T4, as defined in
[local-simulation.md](../../docs/design/local-simulation.md) §7. The Backstage arm is scored on
S1–S3 only. `/readyz` is recorded and is not a pass.

S3 pushes a `catalog-info.yaml` whose component name is chosen after the agent exits, with a
credential the agent was not given.

The check, once L2 exists:

```bash
python3 experiments/deploy-ab/harness/probes.py
```

That file is not in the tree. Adding a script that exits 0 without running S3 would be the failure
this benchmark exists to catch.

## 3. The bar, set before running

| Hypothesis | Holds if | Fails if |
|---|---|---|
| **deploy-usable** | Toolbox passes S1–S3 and T1–T4, Backstage passes S1–S3, and Toolbox uses fewer tokens | Toolbox misses a shared probe that Backstage passes, or Toolbox passes the shared probes with at least as many tokens |

A run where the only green check is `/readyz` is void: it is not evidence for either side.

Also recorded, and not part of the hold/fail line: tool calls, wall time, and lines of glue. A
token win with a portal that fails S3 is a fail.

## 4. Repositories

Mirrored into the façade at these commits. The harness may add a `catalog-info.yaml` commit on top
of OpenFGA, which has none, and records that commit in the run notes.

| Repository | Commit |
|---|---|
| `backstage/backstage` | `feeadd4d4efcbda9f1573b95e356e0f3fc0b5d17` |
| `openfga/openfga` | `d67ba7b1d2eaf945cffab3a9129ca1a4c7ad5251` |

## 5. Before running

| # | Prerequisite | State |
|---|---|---|
| P1 | `serve` binds `0.0.0.0:$PORT` when `PORT` is set | Not started. Today the default is `127.0.0.1:8080` |
| P2 | L2 stack: Forgejo, façade, SCIM, Postgres, image, no `--fixtures` | Not started |
| P3 | L4 usability script passes on that stack with no agent | Not started |
| P4 | Q18: approval to run `npx @backstage/create-app` | Open. A previous session was refused |

## 6. Limits, written before the run

- One fresh agent per arm.
- floci-gcp is not part of this benchmark. It is a later Toolbox-only check of the Cloud Run Admin
  API, and it was not executed while this was written.
- The comparison is a laptop deploy (Compose or the chart on kind). It is not a comparison of
  `gcloud run deploy`.
