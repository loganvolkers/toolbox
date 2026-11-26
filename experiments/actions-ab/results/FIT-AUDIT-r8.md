# actions-ab fit audit, round 8 (P4, P5)

*Independent fit audit under [dx-benchmarks.md](../../../docs/design/dx-benchmarks.md) §4.1. The
auditor did not design the benchmark, build an arm or change the product. Judged: every product
commit from `ce8210f` (the end of the [previous audit](FIT-AUDIT.md)) to round 8's freeze
`8519cd5`; the round 8 workspaces under `/tmp/actions-ab/r8/`, read-only; the whole
`seikika-docs/` tree and the `--help` text of every subcommand of the round's `seikika` binary;
the three Seikika transcripts, plus gha-1's for one P4 question; and `results/r8/` as of `fa1740f`.
At that commit the Seikika and vanilla arms are adjudicated, gha-1 and backstage-1 are measured,
gha-2 and backstage-2 are running, and gha-3 and backstage-3 have not been launched. gha-2's
measures were written, uncommitted, while this audit was being finished; they are included where
noted.*

## 1. Verdicts

**P5 (no harness-shaped product): holds with corrections.** Two product commits sit between
`ce8210f` and `8519cd5`: `7193247` and `fb5dc3f`. Both name a use outside the harness, and both
change behaviour in a general way.

- `7193247` removes the benchmark's names from fixtures, ports and `--help`.
- `fb5dc3f` makes the rest of the previous audit's corrections. It also adds group approvers that
  a parent portal's checkout can resolve, without a server.
- 14 of the previous audit's 15 commit corrections are made. The one left is wording in a
  decision file that the repository's rules make immutable (§3).

`fb5dc3f` needs one correction of its own. It writes round 8's group into the shipped example the
Seikika arm reads: `group:release`, with `user:dave` as its member, appears in two rows of
`examples/federated-actions/README.md`. The same names are in its unit tests and acceptance step,
and its decision cites "someone in release approves". `group:release` appears nowhere in the
product before this commit. dave and erin were already test personas in `crates/seikika-api/tests/`
before the pre-registration. Under P5 a change names no harness convention, and a held-out task's
group is one.

**Held out: partly.** Two parts of the task were held out, and one was not.

- **Held out: the effect and the inputs.** The effect is a notice line at the top of a README in
  any letter case, replacing an earlier notice. The inputs are a `level` enum and a one-line
  `message` of 10 to 160 characters. No doc, skill, example or help text covers either one. Each
  Seikika run designed, wrote and tested its own README script, and that script is most of its
  186–242 kept lines.
- **Not held out: R9, the group defined once at the org level.** README §8 limit 8 says the task
  was chosen after round 7 and that `fb5dc3f` was built before the freeze "because the fit audit's
  held-out-task correction called for a requirement the old task lacked".
  - The previous audit had offered "group approvers" as one example of a held-out task, and the
    product was then extended to handle them before the run.
  - Before `fb5dc3f`, a checkout could not meet a group approver at all. `run.rs` at `ce8210f`
    says: "Without an authorization model nobody is known to be in a group, so only a named user
    approves here".
  - The docs `fb5dc3f` added walk an agent through R9's steps: an `authz` block in the org portal,
    a `groups.fga` model, and `members` in test cases. The skill's sentence "define each group once
    there rather than in every repository" restates R9 ("defined in one place at the org level, not
    in each repository").
- **Already tuned: R1–R8.** These are the requirements the `add-action` skill was tuned on over
  rounds 4–7. The renaming removed the task's words but kept that shape.

