# Friction triage: the backstage arm, rounds 6 and 7

*Triage under rule P4 (symmetric arms) of [dx-benchmarks.md](../../../../docs/design/dx-benchmarks.md)
§4.1, prompted by [FIT-AUDIT.md](../FIT-AUDIT.md) §4 correction 7. The triager built no arm and has
no stake in the result. Sources: five transcripts (r6 backstage-1 to backstage-3, r7 backstage-1 and
backstage-2; bcIds in `r6/runs.tsv` and `r7/runs.tsv`), walked call by call with Python; the
adjudications `r6/adjudication/backstage-*.json` (r7 has none for this arm yet); the measures
`r{6,7}/backstage-*.count.json` and `*.lines.json`; and, read-only, the source app at `/tmp/bs/app`.
r7 backstage-3 had no finished transcript and is not covered. "#n" is the n-th tool call in a run's
transcript; each run's count matches its `count.json` (84, 56, 63, 77, 51).*

## Verdict

**The scaffold and the prompt cost this arm 10 to 16 calls per run, out of 51 to 84. Almost
all of that is one asymmetry: the arm has no Backstage documentation, while the Toolbox arm has
`toolbox-docs/`.** In every run, 10 to 12 calls went to reading compiled sources and `.d.ts` files
in `node_modules` to learn APIs that Backstage's own docs describe:

- the scaffolder's `/v2/tasks` and `/v2/dry-run`;
- `createTemplateAction` and `ActionContext`;
- the backend system's extension points;
- what a static token turns into.

Up to 3 more calls per run went to errors that some of those docs would have prevented.

Real environment faults were few and small:

- the app's git tree is dirty on arrival;
- an empty `bin/`;
- the host's git signing and fsmonitor settings, shared by every arm.

No run hit a start-up failure caused by the environment, a port clash, a `PATH` error or a failed
install. The backend booted on the first try in all five runs.

Separately, the local code host (README §7, limit 2) costs this arm 258 to 379 lines per run: a
catalog provider and git helpers that Backstage's GitHub integration would partly supply.

Most of the arm's cost is Backstage's own and is rejected below:

- a backend plugin for approvals, run records and audit (569–925 lines);
- an org CLI;
- type checks;
- starting and restarting a server.

All five runs fully passed (the three r6 adjudications), so delivery (H1) is not at stake.

**Fair equivalent of Toolbox's help.** Toolbox gets its shipped docs, not a solution. The same level
for Backstage is its upstream `docs/` tree at the matching release, plus the scaffolder and catalog
OpenAPI specs (as Toolbox gets its manifest schema). Backstage ships `examples/template` in the app
already, and gets no task recipe. Copy the whole tree, uncurated: picking pages for this task would
be fitting.

**Effect on the measures (estimate).** Round 6's H3 against Backstage held by a narrow margin:
32 vs 63 calls, a gap of 31 against W 28. Fix B1 replaces source reading with doc reading. Toolbox
runs spend 5–7 calls on their docs, so suppose 6 doc reads replace the source reads and the
doc-preventable errors disappear. Round 6's Backstage calls would then be about 78, 52 and 55:
median 55, W 26, gap 23. H3 against Backstage would turn **inconclusive**. It cannot become *lost*,
because that needs a Backstage median below 4. H2 (lines) is untouched: Toolbox has 139 lines and
Backstage 1,605–2,311. Even with the code-host glue left out, Backstage keeps about 1,300–1,950.
So the round 4–7 calls wins over Backstage depend on this asymmetry. FIT-AUDIT §1 already says
they should not be cited as the benchmark's result.

## Proposed fixes

