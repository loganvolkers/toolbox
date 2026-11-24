# actions-ab round 0: the Toolbox arm's dry run

*Not counted (dx-benchmarks §4.1: a dry run finds friction before the bar is tested). One Toolbox
run, at commit `cf61bb0`, on the round 0 prompt in
[prompts/rendered/r0](../../prompts/rendered/r0/toolbox-1.md). About the box. Measured by the
harness, not by the agent.*

## Result

The run delivered: `DEPRECATED.md` is on `main` of all three origins with the required content
(probe C1 passes, [toolbox-1.check.json](./toolbox-1.check.json)). It did so by building around
Toolbox, not with it: 824 lines to maintain, 425 of them a CLI the agent wrote over Toolbox's REST
API.

| Measure | Value | Source |
|---|---|---|
| C1, the effect on every origin's `main` | pass | [toolbox-1.check.json](./toolbox-1.check.json) |
| Lines to maintain, as counted | 1,604 | [toolbox-1.lines.json](./toolbox-1.lines.json) |
| Struck as runtime data (six run records under `org/.state/`) | 780 | 3 × 175 + 3 × 85 |
| Lines to maintain | 824 | |
| Tool calls | 106 (84 shell, 19 edit, 3 read) | [toolbox-1.count.json](./toolbox-1.count.json) |
| Wall time | 886 s | [toolbox-1.count.json](./toolbox-1.count.json) |

The agent's own account is [toolbox-1.HANDOFF.md](./toolbox-1.HANDOFF.md).

## Friction, and what closed it

Every item is general: a portal user doing any governed change would meet it, so closing it is
not shaped to this task (P5). The benchmark's own action is never added to Toolbox; the
reference example for a landing change is `add-security-policy` in
[examples/federated-actions](../../../../examples/federated-actions/repo/.toolbox/toolbox.yaml).

| # | Friction (lines it cost) | Cause | Closed by |
|---|---|---|---|
| 1 | A CLI over the REST API to approve, follow and read runs (`org/bin/acme-actions`, 425) | `toolbox run` worked only on a checkout; a served portal had no client | `toolbox run --url`, with `--approve`, `--reject`, `--runs` and `--wait` (`fc820a1`) |
| 2 | A custom executor that pushes to `main` (`mark_deprecated.py`, 128) | A `change` could only propose a branch | `land: push`: the approved commit is pushed to the base, never forced (`295e78e`) |
| 3 | A connector that reads origin's `main` (`origin_state.py`, 23, plus its manifest entries) | Only a postcondition over connector facts could mark a run done | A change is verified by its base: the commit and every changed file as written (`295e78e`) |
| 4 | Run records copied to disk by the CLI | Served runs lived in memory and were lost on restart | Runs written through to Postgres or the state directory and reloaded (`ceeb0de`) |
| 5 | A CEL precondition standing in for a JSON Schema `pattern` | Input validation covered a subset of JSON Schema | The whole schema is validated (`e0fcfb6`, review F12) |
| 6 | The executor asks the org server whether the dispatching run is approved | Dispatch flags could be typed by hand in a checkout | Approvals are signed by the parent and checked against the pinned key (`f9e158d`) |

## What round 1 tests

Round 1 runs every arm three times at a frozen commit that includes these fixes, with prompts
committed first (P1). The bar is the README's §6, unchanged.
