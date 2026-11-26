# Deploy A/B, first run (2026-11-23)

> **Adjudicated (2026-11-23): inconclusive.** "Does not hold" below means neither the hold nor the
> fail. The arms also did not get equal conditions: S2 authenticated only Seikika, and Seikika's
> image and container existed before the run. See [ADJUDICATION.md](ADJUDICATION.md).

**deploy-usable does not hold.** Seikika passed S1–S3 and T1–T4. Backstage passed S1 and S3 and
failed S2. The agent runtime returned no token counts, so the token clause was not scored. This is
not the pre-registered fail either: Seikika did not miss a shared probe that Backstage passed.

The bar in [README.md](README.md) is unchanged. Arms ran sequentially, Seikika then Backstage.
Wall time below is each arm's own agent record (`updatedAtMs - createdAtMs`), not time spent
waiting on the other arm. Both arms were launched with the same model parameter, `inherit`. The
Seikika task record stores that as `grok-4.7`.

| Arm | S1 | S2 | S3 | T1–T4 | Tool calls | Wall | Tokens | Glue |
|---|---|---|---|---|---|---|---|---|
| Seikika | pass (10) | pass | pass | pass | 38 | 178 s | absent | 0 lines changed in the workspace |
| Backstage | pass (11) | fail | pass | not required | 138 | 1063 s | absent | 208-line catalog module, plus the create-app app |

Tokens were looked for in each Task result, each transcript, and each cloud-agent record. None of
those objects contains `usage`, `input_tokens`, `output_tokens`, or `total_tokens`. No number is
filled in here.

## What the probes saw

The harness started Forgejo, the façade, the registry and Postgres, and did not start the portal.
Each agent wrote a base URL. Scoring ran after that agent had exited, with
`python3 experiments/deploy-ab/harness/probes.py`.

Seikika's URL was `http://127.0.0.1:8080`. The process was the `seikika-acme:local` image. S1
matched 10 catalog entities (the Acme seed, the Backstage mirror's root component, and the
OpenFGA component the harness added). T1's title provenance was `connector:git-files`. T2 was
alice 200 and carol 404 on `component:payments-api`. T3's failing set was `ledger` plus the
repository S3 had just pushed, which has no README. T4's MCP `get` returned that component.

Backstage's URL was `http://127.0.0.1:7007`. S1 matched 11 entities. The extra one is the
component S3 pushed during the Seikika score; it was still on the Git server, and the probe
compares the portal with the Git server at score time. S3 for Backstage passed on a new name.
S2 failed. `GET /catalog/default/component/payments-api` is a 200 whose HTML is the guest
sign-in shell (`Scaffolded Backstage App`, the word Guest, and ENTER). It does not contain
`payments-api` or `group:payments`. The probe clicks the first text matching `guest` and waits
1.5 s; the page is unchanged. The catalog API does have the entity: `payments-api`, owner
`group:payments`, system `checkout`. The probe was not edited after this result.

## Pins and deviations

| Item | Recorded |
|---|---|
| `@backstage/create-app` | 0.9.4, the version npm served on 2026-11-23. The Backstage agent ran it. Q18's earlier refusal did not recur |
| Node | The arm was given 22.23.5, the spec-ab pin. create-app 0.9.4 rejected Node 22 and the agent built with Node 24.22.3 |
| Yarn | 1.22.22 |
| Backstage mirror | `feeadd4d4efcbda9f1573b95e356e0f3fc0b5d17` |
| OpenFGA mirror | `d67ba7b1d2eaf945cffab3a9129ca1a4c7ad5251`, plus harness commit `846caa914481d402496ef392fd238870d27fc10d` adding a root `catalog-info.yaml` |
| Model | `inherit` for both arms |

kind was not run. `score-compose` was not run. The Seikika agent started the existing portal
image with Compose. The Backstage agent could not use Backstage's GitHub catalog provider: the
façade has no GraphQL, which that provider uses to list repositories, so the agent wrote a
backend module that reads the façade's REST repo list and `catalog-info.yaml` files.

## Before the agents

`python3 experiments/deploy-ab/harness/probes.py` passed S1–S3 and T1–T4 on the Acme seed with
no `--fixtures`. `--suite l4` passed S2, T3 and S3. The published portal port answered
`/readyz`. Those checks are the pre-registered prerequisites, not the benchmark result. The
benchmark result is the table above.
