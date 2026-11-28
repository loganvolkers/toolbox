# Seikika vs Backstage: comparison factors and reproducible benchmarks

*Research brief, 2026-11-23. About the box (running, pinning, reporting, distributing), not a tool.
Backstage facts were read in `backstage/backstage` at tag `v1.57.1` (`9342a5de`) and master `b3ed9690`
(2026-11-23). The create-app template and the docs cited here are identical at both. Source tags:
[P] primary, read first-hand. [P-copy] primary text read through a third-party or cached copy.
[S] secondary. [I] inference. [N] cited but not read.*

---

## Conclusions

1. **Fourteen factors, all measurable on one 4-vCPU / 8 GB VM without agents.** They cover all nine
   ISO/IEC 25010:2023 characteristics except Safety, which does not apply to a portal. DX is one
   factor of fourteen. The agent DX Benchmarks in `.seikika/benchmarks/` stay the evidence for the
   agent claims, and these benchmarks complement them.
2. **Today Seikika is better on four factors, by evidence from source and docs, still to be
   confirmed at runtime:**
   - *authorization defaults*: create-app ships the `allow-all` permission policy and the guest
     provider; Seikika has one enforcement point that rejects unauthenticated reads;
   - *offline analysability*: Seikika can validate, test and plan without a server.
     `backstage-cli catalog validate` needs an authenticated instance;
   - *repo-scoped use*: Backstage has no server-less mode;
   - *signed release provenance*: `github-attestations = true` in Seikika; no npm provenance on the
     `@backstage/*` packages checked.
3. **Seikika "should be better" on eight factors, none measured head-to-head.** These are
   installability, footprint and cold start, read latency, freshness of derived data, extension
   isolation, supply-chain footprint, upgrade cost and recoverability. The design argument for each
   comes from Parnas, Lehman, the Churn Rule and the Tail at Scale. Each has a parity rule below that
   could falsify it.
4. **Backstage is better on functional completeness** (TechDocs, scaffolder, search engines, 162
   plugin directories in-repo plus community plugins) **and on project health** (Scorecard
   Maintained, Code-Review and Contributors at 10/10). Seikika is pre-1.0 (v0.1.1), its actions
   (M5) are paused, and it has one maintainer.
5. **On federation, both lack it, so it is parity.** Backstage's "module federation" is frontend
   code loading, and its scaling doc covers identical replicas or split backends, not
   catalog-of-catalogs. Seikika federation is designed but not built (D68).
6. **One Seikika weakness the benchmarks will expose:** in-image connectors are spawned without
   `env_clear()`, as the server's UID (`crates/seikika-api/src/connectors.rs` `supervise`). With
   `DATABASE_URL` in the environment, a connector inherits full database credentials. The process
   boundary isolates memory, not credentials. Fix this before running F7, and before claiming
   isolation in docs.
7. **Upgrade cost is the factor where Seikika's story matters most and its evidence is thinnest.**
   Backstage's last 12 main-line changelogs (v1.46–v1.57) carry 145 `**BREAKING**` entries [P,
   counted]. Seikika has no release history long enough to compare, and no published stability
   policy yet.

---

## 1. Factors

Claim key: **Better**: the evidence today favours Seikika. **Should**: the design favours Seikika,
unmeasured. **Parity**: neither leads. **Backstage**: Backstage leads.

