# actions-gitlab round 0: the dry run

*Not counted (README §4 step 4, dx-benchmarks §4.1: a dry run finds friction before the bar is
tested). One run per arm, at commit `8f7026d`, on the prompts in
[prompts/rendered/r0](../../prompts/rendered/r0/). About the box and the forge. Measured by the
harness (`harness/measure.sh`), not by the agents; nobody probed these runs.*

## Result

Both arms delivered: the three CODEOWNERS rules are on `main` of alpha, beta and gamma (probe C1
passes in both). Neither touched another group or round, and the GitLab arm never reached for
Seikika (isolation all zero).

| Measure | `seikika-1` | `gitlab-1` | Source |
|---|---|---|---|
| C1, the rules on every project's `main` | pass | pass | `*.check.json` |
| Lines to maintain, as counted | 1,725 | 1,182 | `*.lines.json` |
| Struck as runtime data | 1,364 (`service/state/`: run records and clones; `service/home/`: the bot's git credentials) | 0 | |
| Lines to maintain | 361 | 1,182 | |
| Tool calls | 56 (38 shell, 15 edit, 3 read) | 57 (43 shell, 14 edit) | `*.count.json` |
| Wall time | 628 s | 1,244 s | `*.count.json` |
| GitLab settings changed | 5 | 22 | `*.settings.json` |
| Prompt as rendered | identical | identical | `*.prompt.json` |

The agents' own accounts are [seikika-1.HANDOFF.md](./seikika-1.HANDOFF.md) and
[gitlab-1.HANDOFF.md](./gitlab-1.HANDOFF.md).

**The Seikika arm** followed `examples/federated-actions/gitlab-org`: an org portal with the
`gitlab` connector, the three projects as children by URL, a `change` with `land: push` per
project and `['group:acme-w1/release']` as approvers. Of its 361 lines, 107 are glue:
`bin/setup-service` (42, a group access token for the bot, a `HOME` with its git credentials, a
state directory) and `bin/acme-actions` (65, a wrapper that asks GitLab who owns the caller's token
and passes the answer as `--as`, refusing `--as` itself).

**The GitLab arm** built the action out of GitLab: request pipelines on a branch with
`spec:inputs`, a manual `approve` job, and an executor pipeline on protected `main` started by a
bot-owned trigger, which rechecks who played the job against the `release` group's direct members
through the API before it commits. Its glue is `org/bin/acme-actions` (454 lines),
`set_codeowners.py` (417, copied into each project) and `admin/setup.py` (166). It reached
GitLab's identity for free: GitLab records who started a pipeline and who played a job.

## Friction, and what closed it

Each item is general, not shaped to this task (P5): a team running any governed change on GitLab
meets it.

| # | Arm | Friction | Fix |
|---|---|---|---|
| G1 | both (harness) | Auto DevOps started a pipeline on every imported project with no `.gitlab-ci.yml`; on this runner they fail and hold a slot, and the GitLab arm spent calls telling them from its own | `prepare.sh` turns Auto DevOps off instance-wide, so the only pipelines are the ones a run starts |
| T1 | seikika (product) | A checkout takes `--as` on trust, so the agent wrote a 65-line wrapper to bind each request and approval to the person GitLab says owns the token. Any team approving in a shared checkout or a CI job needs it; the GitLab arm got it from GitLab | `forges[].identity: true` and `seikika run --identity-token-file` (D01M87FMCVSQCQJMGDSN2MCTH5W): Seikika asks the forge who owns the token, refuses `--as` alone or against another's token; acceptance step 9 checks it on the real GitLab. The `run` and `add-action` skills say when to use it |
| T2 | seikika (product) | The bot's git credentials had to go in a separate `HOME`, because Seikika's clones and pushes use git's own credential helpers | Not changed. Putting the forge token in git's configuration was rejected in D01M87AXG095Q0Z0DGGTKYYAM7Y (it would reach action commands through `GIT_CONFIG_*`); an askpass helper that only Seikika's own git calls see is the candidate, recorded as a caveat in [forges.md](../../../../docs/design/forges.md) §7 |

The GitLab arm met no friction of its own beyond G1: its prompt names the Premium features it
lacks and how they are credited, and it said which steps Premium would have performed.

Round 1 reruns both arms at the commit that carries these fixes.

## How to check

The measures are rerun from a run's transcript (fetched with `batch-fetch-details`) against the
GitLab the harness started; the product fix T1 is checked by the acceptance script's step 9
([D01M87FMCVSQCQJMGDSN2MCTH5W](../../../../docs/decisions/D01M87FMCVSQCQJMGDSN2MCTH5W.md),
[forges.md](../../../../docs/design/forges.md) §4.6):

```sh
bash experiments/actions-gitlab/harness/measure.sh 0 seikika-1 <transcript-dir>
scripts/gitlab-acceptance.sh
```