All three Seikika runs copied the example's `authz` row (`{user: user:dave, relation: member,
object: group:release}`) and its `members: {'group:release': …}` form. seikika-2 summed up its
reading as "Seikika natively supports this via federated change actions, approval rules, and
org-portal groups with test cases."

**P4 (symmetric arms): holds with corrections.** Most of what failed last time is fixed:

- every fix in `results/triage/` is applied to the arm it concerns (11 of 11, §5.2);
- round 7 was fully adjudicated before round 8 was frozen;
- each arm has its platform's own docs, pinned and uncurated;
- the rendered prompts differ only in paths, ports and the "Your platform" paragraph;
- all 9 measured runs got exactly their committed prompt, and the isolation check is clean for
  all 9;
- the probes, `score.py`, `count.py`, `tally.py` and the adjudicator prompt are the same for every
  arm, and strikes follow one rule.

Three gaps remain:

1. **The Seikika arm still has a recipe the rivals lack.** `add-action` "covers the whole path",
   and its R9 section was written for this round. P4's "a recipe for all, or for none" holds for
   the held-out effect, but not for R9.
2. **The gha docs have no GitHub teams pages.** They were chosen for the old task. They have no
   GitHub team pages and no Teams REST reference, although round 8's R9 and the gha prompt name
   teams. gha-1 wrote its team-membership API calls from memory. Backstage's whole docs tree covers
   `kind: Group`.
3. **Three rival runs have no prompt check yet.** gha-3, backstage-2 and backstage-3 have no
   `prompt.json`, and two of them have not been launched.

**Discount for round 8's numbers.** Interim figures at `fa1740f`:

| Arm | Calls (runs; median, range) | Lines after strikes (runs; median, range) | Fully passes |
|---|---|---|---|
| Seikika | 42, 23, 36; 36, 19 | 241, 186, 242; 241, 56 | 3 of 3 |
| Vanilla | 30, 20, 16; 20, 14 | 1160, 952, 536; 952, 624 | 3 of 3 |
| gha | 52, 47 (gha-1, gha-2) | 1774, 1908 raw, not yet adjudicated | — |
| Backstage | 81 (backstage-1 only) | 1855 raw, not yet adjudicated | — |

- **Delivery (H1):** no discount for R1–R8 or the held-out effect. Seikika's P9 passes show that a
  feature built for this round works. They are not evidence that Seikika handled a requirement no
  one anticipated. Report P9 for Seikika with that label.
- **Lines (H2): little discount.** The R9 help shapes only Seikika's `authz` block (7–14 lines) and
  a 6-line model, and the task names the people anyway. Most of Seikika's kept lines are the held-out README script.
  Against vanilla, H2 holds on interim numbers by 711 against W 624. That margin is not wide, but
  the help can't account for it.
- **Calls (H3): label any result.** Read Seikika's calls as "with a shipped task skill tuned on
  rounds 4–7 and extended for R9 before this round". The rivals had untuned docs, and gha had no
  team docs. Against vanilla, calls are inconclusive with vanilla ahead (20 vs 36, W 19), so the
  asymmetry produced no win there. Any calls win over gha or Backstage should carry that label and
  should not be cited as a held-out result.

## 2. Product commits (P5)

The two product commits in range are listed here. The other commits from `ce8210f` to `8519cd5`
are bookkeeping: `55650f3` (the previous audit), `2f2aa75` and `5ff668b` (round 7 measures,
adjudication and the rival triage), and `8519cd5` (README, harness and prompts only; it changes no
product file). From the freeze to `fa1740f`, only `experiments/` and `portal/results.json` changed
(`git diff --stat 8519cd5 HEAD -- . ':!experiments'`).

| Commit | What it changed | Use outside the harness | Fits | Evidence |
|---|---|---|---|---|
| `7193247` | Renamed test fixtures: `executor.rs` from `mark-deprecated`, `DEPRECATED.md` and `REASON` to `publish-support`, `SUPPORT.md` and `CHANNEL`; `cases.json` from `{successor}` to `{contact}`; `federation.rs` from `../origins/` to `../repos/`. Moved port 7100 to 8443 in a `detach.rs` test and to 8080 in the `serve` skill. Changed `seikika --help` to list the task skills (`add-action`, `add-scorecard`, `author-connector`) without "alone: it is the whole path". The `run` and `actions` skills now point to `add-action` as one skill among others | Takes harness names out of the product. The help text names the task skills that exist, for any user | yes | The diff. `git grep` at `8519cd5` finds none of the old names or 7100 in `crates/`, `plugins/`, `examples/` or `schema/`. The round's binary (sha256 `eced049c…`, the same in all three workspaces) prints the new help text |
| `fb5dc3f` | Five changes. (1) `seikika_authz::local`: evaluates the portal's OpenFGA model over its derived tuples. It handles direct types, nested groups, wildcards, `or`, computed relations and `from`, and refuses `and` and `but not`. `seikika run --approve/--reject` in a parent's checkout and `seikika test` cases now use it. (2) A `members:` field in test cases. (3) Decision D01M8727W7FEED6VVRY6MB3JFR3, which names the outside use of serverless approvals and its trust caveat. (4) `add-action` made generic: `web` and `api` URLs, a `$REPOS` loop, step 6 "Tell people how to use it", no tool-call directives, the refusal set explained, the `--as` trust caveat, and a groups section. The `test` skill's input renamed. The example README gains `authz` and `members` rows and the trust caveat. T6 in `federated-actions.md` updated. (5) `m5-acceptance.sh` step 11 | Group approvals ("someone in security approves") for an org portal run from one machine, a trial, or a CI job that guards the state directory (D01M8727W7FEED6VVRY6MB3JFR3, point 1). It uses the same model a served portal asks OpenFGA about. It rejects listing members under `federation`, which would be a second way to declare membership | **yes with correction** | **The behaviour is general.** Served portals had group approvers before the benchmark (`352e625`, an ancestor of `816ac61`), and the evaluator refuses what it can't decide exactly. **The names are this task's:** the example README's `authz` row (`user:dave` in `group:release`) and its `tests` row (`members: {'group:release': [user:dave]}`) are what the Seikika arm read. `local.rs` tests use `group:release`, `group:leads`, dave, erin and carol; `action_tests.rs` uses `group:release: [user:dave]`; `m5-acceptance.sh` step 11 uses dave, erin, carol and `group:release`; the decision cites "someone in release approves". The commit came after round 8's task was chosen (README §8, limit 8) |

**Count:** 2 product commits. 1 fits, 1 fits with correction, 0 don't fit.

## 3. The previous audit's corrections

The previous audit marked 15 commits "yes with correction" and grouped the fixes into 8 items
(its §4). Each row below is checked against the tree at `8519cd5`.

| # | Commit (correction item) | Asked | Made | Evidence |
|---|---|---|---|---|
| 1 | `295e78e` (5) | Rename the executor fixture's `mark-deprecated`, `DEPRECATED.md` and `REASON` | yes | `7193247`: `publish-support`, `SUPPORT.md`, `CHANNEL` |
| 2 | `cd61305` (5) | Rename `../origins/` in `federation.rs` | yes | `7193247`: `../repos/alpha.git` |
| 3 | `f85f503` (5) | Rename `../origins/beta.git` in `federated-actions.md` §5.2 | yes | `fb5dc3f`: `../repos/beta.git` |
| 4 | `c8b7405` (5) | Rename `DEPRECATED.md` in the executor tests | yes | `7193247` |
| 5 | `66edec9` (5) | Rename `"Deprecate: use {successor}"` in `cases.json` | yes | `7193247`: `"Publish: {contact}"` |
| 6 | `f1f79f2` (5) | Rename `DEPRECATED.md` in the idempotence test | yes | `7193247` (same file) |
| 7 | `20ef2e4` (1) | Drop "cite it in your handoff" | yes | `fb5dc3f`. Step 3 now says the last two cases are "the refusals every approved action should have", which is the second option in correction 1 |
| 8 | `673303e` (5) | Rename `successor: 'acme/{portal}'` in `action_tests.rs` | yes | `fb5dc3f`: `contact: '{portal}@acme.dev'` |
| 9 | `1d83ab2` (1, 2) | In `add-action`, remove `../origins/alpha.git`, step 6 "Hand it off", the heredoc directive and the one-to-one copies of the probes. In the `test` skill, rename `successor` | yes | `fb5dc3f`. The URLs are now `github.com/acme/web.git` and `git@github.com:acme/api.git`. Step 6 is "Tell people how to use it". The directive is gone. The cases are renamed ("neither the requester nor someone unlisted", with `user:dana`) and explained as the general refusal set. The `test` skill uses `contact` |
| 10 | `8aa85df` (1) | The decision says the skill covers "from layout to handoff" | **no** | D01M86J769VGE6K5FJJQ76EP4AA §18 still says so. Decision files can't be edited (AGENTS.md), and no later decision records that the skill no longer covers the handoff. Cosmetic: decisions are not in the Seikika arm's docs |
| 11 | `9b78410` (4) | Name the outside use of serverless approvals | yes | D01M8727W7FEED6VVRY6MB3JFR3, point 1: one operator, a trial, a CI job or bastion host. It says a team approves through `seikika serve` |
| 12 | `53782e3` (4) | Say in `add-action` and the example that a checkout takes `--as` on trust | yes | `add-action` "Who may say `--as`"; example README step 4; T6 in `federated-actions.md` |
| 13 | `d2fd0b6` (1) | Replace the `alpha beta gamma` loop | yes | `REPOS="web api docs"; first=${REPOS%% *}` |
| 14 | `d9b3cc7` (1) | Drop the `sed … alpha/…` line, "Steps 1 to 3 are one shell command…" and "name it in your handoff" | yes | All three are gone. The skill now says "Replace `ed25519:<key>` with the key from step 1" |
| 15 | `b38df30` (3) | Remove "alone: it is the whole path" from `--help` | yes | `7193247`. Checked in the round's binary |

The corrections not tied to one commit:

| Item | Asked | Made | Evidence |
|---|---|---|---|
| 6 | Change port 7100 in product docs | yes | `7193247` and `fb5dc3f` (example README: 8080). No 7100 at `8519cd5` |
| 7a | Freeze round *n+1* only after round *n* is measured and adjudicated | yes | `5ff668b` ("all twelve runs adjudicated") is the parent of `8519cd5` |
| 7b | Triage each rival's transcripts with the same table, and fix or reject each item | yes, with one gap | `results/triage/{gha,backstage,vanilla}.md`. The Backstage triage covers five of six runs: r7 backstage-3 had no finished transcript, and r7's Backstage adjudications came later. They record no trivial fixes and all three runs fully pass, so the gap is small |
| 7c | Give every arm a recipe of the same level, or run a held-out task the skills weren't tuned on | **partly** | The task is new and its effect is held out, but R9 is not (§1 and §4) |
| 8a | Label rounds 4–7 calls wins and the round 7 pass | yes | `results/r7/README.md`: "It is not a pass", with the label "with a Seikika-only task-fitted recipe" |
| 8b | Record the corrections in the catalog entry's `fitAudit` | **no** | `portal/benchmarks/actions-ab.yaml` at `8519cd5` and at `fa1740f`: `fitAudit: ''`, `status: designed`, `frozenAt: ''`, and `measure` still says "probes P1 to P10" |

## 4. Held-out check

I searched the whole `seikika-docs/` tree (identical in all three workspaces to `git archive
8519cd5` of `plugins/seikika-operator/skills`, `examples/federated-actions`,
`examples/repo-portal-starter` and `schema/manifest.schema.json`) and the `--help` text of
`seikika` and every subcommand (530 lines). The search terms were: notice, README, Info, Warning,
level, message, 160, add-notice, dave, erin, carol, release, `group:`, members, handoff, origins,
alpha, gamma, successor, deprecat, and the round's ports.

**No hits** for the task's effect, file, inputs or handoff. Nothing mentions notices, prepending
to or replacing part of a README, the `level` or `message` inputs, 10–160 characters, `add-notice`,
`HANDOFF.md` or a handoff, `../origins/`, or ports 7110–7221. `--help` has no task text at all.
The `write-readme` action in `examples/federated-actions/repo` replaces a whole README; it predates
the pre-registration (`d7b6abf`). Uses of alpha, beta, gamma, ann, bob and carol in the examples
and skills also predate it (`scripts/m5-acceptance.sh` and the `run` skill at `816ac61`).

**Hits, all for R9:**

| Where (in `seikika-docs/`) | Text | Task element | Walks an agent through the task's steps? |
|---|---|---|---|
| `examples/federated-actions/README.md:80` | `"['group:release']"` needs a member | R4's group | No, but it names the task's group |
| `examples/federated-actions/README.md:81` | `authz`: `{model: groups.fga, tuples: [{static: [{user: user:dave, relation: member, object: group:release}]}]}` "defines the groups approvals name, once for every child" | R9's group, an approver and R9's mechanism | **Yes, for R9** |
| `examples/federated-actions/README.md:89` | `members: {'group:release': [user:dave]}` "stands in for the org's groups" | Testing R9 | **Yes, for R9** |
| `skills/add-action/SKILL.md` §2 | "define each group once there rather than in every repository", then the `authz` block and `groups.fga` (`group:security`, `group:sre`) | R9, almost in its words | **Yes, for R9**, with generic names |
| `skills/add-action/SKILL.md` §3, `skills/test/SKILL.md` | `members: {'group:security': […]}` | Testing R9 | Generic names, R9's step |

**What the runs did with it.** All three read `add-action`, the example README and `seikika run
--help` in their first five calls. They spent 5–7 calls on docs (seikika-1 calls 2–8, seikika-2
calls 2–5, 7 and 8, seikika-3 calls 2–5 and 8). Each org portal they committed has the example's
`authz` row and `groups.fga` model, adding `user:erin`. Each test file has the `members` form. The
names come from the task too, so copying them saved nothing. The mechanism is the R9 path written
for this round.

No run copied anything for the effect. seikika-1 wrote `add-notice.sh` and a script test,
seikika-2 wrote `add-notice.py` with a self-test, and seikika-3 wrote `add-notice.sh` with fixture
READMEs. Each wrote 11–14 action cases for the inputs, the people and a rejected push.

**Why R9 still counts as fitted, though the feature is general.** The product change is legitimate
under P5's first clause: it names an outside use, and its behaviour is general. But the task was
chosen first, and the product and its docs were then extended to meet its new requirement before
the freeze. GitHub Actions, Backstage and vanilla could not be extended that way. Round 8 therefore
measures R9 against a Seikika built for it. A held-out task needs to be one the product cannot
anticipate.

## 5. P4 comparison

### 5.1 What each arm was given

| Arm | Workspace adds | Docs | Recipe | Prompt's platform paragraph |
|---|---|---|---|---|
| `seikika` | `bin/` (the round's binaries), `seikika-docs/` | Operator skills, two shipped examples, the manifest schema, `--help` | **Yes:** `add-action` (the whole path for repository-owned actions offered from an org portal, including groups from `fb5dc3f`) | Where the binaries and docs are |
| `gha` | `bin/act`, `gha-docs/` (9.5 MB) | `github/docs` at `44f009da` (`content/actions/`, branch rules, code owners, Actions reusables, REST data for actions, deployments and repos), act's README at v0.2.98, `act-docs/src` at `35cab610` | No | Host mode, where the docs are, the `github.repository` deviation (G3), the credit for GitHub-only features "in the repository or the org", including teams, with "You need not rebuild such a feature here" (G2) |
| `backstage` | `backstage/` (create-app 0.9.4, install committed), `backstage-docs/` (52 MB) | `backstage/backstage` `docs/` at v1.57.0 (`6b4aecbb`) plus the scaffolder and catalog OpenAPI specs; the app's own `examples/template` | No | How to start the backend, the port, one static token per person including dave and erin, described as external service principals (B3), where the docs are |
| `vanilla` | Nothing (no `bin/`) | The language's standard library | No (none, recorded as such) | Languages installed |
| all | Same origins, checkouts, empty `org/`, local `commit.gpgsign=false` and `core.fsmonitor=false`, baseline beside the workspace | — | — | Identical task text, apart from paths, ports and tmux prefix (`diff` of `rendered/r8/seikika-1.md` against gha-1, backstage-1 and vanilla-1) |

### 5.2 Triage fixes

| Fix | Arm | Applied | Evidence |
|---|---|---|---|
| G1 | gha | yes | `fetch.sh` pins `44f009da`, v0.2.98 and `35cab610`; `setup.sh` copies them; `gha-1/gha-docs/{github-docs,act,act-docs}`; prompt sentence |
| G2 | gha | yes | "You need not rebuild such a feature here … a test of that step is credited when its configuration is committed and explained. For the three runs, say which steps GitHub would have performed" |
| G3 | gha | yes | "under act `github.repository` is an origin's path, not `acme/<name>`" |
| G4 / F1 | all | yes | `setup.sh` `local_git()` on every checkout and `org/` (checked in seikika-1, gha-1, backstage-1 and vanilla-1). The global config is off now. Seikika runs 1 and 3 and gha-1 saw `gpgsign=false` in their own output |
| B1 | backstage | yes | `fetch.sh` pins `6b4aecbb`; `backstage-1/backstage-docs/{docs,plugins}`; prompt sentence |
| B2 | backstage | yes | `build-app.sh` (create-app 0.9.4, Node 24.22.3); the workspace app's first commit is "yarn install" |
| B3 | backstage | yes | Prompt and overlay comment: "external service principal whose subject is the person's name" |
| B4 | backstage | yes | README §7 limit 7 (no credit; code-host lines listed separately in the write-ups) |
| F2 | all | yes | `bin/` only in seikika and gha; `<dest>.baseline.json` beside the workspace; `score.py lines` reads it from there |
| F3 | all | yes | `task-notice.md`: "a rejected push of a change that is not yet on `main`" |
| Shared (bin, baseline) | backstage | yes | `backstage-1` has no `bin/` and no `.baseline.json` |

### 5.3 What is still asymmetric

1. **A recipe for one arm.** The rivals got their platforms' general reference docs, as the
   previous audit asked. Seikika's `add-action` is a task-level recipe for the shape R1–R9
   describe: an org portal over repositories, an action each repository owns, approvers, groups
   defined once, test cases with the refusal set, a rollout loop, then request, approve, check and
   tell people. It is generic in names now. Its structure comes from four revisions against this
   benchmark's transcripts, and its R9 section was added for this round. The held-out effect is
   what keeps this from a failure: the recipe says nothing about how to change a README, and the
   runs' kept lines are mostly that work.
2. **Rival docs not updated for R9.** The G1 selection predates the new task. `gha-docs` has no
   `content/organizations/` team pages and no Teams REST data (`/orgs/{org}/teams` matches nothing
   in the three REST files). gha-1 spent calls 5–7 searching the REST data. It then wrote
   `PUT /orgs/{org}/memberships/…`-style team syncing in `apply-settings.py` from training.
   Backstage's whole `docs/` tree covers `kind: Group` (`descriptor-format.md`). Seikika got R9
   docs written for it.
3. **The credit grew for gha** (limit 9: P9 can be credited through a team configured as code).
   This is pre-registered and favours gha, as limit 1 does.

### 5.4 Probes and measures

- **Probes:** README §8's table is the same for every arm. Every arm has dave and erin by name;
  Backstage got their tokens. The adjudicator prompt takes the table as `{PROBES}` and `{LAST}`
  for every arm alike.
- **Measures:**
  - `score.py check` uses one `check_notice` for every arm.
  - `score.py lines` skips each arm's platform copy alike (`seikika-docs`, `gha-docs`,
    `backstage-docs`, `bin`).
  - `count.py` is unchanged.
  - `tally.py` reads the probe names from the records.
- **Strikes:** both adjudicated arms had only runtime data struck. For Seikika that is run records
  and the generated key. For vanilla it is run records, a run counter and clone metadata.
- **After the freeze:** only bookkeeping changed.
  - `isolation.py`, `prompt_check.py` and `measure.sh` read transcripts and workspaces and compute
    nothing the bar uses.
  - The adjudicator and fit-audit prompts were rendered after the runs.
  - All twelve run prompts were committed in `681cf8e` (15:33), before the first launch in
    `013c58f` (15:34), and none changed afterwards.
  - The 9 `prompt.json` files present are `identical: true` (seikika-1–3, gha-1, gha-2,
    backstage-1, vanilla-1–3), and their 9 isolation files are empty.

## 6. Corrections

1. **Rename round 8's group in the product:**
   - `group:release` and `user:dave` in `examples/federated-actions/README.md` (the `authz` and
     `tests` rows), for example to `group:security`, as the skills use;
   - the same names in `crates/seikika-authz/src/local.rs` tests, `crates/seikika-cli/src/action_tests.rs`
     and `scripts/m5-acceptance.sh` step 11.

   D01M8727W7FEED6VVRY6MB3JFR3 can't be edited, so leave it. This is cosmetic for round 8's
   numbers, because the task names those people.
2. **Label R9 in the record:** "Seikika's P9 passes and its R9 help come from a change built for
   round 8's task before the freeze (README §8, limit 8)". Report round 8's calls with "a shipped
   task skill tuned on rounds 4–7 and extended for R9" (§1).
3. **Seal the next held-out task before the product freezes.** Choose it after the freeze, or have
   someone who doesn't change the product choose it and commit only its hash before the freeze. No
   product or doc change may then anticipate it.
4. **Give the gha arm GitHub's team docs before another round with R9:**
   `content/organizations/organizing-members-into-teams/` and `src/rest/data/fpt-2022-11-28/teams.json`
   at the pinned commit. Uncurated, as G1 is.
5. **Check the prompts of gha-3, backstage-2 and backstage-3** with `prompt_check.py` when they
   finish. Without that check, those runs can't be compared under P1 and P4.
6. **Catalog entry:** fill `fitAudit` with both audits' verdicts, and set `status`, `frozenAt`
   (`8519cd5`) and `measure` (P1 to P11 for round 8).
7. **D01M86J769VGE6K5FJJQ76EP4AA §18:** add a short decision saying the skill no longer covers the
   handoff, since that file can't be edited. This is cosmetic.