| # | Factor | ISO 25010:2023 / framework | Why it matters | Claim | Headline metric | Parity rule (Seikika at least at parity when…) |
|---|---|---|---|---|---|---|
| F1 | Functional completeness | Functional suitability: completeness | The first question in an evaluation. CNCF: platforms should cover "the most common use cases" [P TAG] | **Backstage** | Scenarios passed out of a fixed list of 12, configuration only | Seikika passes every in-scope scenario that Backstage passes. Out-of-scope ones are reported as Backstage wins, not dropped |
| F2 | Installability, time to first portal | Flexibility: installability; Interaction: learnability. DevEx feedback loops [P-copy DevEx]. CNCF "latency to build and deploy a brand new service" [P TAG] | Time to value is the first adoption barrier (`docs/red-team.md` A1) | **Should** | Wall time and bytes downloaded from a clean VM to an authenticated read returning the full corpus. Lines the operator authors | Seikika median ≤ Backstage median on wall time, bytes and authored lines |
| F3 | Footprint and cold start | Performance efficiency: resource utilization; Flexibility: scalability. USE method [P Gregg]. Charter goal 1 (one image + Postgres) | Cost per instance, preview environments, scale-to-zero | **Should** | Idle RSS and peak RSS (cgroup `memory.peak`), CPU-seconds to ingest, image size, exec→ready | Seikika ≤ Backstage on each, reported per metric, no aggregate |
| F4 | Read latency and capacity | Performance efficiency: time behaviour, capacity. Tail at Scale [P Dean & Barroso] | An entity page fans out to many reads, so per-call p99 becomes the page's median | **Should** | p50/p95/p99 at fixed open-loop rates. Capacity = max RPS with p99 < 200 ms and errors < 0.1%. Requests per entity page | Seikika p99 ≤ Backstage p99 at every rate both sustain, and capacity ≥ |
| F5 | Freshness (entities and derived data) | Functional correctness (timeliness), time behaviour. DevEx feedback loops | Stale ownership or scores erode trust. Plan §6.6 sets webhook→visible < 60 s p95 | **Parity** for entities, **Should** for derived data | Commit→visible p50/p95 over 20 trials, per ingestion mode. Commit→score changed | Seikika p95 ≤ Backstage p95 in each mode |
| F6 | Secure defaults | Security: confidentiality, authenticity, integrity. CISA Secure by Design [P]. ASVS 5.0 V13 [P]. SSDF PW.9 [P]. CNCF "secure by default" [P TAG] | ASVS V13: "The application's default configuration must be secure for use on the Internet." | **Better** (authz), **Parity** (unauthenticated API) | Probes failed out of 7, in dev and in production config | Seikika fails no probe that Backstage passes |
| F7 | Extension isolation (blast radius) | Security: integrity, resistance; Maintainability: modularity. Parnas [P-copy] | Backstage's threat model: "there is no boundary between plugins that run on the same host" [P] | **Should**, currently **Parity** on credentials | Successful canary actions out of 6, by a malicious extension | Seikika successes ≤ Backstage's. Target: 0 beyond its own config |
| F8 | Supply chain and dependency footprint | Security: resistance. SLSA v1.2 Build track [P]. SSDF PS.2, PW.4 [P]. OpenSSF Scorecard [P] | Every transitive package is attack surface and upgrade work | **Better** (provenance), **Should** (footprint), **Backstage** (project health) | SBOM package count, known vulns by severity, % artifacts with verifiable provenance, Scorecard per check | Seikika ≤ on packages and high/critical vulns; ≥ on provenance %. Scorecard reported per check, with no aggregate claim |
| F9 | Upgrade cost (adopter-side churn) | Maintainability: modifiability; Flexibility: replaceability. Lehman I/II [P-copy]. Churn Rule and Hyrum's law [P SWE@G] | "Software engineering is programming integrated over time" [P SWE@G]. Upgrades are Backstage's top reported pain ([S], research.md §2) | **Should** (no track record) | Over k = 6 releases: adopter files and lines changed, errors left after the documented tools run, hand-edit lines to green | Seikika ≤ Backstage on all three over the same number of releases |
| F10 | Reproducibility and analysability of config | Maintainability: analysability, testability. DORA change fail rate [P]. Beyoncé Rule [P SWE@G] | Previewing a change before deploy is how a platform lowers its change fail rate | **Better** | Offline validate (yes/no, time), change previews supported out of 10, byte-identical rebuilds, config-like DB rows not in git | Seikika ≥ on previews and determinism, offline ≥ |
| F11 | Extension cost and modularity | Maintainability: modularity, reusability; Flexibility: adaptability. Parnas; Ousterhout "deep modules" [P-copy]. DORA "design for extensibility" [P] | Extensions are where portals accrete cost | Mixed: **Should** (isolation, languages), **Backstage** (ecosystem, UI richness) | For one fixed reference extension: LOC, files touched outside it, core rebuild (y/n), imported API symbols, languages | Seikika ≤ on outside files and rebuilds; LOC reported |
| F12 | Fit to org structure: repo-scoped and federated | Compatibility: interoperability. Conway [P]. Team Topologies TVP [P]. DORA loosely coupled teams [P]. CNCF "optional and composable" [P TAG] | Teams should be able to "do most of their testing on demand, without requiring an integrated test environment" [P DORA] | **Better** (repo), **Parity** (federation: neither has it) | Repo: services needed, CI wall time, secrets needed. Federation: entities propagated, staleness, tolerated version skew | Repo: Seikika needs ≤ services and ≤ time. Federation: not runnable until D68 is built |
| F13 | Agent and API surface | Interaction capability: self-descriptiveness, operability; Compatibility: interoperability. CNCF "GUIs, APIs, command-line tools" [P TAG]. SPACE efficiency/flow [P-copy] | Agents are a primary audience. Tool-list size and calls per answer are direct costs | **Parity → Should** | `tools/list` tokens; schema and annotation coverage; calls and bytes for 20 scripted questions; REST/MCP/CLI result equality | Seikika ≤ on tokens and calls; ≥ on coverage and equality |
| F14 | Recoverability and rollback | Reliability: recoverability, fault tolerance. DORA failed deployment recovery time [P]. SSDF 1.2 draft PS.4.3 rollback [S] | Rollback by image is a charter claim. Backstage documents a manual Knex rollback [P] | **Should** (Seikika side unverified) | Recovery time after kill -9 mid-ingest; errors during a Postgres restart; rollback N→N−1 manual steps | Seikika recovery ≤ Backstage's, and rollback needs zero manual DB steps |

