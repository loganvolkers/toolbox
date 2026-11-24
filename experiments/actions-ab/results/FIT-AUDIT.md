# actions-ab fit audit (P4, P5)

*Independent fit audit under [dx-benchmarks.md](../../../docs/design/dx-benchmarks.md) §4.1. The
auditor did not design the benchmark, build an arm or change the product. Judged from the git
history and files from the pre-registration `816ac61` to `ce8210f` (round 7 in progress), and
read-only checks of the round 6 and 7 Toolbox workspaces. Round freezes: r1 `0ada74c` (r0 dry run
at `cf61bb0`), r2 `ad36c06`, r3 `428ef81`, r4 `4825782`, r5 `8900f7b`, r6 `f172337` (product of
`8aa85df`), r7 `554e83f`.*

## 1. Verdicts

**P5 (no harness-shaped product): holds with corrections.** All 56 product commits name a use
outside the harness. Every change to product *behaviour* is general. Most came from the
independent self-service review (F1–F24) or are defaults any org portal needs: a client for a
served portal, durable runs, a federation listed by git URL, idempotent changes, action test
cases, a readiness fix, exit codes. None of these encodes a harness file, port or name. The
problems are in *product documentation and fixtures*. From round 5 on, the `add-action` skill
(and the `test` skill and `toolbox --help` text it pulled along) was rewritten from this
benchmark's transcripts until it reproduced the harness:

- the workspace layout (`children: [{git: ../origins/alpha.git}]`, checkouts beside `org/`);
- a fleet loop over exactly `alpha beta gamma`;
- the task's input name (`successor`);
- a "Hand it off" step and "cite it in your handoff". The round 3 write-up itself had said that
  shaping Toolbox to write the handoff "would break P5";
- directives aimed at the tool-call measure: "Steps 1 to 3 are one shell command, writing every
  file with heredocs at once", "you need no other skill, example or `--help`", "one real run is
  enough".

Separately, the serverless approval path (D01M86WAVQQXYF9DFHCP3G035MA) gives only the
benchmark's call gap as its reason. Its skill also makes `--as`-on-trust approvals the default
path without saying so. Unit-test fixtures use `mark-deprecated`, `DEPRECATED.md` and
`successor`, and the design doc uses `../origins/`. These are cosmetic, because the product
doesn't depend on them.

No commit's *only* plausible use is this benchmark, but 15 need a correction. **Discount:** round
7's Toolbox runs copied the harness-shaped parts verbatim. All three used `../origins/<repo>.git`,
the `for r in alpha beta gamma` loop and the test case "ann cannot approve her own run, and carol
is not an approver", and cited it in the handoff. So round 7's Toolbox call counts are not
evidence about Toolbox in general, and neither, to a lesser degree, are round 6's. On interim
numbers, round 7 is the first round in which the bar against vanilla could be met (calls 24 vs
20, W 5, inconclusive rather than lost; lines 111 vs 617). That pass should not be reported as
the benchmark passing until a round runs on corrected docs.

**P4 (symmetric arms): fails.** The probes and both measures treat the arms alike. `score.py`,
`count.py` and the adjudicator prompt are unchanged since pre-registration. Every arm gets the
same probes, ports and tmux prefix, and no adjudicator needed a trivial fix in any arm. The
arms' *help* diverged, though, and rival friction was never triaged:

1. **A recipe for one arm only.** The rival scaffolds and prompts never changed after
   `816ac61`, except the tmux prefix that applied to every arm (`070d7fc`). Toolbox's
   `toolbox-docs/` changed every round. It gained a task-shaped recipe in round 4 (`2f03a5a`),
   which by round 6–7 (`1d83ab2`, `d2fd0b6`) matched the task's shape, its probes and its
   handoff. GitHub Actions, Backstage and vanilla got no recipe at any level. P4 requires "a
   recipe for all, or for none".
2. **Rival friction was never triaged, and could not have been fixed in time.** Each round
   write-up's "why, and what changed" table is drawn from the Toolbox transcripts only. Every
   round was frozen before the previous round's Backstage adjudication was published, and
   rounds 2, 4, 6 and 7 before its GitHub Actions adjudication. Round 6's backstage-3 was
   launched after round 7 was already frozen and running. Under P4, "deferring it means the
   round is not run".

