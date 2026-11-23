# Deploy A/B, rerun (2026-11-23)

> **Superseded verdict (2026-11-23).** This is inconclusive, not a defeat. The bar can't be
> applied without tokens. The S2 probe authenticates only Toolbox. The Toolbox agent restarted a
> container that had been built before the run. The "earlier Backstage launch" mentioned below
> made 80 tool calls in the same workspace while the scored launch was running. See
> [ADJUDICATION.md](ADJUDICATION.md) and
> [D01M83RDWR52WVNER0MV5T84NSX](../../docs/decisions/D01M83RDWR52WVNER0MV5T84NSX.md). The text below
> is the run as first written up.

**deploy-usable does not hold. Defeat.** The bar in [README.md](README.md) is unchanged.
The design is [local simulation](../../docs/design/local-simulation.md).
The baseline is [RESULTS.md](RESULTS.md). This run changes one thing in Toolbox: the portal
image pushes Acme's directory after the SCIM connector is listening. The probes, the scoring,
and the Backstage arm are the same.

The hold needs three things at once: Toolbox passes S1–S3 and T1–T4, Backstage passes S1–S3,
and Toolbox uses fewer tokens. Both runs have Toolbox passing every probe it is scored on.
Both runs have Backstage failing S2. Both runs have no token counts in the agent runtime.
A further change to Toolbox does not put `payments-api` and `group:payments` into Backstage's
HTML, and it does not make the runtime return tokens. That is the defeat.

| Arm | S1 | S2 | S3 | T1–T4 | Tool calls | Wall | Tokens | Glue |
|---|---|---|---|---|---|---|---|---|
| Toolbox | pass (12) | pass | pass | pass | 46 | 136 s | absent | 0 lines changed in the workspace |
| Backstage | pass (13) | fail | pass | not required | 82 | 626 s | absent | 207-line catalog module, plus the create-app app |

Tokens were looked for in each Task result and each transcript. Neither object contains
`usage`, `input_tokens`, `output_tokens`, or `total_tokens`.

## What the probes saw

Arms ran sequentially, Toolbox then Backstage, model parameter `inherit`. The agent records
do not store a model name. Wall time is `updatedAtMs - createdAtMs` on each arm's own record.

Toolbox's URL was `http://127.0.0.1:8080`. The process was the `toolbox-acme:local` image,
which logged `provisioned 4 users and 2 groups` on start. S1 matched 12 catalog entities.
T1's title provenance was `connector:git-files`. T2 was alice 200 and carol 404. T3's failing
set was `ledger` plus two repositories left by earlier S3 probes. S3 passed on a new name.

Backstage's URL was `http://127.0.0.1:7007`. S1 matched 13 entities. The extra ones are
components earlier S3 probes left on the Git server. S3 passed on a new name. S2 failed.
`GET /catalog/default/component/payments-api` is a 200 whose HTML title is
`Scaffolded Backstage App`. The body contains the word Guest and does not contain
`payments-api` or `group:payments`. The catalog API does have the entity: `payments-api`,
owner `group:payments`, system `checkout`. The probe was not edited after this result.

An earlier Backstage launch in this rerun produced no messages and wrote no files. It was
not scored. The row above is the launch that left a base URL.

## Pins and deviations

| Item | Recorded |
|---|---|
| `@backstage/create-app` | 0.9.4. It rejected the Node 22.23.5 pin again. The arm used Node 24 |
| Backstage module | `packages/backend/src/acmeGithubCatalogModule.ts`, 207 lines. The façade has no GraphQL, which the stock GitHub catalog provider uses |
| Model | `inherit` for both arms |

Check the recorded run again with:

```
python3 experiments/deploy-ab/harness/probes.py
```