### 1.1 Common harness (applies to every factor)

- **Machine:** one VM with 4 vCPU and 8 GB usable RAM. Record its CPU model, kernel and the balloon
  state. Postgres 16 runs locally, one database per product. Set the CPU governor to `performance`
  where available. Nothing else runs during a timing.
- **Pins:**
  - *Backstage:* `@backstage/create-app@0.9.4` (release 1.57.1 in `backstage.json`), Node 24.15.x
    LTS, and the Yarn version the template generates. Production mode is `NODE_ENV=production
    node packages/backend --config app-config.yaml --config app-config.production.yaml`, or the
    template's `packages/backend/Dockerfile`.
  - *Seikika:* a release tag, its installer binaries and the image from `image/Dockerfile`, all
    recorded by digest.
  - *Prerequisite:* get approval (Q18) to run `npx @backstage/create-app`. A previous session's
    policy refused it (`docs/research-log.md`,
    results-spec row).
- **Corpus:** one seeded, deterministic synthetic org written as `catalog-info.yaml` files: Domains,
  Systems, Components, APIs, Groups and Users with owners and `dependsOn`. Sizes are S = 1k, M = 10k
  and L = 100k entities. L may exhaust 8 GB on either side, and that is a result.
  - Backstage ingests the files through `catalog.locations` (`type: file`); Seikika through the
    git-files connector, on the same directory.
  - **Equivalence gate:** before any timing, both APIs must return the same entity count and the
    same SHA-256 over sorted `(ref, owner, relations)`.
- **Clients:** Backstage uses a static `backend.auth.externalAccess` token [P]; Seikika uses a PAT
  from `seikika token create`.
- **Statistics:**
  - at least 5 repetitions; cold runs start from a fresh database and dropped page cache;
  - report the median, min and max;
  - latency uses an open-loop, constant-arrival generator (k6 `constant-arrival-rate` or vegeta)
    with HDR histograms, 60 s warm-up and 300 s measured;
  - pin the load generator's version and run it on the same VM, with CPU affinity separated by
    `taskset`.
- **Pre-registration:** bars and parity rules go into the experiment README before the first run,
  following `docs/design/dx-benchmarks.md` §4. Losses are
  published as written.
- **DORA's warnings apply** [P]: no single metric, and no metric set as a goal (Goodhart).

### 1.2 Measurement procedures

**F1 Functional completeness.**
- *Scenarios* (pass/fail, configuration only, no code):
  - catalog browse and filter;
  - ownership view;
  - relation graph;
  - full-text search;
  - TechDocs page render;
  - template / self-service run;
  - scorecard with levels;
  - OpenAPI render;
  - Kubernetes view;
  - notifications;
  - custom permission rule;
  - MCP read tool.
- *Source of the list:* draw it from the D47 Backstage parity audit, so it is not chosen to favour
  Seikika.
- *Backstage's add-ons:* Backstage may use community plugins (for example Tech Insights for
  scorecards). Record each one as an add-on, with its install lines.

**F2 Installability.**
- *Start state:* a clean VM image with Node, Yarn, Docker and Postgres preinstalled at the pins, and
  empty npm, Yarn and Cargo caches.
- *Steps:* Backstage goes create-app → Postgres config → corpus locations → static token →
  `yarn build:backend` → start. Seikika goes release installer → starter manifest → `seikika serve
  --database-url`.
- *Stop condition:* the first authenticated catalog read that returns the full corpus count.
- *Recorded:*
  - wall time;
  - bytes received (`/proc/net/dev` delta);
  - disk footprint (`du -sb` of the app and caches);
  - commands run, and lines authored by the operator (`git diff --numstat` against the generated
    starting point).