| # | Fix | Files in `experiments/actions-ab/` |
|---|---|---|
| B1 | **Put Backstage's own docs in the workspace.** Copy them to `{DEST}/backstage-docs/`: the `docs/` tree of `backstage/backstage` at tag `v1.57.0` (commit `6b4aecbbe1159f48a3937dfd839e3018b85b608d`, 918 files), plus `plugins/scaffolder-backend/src/schema/openapi.yaml` and `plugins/catalog-backend/src/schema/openapi.yaml` from the same tag. Fetch it once as a pinned tarball, as the forks are, and copy the whole tree. In the prompt, add one sentence of the same form as Toolbox's: "Backstage's documentation for this release is in `{DEST}/backstage-docs`: the `docs/` tree of backstage/backstage at v1.57.0, and the OpenAPI specs of the scaffolder and catalog backends." Name no pages | `harness/fetch.sh`, `harness/setup.sh` (backstage case), `prompts/platform-backstage.md`, README §2 (the arm's row) |
| B2 | **Hand over a clean app.** create-app commits before `yarn install`, so `/tmp/bs/app` arrives with `M yarn.lock` (a 31,071-line diff) and a reordered `packages/app/package.json`. After `cp -a` in setup.sh, commit that state in the app's repo with a fixed identity and signing off. Also script how `/tmp/bs/app` is built: today the arm can't be rebuilt from this repo | `harness/setup.sh`; a new `harness/backstage/build-app.sh` |
| B3 | **Say what the tokens are.** The prompt says the overlay "gives ann, bob and carol a static API token each". The overlay's comment says each person can "call the API as themselves". Backstage treats them as external service principals, not users. Change the prompt to "a static token each (`backend.auth.externalAccess`: Backstage authenticates each as an external service principal whose subject is the person's name)". Make the overlay comment say the same. This states a fact about what the harness configured, as the port is stated. It says nothing about how to use the tokens | `prompts/platform-backstage.md`, `harness/backstage/app-config.local.yaml` (comment only) |
| B4 | **Record the code-host cost as a limit, not a fix.** Add to README §7: Backstage's GitHub integration (discovery provider, URL reader) would replace the catalog provider and file reads every run wrote for bare local origins. Toolbox reads git paths natively, and vanilla calls `git`. In round write-ups, list the backstage arm's code-host glue lines separately (r6: 376, 294, 379; r7: 258, 298). Keep the primary measure as pre-registered, and decide before round 8 is frozen whether to apply a credit like GitHub Actions' limit 1. Unverified: whether the GitHub module's publish actions could replace the push helper (`publish:github:pull-request` opens a PR; the task lands on `main`) | `README.md` §7; the round write-up |
| — | **Shared with every arm**, as the vanilla triage's F1 and F2: a local git config that turns off the host's commit signing (`commit.gpgsign=true` with Cursor's SSH signer) and `core.fsmonitor=true`; create `bin/` only in the arms that fill it; keep `.baseline.json` (21 MB) out of the workspace | `harness/setup.sh`, `harness/score.py` |

Each Backstage-specific fix (B1 to B3) gives this arm help another arm already has: shipped docs
and a schema, a clean platform copy, an accurate description of the environment. None of them
describes how to build the action.

## Where the calls went

Calls per run are listed as r6-1 / r6-2 / r6-3 / r7-1 / r7-2. Causes:

- **scaffold**: what `setup.sh` and the overlay put in the workspace;
- **prompt**: `platform-backstage.md`;
- **environment**: the host or the app copy;
- **platform**: Backstage's own cost;
- **agent**: the agent's own choices.

### Fixed or recorded

| Friction | Calls per run | Evidence | Cause | Fix or reject |
|---|---|---|---|---|
| Learning the scaffolder, backend-system and catalog APIs from compiled sources and `.d.ts` in `node_modules` | 9 / 6 / 11 / 8 / 8 | r6-1 #7–9 read `plugin-scaffolder-backend/dist/service/router.cjs.js` for `/v2/tasks` and `/v2/dry-run`; #10–11 `type ActionContext`, `createTemplateAction`; #13, #22–23 `CatalogService`, actions registry. r6-3 #5–12, #15, #23–24 (`scaffolderServiceRef`, `ScaffolderTask`, auditor types). r7-1 #17–19: three fruitless greps for nunjucks autoescape. r7-2 #5–8, #14–16, #20 | scaffold: no docs, while Toolbox has `toolbox-docs/`. The versions are recent (scaffolder-node 1.0.0 with zod v4, catalog-node 3.0.0), so training doesn't cover the exact signatures | **Fix B1.** `writing-custom-actions.md`, `dry-run-testing.md` (`supportsDryRun`, `ctx.isDryRun`), `backend-system/core-services/*`, `external-integrations/entity-providers.md` and the scaffolder OpenAPI spec (`/v2/tasks`, `/v2/dry-run`) cover these. Agents may still read some source; the estimate above assumes 6 doc reads |
| Learning what a static token becomes (auth and permission internals) | 3 / 4 / 0 / 4 / 4 | r6-1 #5–6 read `auth/external/static.cjs.js`: "Static tokens yield a service principal whose subject is the bare name (`ann`)". r6-2 #10–13 (`ServerPermissionClient`, `DefaultAuthService`, `accessRestrictions`). r7-1 #10–12, #16 (`getPluginRequestToken`: "Access to target plugin … restricted"). r7-2 #9–12 | prompt and scaffold: the prompt and overlay imply user identities; the doc that explains them is absent | **Fix B1 and B3.** `auth/service-to-service-auth.md` documents static tokens, `subject` and `accessRestrictions` |
| Errors that some docs would have prevented | 0 / 0 / 3 / 3 / 1 | r6-3 #18–19: extension points imported from `/alpha`, but they come from the main entry in these versions. r6-3 #39: `User` needs `spec.memberOf` (found in #38, a 121 s start loop). r7-1 #43, #45, #49: catalog rejected the templates: "/spec/output/observe must be string". r7-2 #32–33: the backend crashed ("Invalid backend feature at features[22] … received undefined") until the local modules got default exports | platform, made costlier by the missing docs | **Fix B1, partly.** `descriptor-format.md` marks `spec.memberOf` [required]. The pages checked don't state that a local module passed to `backend.add(import(…))` needs a default export, or that template outputs must be strings. Those stay the platform's cost |
| App's git tree dirty on arrival | 0 / 0 / 0 / 1 / 1 (noticed in all five) | All runs saw `M packages/app/package.json`, `M yarn.lock` (r6-1 #16, #76; r6-2 #5; r6-3 #5, #13; r7-1 #14, #69; r7-2 #47). r7-1 #70 and r7-2 #47 were spent on it. r7-2 then left its glue uncommitted: "Those `yarn.lock` and `packages/app/package.json` changes predate my work, so I'll leave the Backstage repo uncommitted". `/tmp/bs/app` shows the same diff | environment | **Fix B2** |
| No Backstage integration for a bare local origin: every run wrote a catalog entity provider and git helpers | about 2–3 edits per run; 376 / 294 / 379 / 258 / 298 lines | `catalogModule.ts` (73–234 lines), `git.ts` or `shared.ts` or `repoSource.ts` (68–155), and r6-1's `org.ts` (69) in each `lines.json`. r6-1 #17–19, r6-2 #16–17, r6-3 #16–17, r7-1 #20, #22, #24, r7-2 #17–18 | scaffold (limit 2). Every arm has the same origins, but only this one pays for them in glue | **Record, B4.** Giving this arm a code host its integrations support would change its origins and break "same origins" (README §4.1). So credit the cost or report it separately; don't change the help |
| Empty `bin/` in the workspace | 0 / 0 / 1 / 0 / 0 | r6-3 #3 `for f in bin/*` (no output). Also listed inside r6-1 #2, r7-1 #3 and r7-2 #2 | scaffold, every arm (`setup.sh` makes `$dest/bin` for all) | **Fix, shared** (vanilla triage F2) |
| Host git config: commit signing on, fsmonitor on | 0 dedicated; about 5–10 lines | r6-2 #14 shows `user.signingkey`, `core.fsmonitor=true`. r6-2 #16 wrote `GIT_CONFIG_GLOBAL: '/dev/null'` into its git env, and passed `-c commit.gpgsign=false` on commits (#29, #30, #51). r7-1 #3 shows `commit.gpgsign=true`; #20 added a `SAFE_CONFIG` array to `git.ts`, and every later git command carried `-c` flags | environment, every arm | **Fix, shared** (vanilla triage F1) |
| `.baseline.json` (21 MB, every file's text) at the workspace root | 0 dedicated | Read inside r6-1 #1–2 ("probably a harness artifact"), r6-3 #2 and r7-1 #3 | scaffold, every arm | **Fix, shared** (vanilla triage F2) |

### Rejected

| Friction | Calls per run | Evidence | Cause | Reject, with reason |
|---|---|---|---|---|
| Checking and declaring dependencies in `packages/backend/package.json` | 9 / 1 / 3 / 2 / 1 | r6-1 #12, #16 presence checks; #50–56: after lint's `no-undeclared-imports`, four separate `package.json` edits and `yarn install --mode=update-lockfile`. r6-3 #13, #28–29; r7-1 #13, #33; r6-2 #14; r7-2 #13. Every package was already hoisted, and the network and yarn cache worked | platform (a backend module declares its dependencies) and agent (r6-1 chose to lint and edited one dependency at a time) | **Reject.** Nothing in the scaffold was missing or broken. The docs' route (`yarn backstage-cli new` in `writing-custom-actions.md`) generates a module with its dependencies, which B1 makes visible |
| Type checks, lint, prettier and their fixes | 13 / 3 / 5 / 3 / 4 | r6-1 #46–49, #57–65 (nested ternary, use-before-define, control regex); r6-2 #46–48; r6-3 #35–36, #55–57; r7-1 #62–64 (including a probe that `tsc` sees the new files); r7-2 #41–44 | platform (TypeScript backend) and agent (lint isn't required) | **Reject.** It is the platform's language cost; the lint pass was the agent's choice |
| Starting, waiting for, restarting and stopping the backend | 5 / 5 / 4 / 7 / 4 | First starts used fixed sleeps of 40–45 s (r6-1 #15, r6-2 #32, r6-3 #14, r7-2 #32), but the backend listened about 8 s after start (r6-2 log 13:53:10 → "Listening on :7670" 13:53:18). r6-1, r6-3 and r7-1 started the stock backend before writing code, and it worked. Restarts tested persistence (r6-2 #49, r7-1 #68) and rehearsed the handoff (r6-1 #80, r6-2 #54, r7-1 #74, r7-2 #50). r6-3 #52 waited 186 s on a restart that tmux lost (its recovery is under agent mistakes) | platform (Backstage is a server; Toolbox r6 spent 1–3 calls on its own) and agent | **Reject.** No start failed for an environment reason. B1 makes `/.backstage/health/v1/readiness` (`root-health.md`) findable, which shortens waits, not calls. No prompt change: Toolbox's readiness help is a product feature (`serve --detach`) |
| Watch-mode reloads after every backend edit | 0 calls; 15–20 s sleeps inside test commands | r6-3 #42, #44, #46, #51, #58; r7-1 #50, #52, #55, #58, #65; r7-2 #38, #45; r6-2 #40 | platform (`backstage-cli package start` restarts on change) | **Reject.** This is the dev server's behaviour, and it costs wall time only |
| The approval gate, run store, audit trail and org CLI | most edits: plugin 569–925 lines, CLI 132–189 lines | `plugin.ts` / `actionsPlugin.ts` / `selfServicePlugin.ts` in every `lines.json`; one large edit each (r6-1 #24, r6-2 #19, r6-3 #25, r7-1 #27, r7-2 #21) | platform: the scaffolder has no approval step or run record | **Reject.** This is the gap the benchmark measures. A recipe for it would be fitting (FIT-AUDIT §4.7 offers it only as "a recipe for all") |
| Closing bypasses: token `accessRestrictions`, a permission policy, removing the guest provider, probing direct scaffolder calls | 1 / 2 / 1 / 5 / 1 | r6-1 #43; r6-2 #37–38; r6-3 #47; r7-1 #28, #30–32 (policy module, guest provider), #54; r7-2 #38 | platform defaults (create-app's allow-all policy and guest provider) and agent (no probe tests bypass) | **Reject.** Those are the defaults Backstage ships; B1 makes them cheaper to change |
| Agent mistakes | 0 / 2 / 2 / 0 / 2 | r6-2 #24–25: `sed s/alpha/$r/` turned `v1alpha1` into `v1beta1`. r6-2 #33–34: CLI flag parsing. r6-3 #53–54: recovering a restart typed into tmux during shutdown. r7-2 #34–35: EPIPE in its git helper | agent | **Reject** |
| `PATH` for node and yarn | 0 | Every node or yarn command exported `/tmp/bs/node24/bin` as the prompt says; no "command not found" in any run | — | **Reject.** There's no fault; the prompt states it |

## Sources

- Transcripts: `/tmp/cursor/cloud-agent-transcripts/2026-11-24T15-10-20Z-b058/<bcId>/transcript.json`
  for bc-c7a96db9 (r6-1), bc-30c5d2df (r6-2), bc-d5073979 (r6-3), bc-ff63a896 (r7-1) and
  bc-f7a616a3 (r7-2).
- Backstage docs at `v1.57.0`, checked with `gh api repos/backstage/backstage/contents/…?ref=v1.57.0`:
  - `docs/auth/service-to-service-auth.md` ("Static Tokens", `subject`, `accessRestrictions`);
  - `docs/features/software-templates/writing-custom-actions.md` (`createTemplateAction`,
    `scaffolderActionsExtensionPoint`, `yarn backstage-cli new`);
  - `docs/features/software-templates/dry-run-testing.md`;
  - `docs/backend-system/core-services/root-health.md`;
  - `docs/features/software-catalog/descriptor-format.md` (`spec.memberOf` [required]);
  - `docs/features/software-catalog/external-integrations/entity-providers.md`.
- The tree has 918 files under `docs/`, 763 of them Markdown. The OpenAPI specs at the same tag
  are `plugins/scaffolder-backend/src/schema/openapi.yaml` (it defines `/v2/tasks` and
  `/v2/dry-run`) and `plugins/catalog-backend/src/schema/openapi.yaml`.
- Not verified: which GitHub module actions could commit a file straight to `main` (B4).
