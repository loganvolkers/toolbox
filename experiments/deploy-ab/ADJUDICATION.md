# deploy-ab: independent adjudication (2026-11-23)

**Outcome against the bar in [README.md](./README.md) §3: deploy-usable is inconclusive in both runs.
It did not hold, it did not fail, and it was not a defeat.** The hold needs Backstage to pass S1–S3
and Seikika to use fewer tokens. The fail needs Seikika to miss a shared probe that Backstage passes,
or to pass the shared probes on at least as many tokens. Neither happened. No tokens were recorded,
so the token clause can't be scored in either direction. This also does not count against Backstage:
the arms were not given equal conditions, as set out below. This supersedes the "Defeat" header of
[RESULTS-v2.md](./RESULTS-v2.md) and the catalog's earlier outcome. The probe outputs are unchanged.

The adjudicator is a cloud agent that did not design the experiment, build an arm, or change
the product ([D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md), P7).
The decision that records this outcome is
[D01M83RDWR52WVNER0MV5T84NSX](../../docs/decisions/D01M83RDWR52WVNER0MV5T84NSX.md). The prompts
were recovered from the agent records into [prompts/](./prompts/README.md).

## Findings

| # | Finding | Evidence | Protocol 2 rule |
|---|---|---|---|
| 1 | **No tokens, and the bar is written only in tokens.** The cost clause can't be scored either way. A missing measure should have been found in a dry run, before the bar was fixed | `usage.tsv`, `usage-v2.tsv`; [RESULTS.md](./RESULTS.md) | P6 |
| 2 | **The probes give only Seikika an identity.** S2 sends Seikika `x-seikika-user` and `?as=`. It sends Backstage nothing. If Playwright is installed it clicks a "guest" button, and the results don't say whether that render ran. Backstage failed S2 because its page was the sign-in shell, while its catalog API had `payments-api` with owner `group:payments`. Neither prompt asks the agent to set up sign-in | `harness/probes.py` (`s2`, `render`); RESULTS-v2.md "What the probes saw"; [prompts/](./prompts/README.md) | P4 |
| 3 | **The Seikika deploy was done before the run.** The prompt says "The portal is not running" and "The database is empty". Both Seikika agents report that the `seikika-acme:local` image and the `acme-portal-1` container "were already correct" and had only stopped, and that Postgres "already had the portal schema". Both restarted the container. In each run, Seikika's 0 lines of glue measure a restart | Final messages of `bc-51601c2d…` and `bc-514b615a…`; [prompts/seikika.md](./prompts/seikika.md) | P4 |
| 4 | **The arms got different starting material.** Seikika got a Score file, a Helm chart, a portal image and operator docs written for this stack (`examples/acme-live`). Backstage got `create-app` and a pointer to "the GitHub catalog provider". That provider needs GraphQL, and the facade has none, so the agent wrote a 207- or 208-line catalog module. The prompt also pinned Node 22.23.5, which `create-app` 0.9.4 rejects. That happened in both runs | [prompts/backstage.md](./prompts/backstage.md); RESULTS-v2.md "Pins and deviations"; `usage.tsv` | P4 |
| 5 | **Two Backstage launches overlapped in the rerun.** RESULTS-v2.md says an earlier launch "produced no messages and wrote no files". Its record, `bc-49ebdbc7…`, holds 80 tool calls. They start about 3 minutes before those of the scored launch, `bc-c748ae14…`, and end 25 seconds before its last one, so the two ran together for about 10 minutes. Both worked in `/tmp/deploy-ab/arm-backstage`, and both report writing `http://127.0.0.1:7007` | Tool-call timestamps in the two agent records | P2, P3 |
| 6 | **One run per arm, and the prompts weren't committed.** The prompts are now in [prompts/](./prompts/README.md), recovered from the agent records and updated to use the current project name | — | P1, P2 |

## Per hypothesis

| Hypothesis | First run | Rerun |
|---|---|---|
| deploy-usable | **Inconclusive.** Seikika passed S1–S3 and T1–T4. Backstage failed S2. No tokens | **Inconclusive.** Same probe results, no tokens, and the Backstage arm was contaminated (finding 5) |

Even with tokens, the result would not have been a fair comparison until findings 2–4 were fixed.
Seikika's deploy path worked on this stack, and that is worth recording. What the two runs do not
show is that an agent deploys Seikika more cheaply than Backstage.

## What the benchmark is still worth

The work it forced stays: the `0.0.0.0:$PORT` bind, the Score file and the chart, the Acme seed,
OpenFGA's tuple view under a `search_path`, and the image pushing the directory after the SCIM
connector is listening ([local-simulation.md](../../docs/design/local-simulation.md) §10).

## Before a protocol-2 rerun

1. The harness removes any portal image, container and schema before the Seikika arm starts, so
   the prompt's "not running" and "empty" are true.
2. S2 authenticates both arms the same way. One option is a Backstage guest sign-in that the
   harness enables, as Seikika's reader header does. The other is that both prompts ask for
   sign-in and the probe signs in to both.
3. The Backstage prompt points at a catalog source that works against the facade, and pins a Node
   that `create-app` accepts.
4. The bar is written in a measure a dry run shows the runtime reports, such as tool calls and
   wall time.
5. At least three runs per arm, one launch at a time, and adjudication by someone other than the
   designer.