**F3 Footprint and cold start.**
- *Setup:* run each product in its own cgroup (`systemd-run --scope`) with the M corpus.
- *Recorded:*
  - `memory.peak` during first ingest;
  - idle RSS/PSS (`smem`) 5 minutes after ingest;
  - `cpu.stat usage_usec` for the ingest;
  - compressed image size (`docker save | gzip -c | wc -c`);
  - cold start as exec → readiness 200, with a populated database. Readiness is
    `/.backstage/health/v1/readiness` [P] for Backstage and `/readyz` for Seikika.
- *USE check during ingest* [P Gregg]: CPU utilization, run-queue saturation (`vmstat`) and errors.

**F4 Read latency and capacity.**
- *Endpoint pairs:*
  - get by ref: Backstage `/api/catalog/entities/by-name/:kind/:namespace/:name` [P], Seikika
    `/v1/entities/{id}`;
  - filtered list: Backstage `/entities/by-query` [P], Seikika `/v1/entities` with filters;
  - full-text search: Backstage `/api/search/query` with the template's pg engine, Seikika search.
- *Rates:* 50, 100, 200 and 500 RPS, single replica, M corpus.
- *Authorization configurations:* run Backstage twice, with its default `allow-all` and with a
  minimal ownership policy. Run Seikika with its OpenFGA co-process. This compares like with like.
- *Fan-out:* headless Chromium loads one component page and counts fetch/XHR requests and time to
  network idle. Dean & Barroso: at 100 parallel calls with a 1% one-second tail, "63% of user
  requests will take more than one second" [P].

**F5 Freshness.**
- *Trial:* commit an owner change to one `catalog-info.yaml`, then poll the API every 100 ms until it
  returns the new owner. 20 trials per mode.
- *Modes:*
  - (a) default pull. Backstage re-processes locations every 100–150 s by default [P,
    `CatalogBuilder.ts`]; Seikika uses git-files polling;
  - (b) nudged: Backstage `POST /api/catalog/refresh` [P] and Seikika `seikika refresh`.
- *Derived data:* time until a rule that reads the owner field changes outcome. Backstage needs the
  Tech Insights community plugin with its fact-retriever schedule; record that schedule.

**F6 Secure defaults.**
- *Configurations:* run the probe script against (i) the documented dev mode (`yarn start`, and
  `seikika serve` with no flags) and (ii) the template's production config and image, both unchanged.
- *Probes:*
  - P1, bind address (`ss -ltnp`);
  - P2, unauthenticated catalog GET returns 401;
  - P3, sign-in possible without an IdP;
  - P4, an authenticated non-owner can delete an entity or unregister a location;
  - P5, a non-owner can read an owner-only field;
  - P6, the frontend bundle is served without auth;
  - P7, the number of config changes the docs require before an Internet-facing deployment.
- *Expected from source reading (to verify at runtime):*
  - Backstage dev fails P1 (empty host means all interfaces [P], and the production template says
    `':7007'` "bind[s] to all interfaces"), P3 (guest) and P4 (allow-all);
  - Backstage prod refuses guest unless `dangerouslyAllowOutsideDevelopment` [P], passes P2 (the
    default auth policy since 1.24 [P]), and still fails P4 once anyone can sign in;
  - Seikika binds loopback, returns 401, and shapes facets.

**F7 Extension isolation.**
- *Canary:* install the same canary as a Backstage backend module and as a Seikika in-image
  connector.
- *Attempts:*
  - (1) read the host's environment secrets;
  - (2) read app-config or manifest secrets on disk;
  - (3) connect to Postgres with discovered credentials;
  - (4) read another component's tables (auth sessions, native values);
  - (5) call another plugin's API with a self-minted service token;
  - (6) write outside its own directory.
- *Score:* the number of attempts that succeed.
- *Expected:* Backstage succeeds on all six by its own threat model [P]. Seikika today likely
  succeeds on (1), (3) and (4) because connectors inherit `DATABASE_URL` [P, repo code].

**F8 Supply chain.**
- (a) An SBOM of each deployable image with a pinned syft, counted by ecosystem.
- (b) Known vulnerabilities with pinned grype or osv-scanner against a dated DB snapshot, counted by
  severity.
- (c) OpenSSF Scorecard, the same version in the same `--repo` mode, on the pinned commits. The local
  7.2 for Seikika and the API's 5.7 for Backstage are **not** comparable.
