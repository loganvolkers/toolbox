# actions-gitlab round 1: the bar passes

*Counted. Three runs per arm, at the frozen commit `8c8ea2c`, on the prompts in
[prompts/rendered/r1](../../prompts/rendered/r1/), committed before the first launch. About the
box and the forge. Measured by the harness (`harness/measure.sh`); probed by two fresh
adjudicators who built no part of either arm (P7), each taking runs from both arms.*

## Result

**The benchmark passes (README §6).** Every run of both arms fully passes C1 and P1 to P11, so H1
holds. Toolbox's medians are lower by more than *W* on both primary measures, so H2 and H3 hold:

| Hypothesis | Toolbox | GitLab CE | *W* | Verdict |
|---|---|---|---|---|
| H1 delivery (fully passing runs) | 3 of 3 | 3 of 3 | | holds |
| H2 lines to maintain (median) | 247 | 1,076 | 642 | holds |
| H3 tool calls (median) | 38 | 78 | 19 | holds |

No probe needed a Premium credit: both GitLab arms enforced every rule in their own pipeline code
and committed the Premium settings as code that CE refuses. No adjudicator made a fix or struck a
line; every runtime file (run records, signing keys, clones, the controller's log) sat under
`home/`, which the line count already skips.

| Run | Lines | Calls (shell, edit, read) | Wall | GitLab settings changed | Glue the agent wrote |
|---|---|---|---|---|---|
| `toolbox-1` | 271 | 54 (41, 10, 3) | 476 s | 1 | `env.sh` (10) |
| `toolbox-2` | 247 | 38 (35, 3, 0) | 393 s | 3 | `env.sh` (11) |
| `toolbox-3` | 154 | 35 (34, 1, 0) | 317 s | 1 | none |
| `gitlab-1` | 1,542 | 82 (56, 24, 2) | 1,692 s | 16 | `bin/acme-actions` (903, with a controller that must keep running), `set_codeowners.py` (430) |
| `gitlab-2` | 1,076 | 76 (56, 20, 0) | 1,364 s | 14 | `bin/acme-actions` (318), `ci/runner.py` (274), `lib/acme_gitlab.py` (98) |
| `gitlab-3` | 900 | 78 (53, 25, 0) | 1,303 s | 19 | `org/actions` (354), `codeowners.py` (314) |

Spread (min, median, max): lines, Toolbox 154, 247, 271 and GitLab 900, 1,076, 1,542; calls,
Toolbox 35, 38, 54 and GitLab 76, 78, 82. Every run's measures and adjudication are in this
directory; [tally.md](./tally.md) is `harness/tally.py`'s output.

**The Toolbox arm** wrote, per project, an action with a JSON Schema for its inputs, its edit (an
`awk` line in `toolbox-3`, a Python script with unit tests in the others) and `toolbox test` cases,
and an org portal that lists the three projects, reads `group:<org>/release` from GitLab through
the `gitlab` connector and takes each person's identity from their own token
(`forges[].identity: true`). Nothing runs between commands.

**The GitLab arm** took three shapes: merge requests as runs with a controller that counts
approvals (`gitlab-1`), pipelines in the org project with a manual approve job (`gitlab-2`), and
pipelines on a protected `self-service` branch in each project (`gitlab-3`, because on CE only
someone who may merge into a branch can play its manual job). Each rechecks the release group
through the API, because CE has no approval rules, and each wrote its own command-line tool for
the one place to find and run the action.

## Labels (read before quoting the numbers)

1. **Toolbox's identity came from a fix this task's dry run asked for.** Round 0's Toolbox agent
   wrote a 65-line wrapper to bind `--as` to a GitLab token; `forges[].identity: true` replaced it
   before the freeze (T1 in [round 0](../r0/README.md)). The fix is general (any checkout approving
   on behalf of people needs it), but it was built after seeing this task. Without it, round 0's
   Toolbox run still had 361 lines against the rival's 1,182.
2. **One round, one task, one forge version.** CE 19.6.1 on loopback, no TLS or SSO (limit 8).
   The rival arm had no product to fix; its friction (below) is GitLab's.
3. **Agents know GitLab from training and Toolbox only from its docs** (limit 3). The Toolbox arm
   had the shipped `examples/federated-actions/gitlab-org` portal, which shows this shape (limit 2).
4. **Settings are not lines** (limit 5). The GitLab arm changed 14 to 19 settings through the API
   (protected branches, tokens, CI variables, job token scopes); Toolbox changed 1 to 3. Neither is
   in the primary measure.

## Friction and losses

Real runs drove these. Fixed items were fixed after the round, so they don't touch its scores.

| # | Arm | Friction | Status |
|---|---|---|---|
| T3 | toolbox | A loopback `toolbox serve` signs people in by header, so on an `identity: true` portal it would have let anyone approve as anyone. `toolbox-1` noticed and avoided the server | Fixed (`52982d1`): a dev server refuses to create, approve, reject or cancel runs there (`identity-from-forge`, 403); API test, [forges.md](../../../../docs/design/forges.md) §4.6 |
| T4 | toolbox | A waiting run's hint said `--as <approver>`, which these portals refuse (`toolbox-3`, both adjudicators) | Fixed (`54a1389`): the hint names `--identity-token-file`; acceptance step 9 checks it |
| T5 | toolbox | Listing runs shows a federated run twice, as the org's run and the project's (`toolbox-2`) | Open |
| T6 | toolbox | A new `HOME` mints a new signing key, and each project's action must then pin it by hand (`toolbox-1`, `toolbox-2`) | Open; the key's place is documented |
| T2 | toolbox | Approved changes push with the platform engineer's git credentials in `HOME`, not a bot's (all three) | Open, as in round 0 ([forges.md](../../../../docs/design/forges.md) §7) |
| T7 | toolbox | The signing key and run records are plain files: whoever can write there can forge an approval (`toolbox-3`) | Open: a checkout trusts whoever can write its state directory ([federated-actions.md](../../../../docs/design/federated-actions.md) T6) |
| R1 | gitlab | CE lets only someone who may merge into a branch play its manual job, so approvals can't live on Maintainer-only `main` | GitLab's; `gitlab-3` used a `self-service` branch |
| R2 | gitlab | The shell runner reuses its working directory: a stale preview leaked into a later run (`gitlab-3`, fixed in-run) | GitLab's runner default |
| R3 | gitlab | A pipeline whose approval was refused is failed and can't be cancelled, so the run stays approvable (`gitlab-2`, `gitlab-3`) | GitLab's |
| R4 | gitlab | CE records `glab mr approve` from anyone; the run's controller ignores them, but the merge request still shows them (`gitlab-1`) | GitLab's |
| R5 | gitlab | `gitlab-3` committed a Python bytecode cache to alpha's `main` | Agent's; binary, not counted |

Where GitLab is ahead: it knows who started a pipeline or played a job with no extra token, which
Toolbox needs `forges[].identity` and a token per person for.

## How to check

```sh
python3 experiments/actions-gitlab/harness/tally.py experiments/actions-gitlab/results/r1
bash experiments/actions-gitlab/harness/measure.sh 1 toolbox-1 <transcript-dir>   # from a fetched transcript
cargo test -p toolbox-api --test api a_dev_server_does_not_take_your_word   # T3
scripts/gitlab-acceptance.sh                                                  # T4, step 9
```