The pre-registered framing holds: Toolbox's skills count as product, GitHub's and Backstage's
docs and conventions come from training, and a vanilla app relies on its language ecosystem
(README §7, limits 4 and 6). That would be a fair comparison for docs written for users at
large. It isn't one for a skill fitted over five revisions to this task's transcripts: no
rival's docs are fitted to this task. **Discount:** the losses to vanilla in rounds 1–6 stand,
since the asymmetry favours Toolbox. The lines wins over GitHub Actions and Backstage (5–15×
beyond W) are unlikely to be erased by a rival recipe. The calls wins over GitHub Actions and
Backstage in rounds 4–7, and any bar pass in round 7, should be labelled as obtained with a
Toolbox-only, task-fitted recipe and not cited as the benchmark's result.

**Count:** 56 product commits. 41 fit, 15 fit with correction, 0 don't fit. Of the 15, 7 are
cosmetic (tests or a design-doc example). The other 8 affect what the Toolbox arm reads or how
approvals are trusted.

## 2. Product commits (P5)

"Round" is the first round whose frozen commit contains the commit; "after r7" means no round
yet. Bookkeeping commits are listed under the table.

| Commit | Round | What it changed | Use outside the harness it names | Fits | Evidence |
|---|---|---|---|---|---|
| `e307c52` | r1 | Two children with one name offer nothing and name each other | Spoofing defence, review F3 | yes | `docs/self-service-review.md` F3 |
| `e0fcfb6` | r1 | Inputs validated against full JSON Schema | Review F12 (CODEOWNERS injection via `pattern`) | yes | Commit title; test uses `@acme/web` |
| `f9e158d` | r1 | Parent owns runners; signed approvals; cleared env | Review F1–F6, F11, F13, F16, F24 | yes | Body lists findings; fixtures use the product's pre-existing names (ann, bob, gamma in `m5-acceptance.sh` at `816ac61`) |
| `a93e11e` | r1 | Atomic approvals and cancel, human-only approvals, observed effects | Review F7–F10, F14, F15, F18 | yes | Commit body |
| `ea07ae5` | r1 | Test: only a postcondition reads outputs | Expression safety | yes | Test only |
| `20c5a6a` | r1 | Nonce in confirm tokens; runner allowlist tests | Review F1, F2; token collision | yes | Commit body |
| `f95319c` | r1 | M5 acceptance in CI with races | Review F7 | yes | `.github/workflows/ci.yml` |
| `c4d1752` | r1 | Self-service review, D01M86BY3YQX3SA1W99QXF1QA1H, docs | Unpauses M5 (D64) | yes | Decision file |
| `295e78e` | r1 | `land: push` for `change`; change verified on its base | Approved direct commits where the approval rule replaces review (`add-security-policy` in `examples/federated-actions/repo`) | yes with correction | Unit test in `toolbox-exec/src/executor.rs` uses action `mark-deprecated`, file `DEPRECATED.md`, input `REASON`: the benchmark's action as a fixture. Cosmetic |
| `fc820a1` | r1 | `toolbox run --url` client | Any served portal's CLI users | yes | r0 friction 1; general client |
| `ceeb0de` | r1 | Served runs survive restart | Durability | yes | Postgres migration |
| `88f1d74` | r1 | M5 acceptance: push, `--url`, restart | CI | yes | Uses `origin-alpha.git`, not the harness layout |
| `c9efc27` | r1 | Docs for the above | — | yes | Design doc, skills |
| `d379da6` | r1 | Refactor `toolbox run` | — | yes | Also edits `results/r0/README.md` (mixed commit, not a P5 issue) |
| `0ada74c` | r1 | `toolbox-authoring`: a change needs no postcondition test | Any `change` action | yes | 5-line skill edit |
| `2d69a48` | r2 | Scale example uses a command executor | Unrelated | yes | |
| `cadf8cc` | r2 | Site CLI fingerprint | Generated | yes | |
| `cd61305` | r2 | `federation.children` by git URL; kept clones and key | Org portal over GitHub repos (`https://github.com/acme/*.git` in the example) | yes with correction | Unit tests in `toolbox-model/src/federation.rs` use `../origins/alpha.git`, the harness layout. Cosmetic |
| `4177626` | r2 | M5 acceptance: git children | CI | yes | |
| `a99589b` | r2 | `--url --yes` stops at a blocked preview | Bug fix | yes | |
| `0866595` | r2 | `examples/federated-actions` start-here README, `git-org`, `minimal-repo` | Shipped example, run by `m5-acceptance.sh` step 9 | yes | Generic `add-code-of-conduct`; the closeness to the task is pre-registered (limit 4). Uses port 7100 (see §4.6) |
| `f85f503` | r2 | D01M86J769VGE6K5FJJQ76EP4AA; skills; design doc | General defaults (points 1–8) | yes with correction | `federated-actions.md` §5.2 example `{git: ../origins/beta.git, …}` is the harness layout. Cosmetic |
| `c19f747` | r2 | `additionalProperties: false` in the form subset | JSON Schema habit; forms never send hidden fields | yes | Decision point 8 |
| `c8b7405` | r2 | `verification.files`, drift check, `Approved-By`, commit as Toolbox | Audit and integrity of any landed change | yes with correction | Executor tests use `DEPRECATED.md`. Cosmetic. Also bundles `results/r2dry/` files |
| `a56cfd2` | r3 | `toolbox run` reads `TOOLBOX_URL`, `TOOLBOX_AS` | Parity with the read commands | yes | |
| `83081db` | r3 | A base that can't be fetched fails the change | Bug fix (found by an adjudicator) | yes | |
| `d88e2f7` | r3 | Exit 4 for unverified | Scripts that check exit codes | yes | |
| `b681579` | r3 | `serve --detach` / `--stop` | Any background server start | yes | |
| `da71c5d` | r3 | Field table in the example README; `--detach` in skills | Reference docs | yes | |
| `29ca002` | r3 | `--detach` prints the run-records dir | — | yes | |
| `5a84aff` | r3 | Test cleanup | — | yes | |
| `64ec5c8` | r3 | Approvers as plain identities in records | Readable audit records | yes | |
| `66edec9` | r3 | Placeholders in commit messages; validate refuses unknown ones | Any `change` message | yes with correction | Fixtures `"Deprecate {entity}: {inputs.reason}"` and `"Deprecate: use {successor}"` in `tests/cases.json`. Cosmetic |
| `18309d0` | r3 | Ready once children are read | Any federating portal | yes | |
| `428ef81` | r3 | Hub refactor; doc | — | yes | |
| `f1f79f2` | r4 | A change its base already has succeeds without a commit | Idempotent re-requests | yes with correction | Test uses `DEPRECATED.md`. Cosmetic |
| `0219541` | r4 | `toolbox test` action cases, incl. `rejectPush` | Testing an action before anyone runs it (§6.5); branch protection | yes | Product's pre-existing names |
| `e24b19d` | r4 | Action cases run on the working tree | Test before first commit | yes | |
| `2f03a5a` | r4 | `add-action` skill, first version | Planned in `agent-plugins.md` at `816ac61` | yes | Generic URLs (`https://github.com/acme/alpha.git`) |
| `674dd4c` | r5 | Site CLI fingerprint | Generated | yes | |
| `3c81d57` | r5 | Test-file `defaults` | Shared fields in any test file | yes | |
| `72d0fa0` | r5 | Hint for a value cut at a comma | General YAML pitfall | yes | |
| `20ef2e4` | r5 | `add-action`: preconditions, record fields, "cite it in your handoff" | Preconditions and record fields: general | yes with correction | "Handoff" is the benchmark's `HANDOFF.md`; r3 README said shaping Toolbox to it breaks P5 |
| `8a11dff` | r6 | Decision §16–17 | — | yes | |
| `8615169` | r6 | A child's `federation` may come from an included file | Bug fix | yes | |
| `673303e` | r6 | `{portal}` in test cases; standard included action file | Fleet standards (stories S3, S10) | yes with correction | Unit test in `action_tests.rs` uses `successor: 'acme/{portal}'` and "is not its own successor". Cosmetic |
| `c8bcb21` | r6 | Refactor | — | yes | |
| `1d83ab2` | r6 | `add-action` rewritten as the "whole path"; `test` skill example | Fleet action rollout | yes with correction | Shipped to r6 and r7 (workspace docs match `f172337` and `554e83f`). Contains `../origins/alpha.git`, a "6. Hand it off" step matching the task's handoff spec, "cite it in your handoff", and "Each step is one shell command: write every file with heredocs at once". Test cases mirror probes P2, P4 and P6. The `test` skill gained `inputs: {successor: 'acme/{portal}'}` |
| `8aa85df` | r6 | Decision §18 | Stories S3, S10 | yes with correction | States the skill now covers "from layout to handoff" |
| `9b78410` | r7 | A parent's checkout approves and signs, no server | Not named: the commit and its decision cite only rounds 5–6 call counts. Plausible: one operator, offline or trial use | yes with correction | Behaviour is general and covered by `m5-acceptance.sh` ("fine offline"), but P5 needs the use named |
| `53782e3` | r7 | D01M86WAVQQXYF9DFHCP3G035MA; skills and example default to serverless | As above | yes with correction | Decision: "the vanilla arm has no server to keep". `add-action` makes `--as`-on-trust approval the default path without the trust caveat that T6 states |
| `d2fd0b6` | r7 | Fail-fast loop in `add-action`; `run` skill points to it alone | Rolling out one action to a fleet | yes with correction | `for r in alpha beta gamma; do [ $r = alpha ] \|\| cp -r alpha/.toolbox/…`: the harness's repository names and sibling-checkout layout |
| `d20a43a` | r7 | `run` and `test` help describe parent approvals and action cases | Describes shipped features | yes | Present in the r7 binary (checked) |
| `11737f7` | after r7 | Refused approvals recorded; blocked run exits 2 in a checkout | Audit completeness (T6) | yes | |
| `d9b3cc7` | after r7 | `add-action` reordered: key first | Fewer edits for any author | yes with correction | Keeps `../origins/alpha.git` and "name it in your handoff". Adds `sed -i … alpha/.toolbox/actions/…` and "Steps 1 to 3 are one shell command, writing every file with heredocs at once" |
| `b38df30` | after r7 | `toolbox --help` names `add-action` "alone"; `actions` skill points there | Skill discoverability | yes with correction | The after-help singles out one workflow in the task's terms ("a self-service action that repositories own and offer from one org portal … alone: it is the whole path"). Its aim is to cut orientation calls |