- (d) Provenance:
  - `gh attestation verify` on every Seikika release asset and the image;
  - for every `@backstage/*` package in the generated lockfile, whether npm `dist.attestations`
    exists (or `npm audit signatures`), as a percentage.

**F9 Upgrade cost.**
- *Backstage:*
  - generate an app at release N−6 with the create-app version that release shipped;
  - apply a scripted customisation set (three added plugins, one custom page, a permission
    policy, the corpus);
  - upgrade to N by the documented path: `yarn backstage-cli versions:bump --release N`, then the
    create-app template diff (the Upgrade Helper, or `git diff` of `packages/create-app/templates`
    between tags), then the versioned codemods [P].
- *Seikika:* bump the pinned release in the installer step and image tag, then run `seikika validate
  && seikika test && seikika plan --local`.
- *Recorded for both:*
  - adopter `git diff --shortstat`;
  - errors left after the documented tools: `tsc`, `yarn test`, backend start and F1 probes for
    Backstage; validate, test and probes for Seikika;
  - hand-edit lines needed to get green.
- *Mechanical side metric:* `**BREAKING**` entries between N−6 and N, restricted to packages in the
  adopter lockfile.
- *Window:* k = 6 matches Backstage's six-month backport window for high and critical fixes [P].
- *Until Seikika has six releases:* run per release, and label the factor "not yet comparable".

**F10 Analysability.**
- (a) With the network down and no server, validate the full config and corpus, timed. Backstage's
  `catalog validate` requires an authenticated instance [P], so its time includes starting an
  ephemeral one.
- (b) Ten scripted changes (owner, new annotation, rule threshold, permission rule, new kind). For
  each, does a tool show the effect diff on entities, scores and permissions before deploy?
- (c) Rebuild from git twice and compare canonical JSON dumps of the API state byte for byte.
- (d) Count the database rows holding config-like state that is not in git. Examples: locations
  registered through the API in Backstage, native values in Seikika.

**F11 Extension cost.**
- *Reference extension (fixed spec):* ingest a JSON incident feed per service, attach an open-sev1
  count, and add one scorecard rule. Maintainers implement it once per product, idiomatically.
  Record any published plugin that already does it.
- *Recorded:*
  - LOC (`tokei`);
  - files changed outside the extension (Parnas: hide the decision, so ideally 0);
  - whether the core image must be rebuilt (a Backstage backend plugin is added in
    `packages/backend/src/index.ts`, so yes);
  - distinct imported framework symbols, against protocol message types;
  - whether the extension runs standalone (the box-and-tools rule).

**F12 Org-structure fit.**
- *Repo scope:* on a pinned open-source repo in CI (GitHub Actions, or `act` locally), produce a
  scored report on that repo's own descriptors and standards with no long-running service. Record
  services started, wall time, bytes downloaded and secrets needed.
  - Seikika: `actions/report` or `seikika report`.
  - Backstage: the nearest equivalent, an ephemeral backend in the job plus `backstage-cli catalog
    validate`; scoring adds Tech Insights.
- *Federation:* three team portals and one parent, with children one release apart. Record entities
  propagated, ownership intact, staleness p95, and tolerated version skew.
  - Backstage: a parent ingesting children through URL locations or a custom provider. Its skew
    policy requires core groups on the same release [P].
  - Seikika: not runnable until D68 is built.

**F13 Agent and API surface.**
- (a) Tool count and `tools/list` size in tokens (`o200k_base`), with default config.
- (b) The percentage of tools with input and output schemas and read-only annotations.
- (c) A fixed oracle script answers 20 catalog questions with the minimal call plan; count calls and
  response bytes.
- (d) Whether a versioned, published OpenAPI contract exists, and whether errors are RFC 9457.
- (e) Whether the same query gives identical sets over REST, MCP and CLI.
- (f) Which MCP auth flows each supports. Backstage has OAuth with CIMD [P].

**F14 Recoverability.**
- (a) `kill -9` the server during M ingest, restart it, and time until the equivalence gate passes.
- (b) Restart Postgres under 100 RPS; count errors and time to recover.
- (c) Upgrade N−1→N, then roll back to N−1 on the same database, and count the manual steps.
  - Backstage documents running Knex `migrate:down` "in the new version", otherwise "The migration
    directory is corrupt" [P].
  - Seikika's migration and rollback behaviour was not verified for this brief.

---

## 2. Verified Backstage facts

All of these were read at `v1.57.1` (`9342a5de`) and master `b3ed9690` (2026-11-23).
`git diff v1.57.1 HEAD` is empty for `packages/create-app/templates/default-app`,
`docs/overview/versioning-policy.md`, `docs/getting-started`, `docs/deployment`, `docs/auth/guest`
and the guest provider source.

| Fact | Evidence |
|---|---|
| Latest release **v1.57.1** (tags: v1.57.0, v1.57.0-next.0–2, v1.57.1; no v1.58) | `git ls-remote --tags` [P] |
| `@backstage/create-app` **0.9.4** (`latest`, published 2026-11-17; `next` 0.9.4-next.2) | npm registry JSON; `packages/create-app/package.json` [P] |
| Node **`^24.15.0 \|\| ^26.13.0`**. Policy: "the two most recent Node.js releases that have reached LTS"; "supports Node.js 24 and 26 as of the 1.57.0 release" | `templates/default-app/package.json.hbs`; `docs/overview/versioning-policy.md` §Node.js Releases [P] |
| Dev database: `client: better-sqlite3`, `connection: ':memory:'`. Production template: `client: pg` via `POSTGRES_*` env | `app-config.yaml.hbs`, `app-config.production.yaml`; `docs/tutorials/switching-sqlite-postgres.md` [P] |
| Auth: the template installs `plugin-auth-backend-module-guest-provider` and sets `auth.providers.guest: {}` in **both** dev and production configs. The guest provider throws outside `NODE_ENV=development` unless `dangerouslyAllowOutsideDevelopment: true` | `packages/backend/src/index.ts`; `plugins/auth-backend-module-guest-provider/src/authenticator.ts`, `resolvers.ts`; `docs/auth/guest/provider.md` [P] |
| Permissions: `permission.enabled: true`, with **`plugin-permission-backend-module-allow-all-policy`** installed. "Backstage ships with a default allow-all policy" | template `app-config.yaml.hbs`, `index.ts`; `docs/permissions/getting-started.md` [P] |
| Backend-to-backend auth is on by default (self-signed tokens). It can be disabled by `backend.auth.dangerouslyDisableDefaultAuthPolicy`. Built-in protection against unauthorized access dates from v1.24 | `docs/auth/service-to-service-auth.md`; `docs/overview/threat-model.md` [P] |
| Threat model: signed-in users "generally have full access to all information and actions". "There is no boundary between plugins that run on the same host". The project does not "sandbox or limit the access of plugins". The frontend bundle is not protected by default | `docs/overview/threat-model.md` [P] |
| Default listen host `''` (all interfaces). The production template's `listen: ':7007'` comment: "we bind to all interfaces, the most permissive setting" | `backend-defaults/.../http/config.test.ts`; `app-config.production.yaml` [P] |
| Releases: main line **monthly** ("Tuesday before the third Wednesday"), `next` weekly. Versions do **not** follow semver: "Each new minor version can contain new functionality, breaking changes, and bug fixes". High and critical security fixes are backported "for the last 6 months if feasible" | `docs/overview/versioning-policy.md` [P] |
| Upgrades: `yarn backstage-cli versions:bump` (with `--release`, `--pattern`); downgrades across 2–3 releases "may result in package mismatches". Template changes do not reach existing apps; adopters read the create-app CHANGELOG or the **Upgrade Helper** (`backstage/upgrade-helper`, last pushed 2026-04-20), then run versioned codemods | `docs/getting-started/keeping-backstage-updated.md`; `gh api repos/backstage/upgrade-helper` [P] |
| Breaking-change volume: `**BREAKING**` entries per main-line changelog, v1.46→v1.57: 9, 5, 18, 26, 13, 9, 3, 9, 7, 12, 3, 31 (**145** total, repo-wide, not filtered to the template's packages) | `docs/releases/v1.*.0-changelog.md` [P, counted] |
| Rollback: downgrading needs manual Knex `migrate:down` / `migrate:rollback`, run from the newer version. There is no CLI support yet (issue #6366) | `docs/tutorials/manual-knex-rollback.md` [P] |
| Scaling: identical instances that share the database, or split backends by plugin. Multi-region "is not a pattern that there is built-in support for" | `docs/deployment/scaling.md` [P] |
| **No catalog federation and no repo-scoped mode.** "Federation" in the docs means webpack/Rspack **module federation** of frontend code. `root-instance-metadata` notes it cannot list plugins "across all instances". `backstage-cli catalog validate` needs an authenticated instance (`auth login`, `--instance`) | `docs/frontend-system/building-apps/07-module-federation.md`; `docs/backend-system/core-services/root-instance-metadata.md`; `docs/tooling/cli/module-catalog.md` [P] |
| Catalog pull re-processing interval: 100–150 s by default | `plugins/catalog-backend/src/service/CatalogBuilder.ts:780` [P] |
| Health endpoints: `/.backstage/health/v1/readiness` and `/liveness` | `backend-defaults/.../createHealthRouter.ts` [P] |
| Template backend image: `node:24-trixie-slim` plus `libsqlite3-dev`; `yarn workspaces focus --all --production` | `templates/default-app/packages/backend/Dockerfile` [P] |
| Repo surface: 162 `plugins/*` and 78 `packages/*` directories; 47 catalog-backend Knex migrations | tree at `b3ed9690` [P] |
| Adoption guidance: a "central team that owns your Backstage deployment", including "on-call support". Spotify's self-reported KPI: time to 10th PR "down 55% two years after deploying". This is not a staffing figure, and it is self-reported | `docs/overview/adopting.md` [P] |
| Provenance: npm `dist.attestations` is absent for `@backstage/create-app` 0.9.4, `cli` 0.36.8, `backend-defaults` 0.19.0, `core-components` 0.18.16 and `plugin-catalog-backend` 4.0.5. `.github/workflows` has no `provenance` or `attest` | npm registry; repo grep [P] |
| OpenSSF Scorecard (API, 2026-11-23, commit `b3ed9690`): **5.7**. 10s on Maintained, Code-Review, Contributors, Security-Policy, Dependency-Update-Tool, License and Packaging; 0 on Dangerous-Workflow, Token-Permissions, Pinned-Dependencies, Vulnerabilities and Fuzzing | `api.securityscorecards.dev` [P]. `seikika/seikika` has no API entry |

**Seikika facts used above** (read in this repo):
- `crates/seikika-api/src/access.rs` returns 401 `unauthenticated` without IAP, OIDC or PAT
  credentials.
- `crates/seikika-cli/src/server.rs` binds loopback by default.
- `serve` without `DATABASE_URL` uses an in-memory store.
- `connectors.rs` `supervise` spawns connectors with no `env_clear`.
- `dist-workspace.toml` sets `github-attestations = true`.
- `image/Dockerfile` is Debian slim with OpenFGA 1.22.1 (checksum-pinned), running as UID 65532.
- `Cargo.lock` has 405 packages.
- The `scale-slo` workflow sets a budget of 100k entities, 2M relations and 100 rules in ≤ 10 s on 4
  vCPU.
- technology-critique §7.1 records a full recompute of 2.9 s with a 576 MB peak on 4 vCPU.

---

## 3. Sources

| Source | URL | Access |
|---|---|---|
| ISO/IEC 25010:2023, abstract | https://www.iso.org/standard/78176.html | [P] abstract only (paywalled) |
| ISO/IEC 25010:2023, foreword and scope (VDE preview) | https://assets.vde-verlag.de/iec-normen/preview-pdf/info_isoiec25010%7Bed2.0%7Den.pdf | [P-copy] via search cache: nine characteristics, 2023 changes |
| ISO 25010 sub-characteristics | https://iso25000.com/index.php/en/iso-25000-standards/iso-25010 ; https://university.kurz.fyi/computer-science/software-distributed/software-engineering/the-iso-iec-25010-quality-model | [S]. The first page was partly read (curl blocked by mod_security); the full list is from the second, cross-checked against the foreword |
| DORA software delivery metrics (five metrics, pitfalls) | https://dora.dev/guides/dora-metrics/ | [P] (updated 2026-01-05) |
| DORA capability: platform engineering | https://dora.dev/capabilities/platform-engineering/ | [P] (updated 2026-01-12) |
| DORA capability: loosely coupled teams (quotes Conway) | https://dora.dev/capabilities/loosely-coupled-teams/ | [P] (updated 2025-10-20) |
| Forsgren, Humble, Kim, *Accelerate* (2018) | — | [N]. The metrics are cited through dora.dev |
| Forsgren et al., "The SPACE of Developer Productivity", ACM Queue 19(1), 2021 | https://queue.acm.org/doi/fullHtml/10.1145/3454122.3454124 | [P-copy]. queue.acm.org is Cloudflare-blocked; read through the search cache ("at least three [dimensions] are recommended") |
| Noda, Storey, Forsgren, Greiler, "DevEx: What Actually Drives Productivity", ACM Queue 2023 / CACM | https://cacm.acm.org/practice/devex-what-actually-drives-productivity/ | [P-copy] via search cache: the three dimensions and the cognitive-load definition. The brief names Zimmermann as an author; the published author list is Noda, Storey, Forsgren and Greiler |
| Skelton & Pais, Thinnest Viable Platform | https://teamtopologies.com/key-concepts-content/what-is-a-thinnest-viable-platform-tvp | [P] (authors' site). The book itself was not read [N] |
| CNCF TAG App Delivery, Platforms white paper and Platform Engineering Maturity Model v1 | https://github.com/cncf/tag-app-delivery (`platforms-whitepaper/latest/index.md`, `platforms-maturity-model/v1/index.md`, commit `cadc7760`, 2025-04-16) | [P] cloned: platform attributes, success measures, maturity aspects |
| Parnas, "On the Criteria To Be Used in Decomposing Systems into Modules", CACM 15(12), 1972 | http://sunnyday.mit.edu/16.355/parnas-criteria.html | [P-copy] (MIT-hosted reprint) |
| Conway, "How Do Committees Invent?", Datamation, April 1968 | https://www.melconway.com/Home/Committees_Paper.html | [P] (author's site) |
| Lehman, "Programs, Life Cycles, and Laws of Software Evolution", Proc. IEEE 68(9), 1980 | https://users.ece.utexas.edu/~perry/education/SE-Intro/lehman.pdf | [P-copy] (university-hosted scan): Laws I and II |
| Ousterhout, *A Philosophy of Software Design* (2018) | — | [P-copy] read via a third-party PDF (§2.1–2.2, §4.4). Cite the book |
| Winters, Manshreck, Wright, *Software Engineering at Google*, ch. 1 | https://abseil.io/resources/swe-book/html/ch01.html | [P] (publisher-sanctioned free edition): "programming integrated over time", Hyrum's law, the Churn Rule, the Beyoncé Rule |
| Dean & Barroso, "The Tail at Scale", CACM 56(2), 2013 | https://www.barroso.org/publications/TheTailAtScale.pdf | [P], PDF text extracted |
| Gregg, The USE Method | https://www.brendangregg.com/usemethod.html | [P] |
| OpenSSF Scorecard API | https://api.securityscorecards.dev/projects/github.com/backstage/backstage | [P] JSON, 2026-11-23 |
| SLSA v1.2 (Build track L0–L3; Source track new) | https://slsa.dev/spec/v1.2/build-track-basics ; https://slsa.dev/blog/2025/11/announce-slsa-v1.2 | [P]. v1.1 is marked "Retired" |
| NIST SP 800-218 SSDF v1.1 (final, Feb 2022) | https://csrc.nist.gov/pubs/sp/800/218/final ; PDF https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-218.pdf | [P]: PW.9, PS.2, PS.3 and PW.4 text extracted |
| NIST SP 800-218 Rev. 1 (SSDF 1.2, initial public draft, 2025-12-17) | https://csrc.nist.gov/pubs/sp/800/218/r1/ipd | [P] landing page. The PS.4 contents (rollback) are [S], from a secondary summary |
| CISA Secure by Design | https://www.cisa.gov/securebydesign | [P] landing page ("Out-of-the-box, products should be secure…") |
| OWASP ASVS 5.0.0 (May 2025), V13 Configuration | https://github.com/OWASP/ASVS/blob/v5.0.0/5.0/en/0x22-V13-Configuration.md | [P] |
| Backstage repository (docs, templates, code) | https://github.com/backstage/backstage at `v1.57.1` / `b3ed9690` | [P], shallow clone. Files are listed in §2 |
| Backstage Upgrade Helper | https://github.com/backstage/upgrade-helper | [P] repository metadata only |
| npm registry metadata for `@backstage/*` | https://registry.npmjs.org/@backstage/create-app | [P] |
| Spotify engineering blog | — | Not used: no post needed for a claim here |
| Seikika context | `docs/technology-critique.md` §4, §7; `docs/research.md` §2; `docs/red-team.md`; `docs/adversarial-critique.md` §4; `npm/site/src/content/docs/compare/backstage.md`; `experiments/deploy-ab/README.md` | [P] repo |

**Threats to validity.**
- Both products are the maintainers' configurations, so expertise in each must be comparable.
  Publish all configs and invite a Backstage maintainer to review the Backstage arm before the run.
- F1, F9 and F11 depend on scripted customisation sets. Fix and publish them before the run.
- Single-VM numbers say nothing about multi-replica behaviour.
- Corpus shape (fan-in skew) changes F4 and F5. Publish the generator and its seed.