**Not product (bookkeeping, not counted):** `ff96b40` (benchmark catalog entry), `da80983`
(toolbox-expr reassessment), `82bd1e6`, `15500a7`, `bc56360`, `bca9061`, `86718a1`, `58ebab6`,
`4a36dda`, `005978e` (`portal/results.json` baselines), `dc47bf4`, `13a0de3`, `c8cd393`,
`8900f7b` (research log), `0296f33` (round 5 write-up plus a baseline).

## 3. Arms (P4)

| Arm | Scaffold and prompt provide | Friction fixed between rounds | Symmetric | Evidence |
|---|---|---|---|---|
| `toolbox` | `bin/` (binaries at the freeze); `toolbox-docs/` (operator skills, `examples/federated-actions`, `examples/repo-portal-starter`, the manifest schema). The prompt points to them and to `--help` | 56 product commits over r0→r7 (41 after round 1's freeze), from Toolbox transcripts, the self-service review and adjudications, including five revisions of the `add-action` recipe fitted to this task (`2f03a5a`, `20ef2e4`, `1d83ab2`, `d2fd0b6`, `d9b3cc7`) | no | Round write-ups r0–r6, "why, and what changed". r6 and r7 workspaces' `toolbox-docs` equal `git archive` of `f172337` and `554e83f`. r7 runs copied the skill's layout, loop and probe-named test case |
| `gha` | `bin/act`. The prompt gives host mode (`-P ubuntu-latest=-self-hosted`) and says GitHub features configured as code and explained count (credited) | None. No GitHub Actions friction was triaged in any write-up. Adjudications of r1, r3, r5 and r6 landed after the next freeze | no (vs toolbox) | `git log 816ac61..HEAD -- prompts/platform-gha.md` is empty. Timeline below. No probe was ever credited (0 `"credited"` verdicts) |
| `backstage` | Stock `@backstage/create-app` 1.57 app (scaffolder backend 4.3, GitHub module, `examples/template`) and `node_modules`; an overlay with the port and static tokens for ann, bob and carol. The prompt says how to start it and to write custom actions or modules as glue | None. Every round froze before the previous round's Backstage adjudication. Rounds 3, 6 and 7 froze before every Backstage run of the previous round was even measured. r6 backstage-3 launched after r7 was frozen | no (vs toolbox) | `harness/backstage/` changed only by `86be348` (yamark format). Timeline below |
| `vanilla` | Nothing. The prompt names Python 3.12, Go 1.22, Node 24 | None (none recorded) | no (vs toolbox) | `platform-vanilla.md` unchanged since `816ac61` |
| all | Same task, origins, checkouts, ports, tmux prefix (`070d7fc`, all arms); same probes, `score.py` and `count.py` (unchanged since `816ac61`); same adjudicator prompt | — | yes | No trivial fix recorded in any arm's adjudication. Backstage's tokens are the minimum its API needs, and every arm's task gives the same three identities by name |

**Freeze timeline** (commit times, 2026-11-24 UTC):

| Next round frozen | Previous round's Toolbox fixes landed | Previous round's GHA adjudication | Previous round's Backstage adjudication |
|---|---|---|---|
| r2 `ad36c06` 11:10 | 10:36–11:09 (`cd61305`, `f85f503`, `c8b7405`) | 11:13 (after) | 11:14 (after) |
| r3 `428ef81` 12:01 | 11:17–12:00 | 11:45 (before) | 12:18 (after) |
| r4 `4825782` 12:34 | 12:18–12:32 | 12:40 (after) | 12:40 (after) |
| r5 `8900f7b` 12:57 | 12:55 | 12:55 (before) | 13:04 (after) |
| r6 `f172337` 13:20 | 13:14–13:18 | 13:23 (after) | 13:30 (after) |
| r7 `554e83f` 13:48 | 13:40–13:48 | 14:12 (after) | 14:30 (after) |

Times are when the first adjudication record of that arm was committed. The fixes after rounds 3,
5, 6 and 7 landed before that round's own Toolbox adjudication, from its transcripts.

**Is "Toolbox's skills are product" a fair comparison?** In principle, yes. Toolbox's docs ship
with Toolbox, as GitHub's and Backstage's docs and conventions come with theirs (and from
training, which weighs against Toolbox, limit 6), and a vanilla app has its language's standard
library. In this benchmark, no. The `add-action` skill was written during the benchmark and
revised against its transcripts until it encoded this task's shape: one file landed on `main`,
two maintainers and a non-maintainer, the four handoff probes as test cases, a three-repository
loop in the harness layout, and a handoff step. No rival's documentation was fitted to this task,
and no rival was offered an equivalent. A fair comparison of shipped docs needs either a task
the docs were not tuned on, or a recipe of the same level for each arm.

## 4. Corrections

1. **Make `add-action` generic** (`plugins/toolbox-operator/skills/add-action/SKILL.md`, from
   `1d83ab2`, `d2fd0b6`, `d9b3cc7`, `20ef2e4`):
   - Replace `../origins/alpha.git` with a real URL or `<path-or-url>`, and the
     `alpha beta gamma` loop with `for r in $REPOS` over `<repo>` checkouts.
   - Drop step 6 "Hand it off" and every "handoff" mention, or replace them with something an
     org would do, such as "tell requesters the commands in step 5".
   - Drop the tool-call directives ("Steps 1 to 3 are one shell command … heredocs", "one real
     run is enough"). "You need no other skill" may stay if it's true.
   - Vary the test cases away from a one-to-one copy of probes P2, P4 and P6, or keep them and
     say they are the general refusal set (requester, unlisted approver, protected branch).
2. **Make the `test` skill generic:** replace `inputs: {successor: 'acme/{portal}'}` with the
   skill's own `add-security-policy` input.
3. **`toolbox --help` (`b38df30`):** keep "Each command has an agent skill of the same name",
   and list the task skills (`add-action` among them) without "alone: it is the whole path".
4. **Serverless approvals (`9b78410`, `53782e3`):**
   - Add a decision that names the outside use (one operator, offline or trial use; CI or a
     bastion host that holds the key).
   - Have `add-action` and the example say that a checkout takes `--as` on trust (T6), and
     that a team approves through `toolbox serve` with sign-in.
5. **Rename the test fixtures** that use the benchmark's action, file and input (`295e78e`,
   `c8b7405`, `f1f79f2` in `toolbox-exec/src/executor.rs`; `66edec9` in
   `toolbox-model/tests/cases.json`; `673303e` in `toolbox-cli/src/action_tests.rs`). Also
   rename the `../origins/` paths in `toolbox-model/src/federation.rs` (`cd61305`) and
   `federated-actions.md` §5.2 (`f85f503`). Cosmetic: no round's result depends on them.
6. **Port 7100:** it is `round.sh`'s `PORT_BASE` default (in the pre-registration) and appears
   in product docs from `0866595`, `2f03a5a`, `da71c5d` and `29ca002`. No run is assigned
   7100 (runs get `PORT_BASE + 10i`), so it has no effect. Change it anyway, to make the
   independence obvious.
7. **Restore P4 before counting another round:**
   - Freeze round *n+1* only after every arm of round *n* is measured and adjudicated.
   - Triage each rival's transcripts for scaffold and prompt friction with the same table the
     Toolbox arm gets, and fix or explicitly reject each item before the freeze.
   - Either give every arm a recipe of the same level (GitHub Actions: `workflow_dispatch` with
     environment reviewers and a local CLI; Backstage: a custom scaffolder action, a permission
     policy and an approval step; vanilla: none, recorded as such), or run on a held-out task
     the Toolbox skills were not tuned on. Examples: an action that opens a pull request, edits
     an existing file, or has group approvers.
8. **Label the record:**
   - Report the round 4–7 calls wins over GitHub Actions and Backstage, and any round 7 bar
     pass, as "with a Toolbox-only task-fitted recipe".
   - Don't cite round 7 as the benchmark passing.
   - Record this audit's corrections in the catalog entry's `fitAudit` and rerun as round 8 on
     corrected docs.
