# Adjudication: Toolbox against Backstage, beyond DX

*2026-11-23. Written by an agent acting as the independent adjudicator (dx-benchmarks.md §4.1, P7).
I designed no part of this benchmark. I created this file and changed nothing else in the
repository: no commits, no edits, and every probe below wrote only under `/tmp`. Inputs: the
pre-registration as frozen at `dfeeea4`, [RESEARCH.md](./RESEARCH.md),
[RESULTS.md](./RESULTS.md), the raw results in [results/r1](./results/r1) and
[results/r2](./results/r2), the scorecards in `portal/standards/backstage-comparison.yaml`, the
records in `portal/comparisons/`, [facts.yaml](./facts.yaml),
D01M857ZB468ZM2YS6GNSEG6YB9, and every product commit in `main..HEAD`
(HEAD = `22bdd5f`).*

## Conclusion

**I uphold the headline. After round 2, Toolbox is at least at parity with Backstage on every
pre-registered factor except capabilities, where it is behind. After round 1 it was also behind on
performance.** The numbers in RESULTS.md recompute exactly from the raw JSON, and the between-rounds
fix (`916f596`) is general, correct and fit under P5. But the write-up overstates its
pre-registration in several places, and some facts are scored in Toolbox's favour:

- the scorecards were never frozen, and they are still not committed;
- `facts.yaml` was written after the freeze, and it decides 24 of the 48 rules;
- three facts are scored in Toolbox's favour against the brief's own protocol;
- round 1's performance tally is miscounted.

None of these moves any factor to *behind*. Every correction below turns a rule from *ahead* into
*parity*, and every factor except capabilities keeps at least one well-evidenced rule ahead.

| Factor (README §Measures) | Round 2 (`916f596`) | Round 1 (`5ef6263`) | Robust to the corrections below? |
|---|---|---|---|
| Performance | **ahead** (7 ahead / 0 parity / 0 behind) | **behind** (5 / 0 / 2: `listP95Ms` and `listRps`) | Yes. It also holds on a spread-out workload that I ran (§4.2) |
| Footprint | **ahead** (5 / 0 / 0) | **ahead** (5 / 0 / 0) | Yes |
| Security | **ahead** (5 / 3 / 0 as scored) | **ahead** (same) | Yes, but at 2 / 6 / 0 once loopback, credential isolation and the delete probe are judged like for like (§5) |
| Supply chain | **ahead** (4 / 0 / 0) | **ahead** | Yes, with the omitted Toolbox components counted (§4.3) |
| Change safety | **ahead** (4 / 0 / 0) | **ahead** | Yes, though `upgrade-lines` compares unlike measures (§4.3) |
| Org fit | **ahead** (1 / 1 / 0) | **ahead** | Yes |
| Agent surface | **ahead** (2 / 1 / 0) | **ahead** | Yes |
| Capabilities, with extension cost as the README groups it | **behind** (3 / 5 / 7) | **behind** | Behind either way. Corrected: 2 / 6 / 7 |
| *Extensibility, a scorecard the README does not have* | *ahead (2 / 0 / 0)* | *ahead* | *Not a pre-registered factor (§3.2)* |

The claim "Toolbox is at least at parity on every factor except capabilities" is **supported** for
round 2. The stronger claim in RESULTS.md, "ahead on eight of the nine factors", also holds under
D01M857ZB468ZM2YS6GNSEG6YB9's semantics. Two caveats apply to it: the README pre-registered eight
factors, not nine, and several *ahead* rules rest on unfrozen judgements (§3, §5).

## 1. Recomputation

I recomputed min, median and max for each fact, size and arm from
`results/r{1,2}/perf-<arm>-<size>-<rep>.json`. A read whose URL contains `?` is the filtered
list; the other read is get-by-id.

- **All 36 rows of RESULTS.md's round 1 and round 2 tables match exactly**, after stripping the
  bold markup (`diff` of the generated rows against RESULTS.md: identical).
- Every read in all 40 perf files returned only `200`, with `ok = 1.0`. The only errors are oha's
  15–16 `aborted due to deadline` at the end of each 10 s window.
- No record is duplicated across files or rounds. One value coincides: Backstage's
  `perf-backstage-10000-1` get rate is 3547.4 in both rounds. The records differ (35,462 against
  35,463 responses, and different percentiles), so I accept it as chance.
- `portal/comparisons/{toolbox,backstage}.yaml` equal the round-2 10k medians and the r2
  surface, footprint and upgrade JSON, field for field.
- The probes and static facts are byte-identical between r1 and r2, as RESULTS.md:103 says.
- Timeline, from the raw files in `/tmp/bc`:
  - freeze `dfeeea4` at 22:56:23;
  - round 1 from about 22:56:30 to 23:08:36;
  - `916f596` committed at 23:14:36, and `target/release/toolbox` built at 23:14;
  - round 2 from about 23:18:25 to 23:30:28.

  The fix came after round 1's results and before round 2 started.
- The live Compare view (`toolbox report portal --skip runner,github-actions -o json`, written to
  `/tmp`) gives the same per-rule standings as RESULTS.md's round-2 table. Note that
  `--fixtures` reads `portal/fixtures/workspace.ndjson`, not `portal/comparisons/`, and shows
  every rule *ahead*. Don't use it to check this comparison.

**Mismatch.** RESULTS.md:17 gives round 1's performance as "6 / 0 / 1". Both list rules were
behind: p95 118.05 ms against 21.09 ms, and 197.6/s against 1,207.2/s. The tally is **5 / 0 / 2**.
The standing, behind, is unaffected.

## 2. Verdicts under the pre-registered rule

The rule: a value rule is ahead or behind when the medians differ by more than 10% of the larger,
otherwise parity. A boolean rule goes to the passing side. A factor is behind if any rule is
behind, and a missing value is unknown. All values are present. Toolbox's value comes first in each
pair.

| Factor | Rules, round 2 (Toolbox vs Backstage) | Round 1 difference |
|---|---|---|
| Performance | All seven ahead: ready 0.92 s vs 3.45 s, readable 2.18 s vs 27.2 s, get p95 0.45 ms vs 6.47 ms, get 58,576/s vs 3,687/s, list p95 6.01 ms vs 17.68 ms, list 3,900/s vs 1,279/s, recovery 1.13 s vs 2.31 s | list p95 118.05 vs 21.09 behind; list 197.6 vs 1,207.2 behind |
| Footprint | All five ahead: 89.7 vs 452.9 MB idle, 175.4 vs 829.2 MB loaded, 121.1 vs 709.9 MB installed, 2 vs 44 adopter files, 230 vs 1,357 lines | none |
| Security | Parity: anonymous read, forged identity, anonymous MCP (all refused). Ahead: delete probe, loopback, owner-only fields, process isolation, credential isolation | none |
| Supply chain | All four ahead: 388 vs 2,791 packages, 2 vs 34 known vulnerabilities, 0 vs 16 high, 100% vs 0% provenance | none |
| Change safety | All four ahead: offline validate, change preview, 0 vs 47.3 upgrade lines, 0 vs 1.5 rollback steps | none |
| Org fit | Repo-scoped ahead; federation parity (neither) | none |
| Agent surface | MCP by default parity; 6,473 vs 20,500 B for `tools/list`; 253 vs 580 B to get one entity | none |
| Capabilities (README grouping) | 12 scenarios, extension cost (2 rules) and contributors. Ahead: scorecards, any language, without rebuild. Parity: browse, ownership, graph, permission rule, MCP read. Behind: docs search, docs, templates, API docs, Kubernetes, notifications, community (4 vs 2,826) | none |

At 1k (published only), round 2 is ahead on every performance rule, and so was round 1. Round 1's
list loss appears only at 10k.

## 3. Bar audit

### 3.1 The bars the README set

Every bar in the frozen README is implemented as written in
`portal/standards/backstage-comparison.yaml`:

- ready ≤ 10 s, readable ≤ 60 s, get p95 ≤ 50 ms, get ≥ 1,000/s, list p95 ≤ 500 ms, list ≥ 20/s and
  recovery ≤ 60 s;
- 512 MB idle, 1,024 MB loaded, 1,000 MB installed;
- each security boolean true;
- high vulnerabilities == 0 and provenance == 100;
- rollback == 0, with offline validate and change preview true;
- repo-scoped and federation true;
- MCP by default true, `tools/list` ≤ 32,768 B and get-entity ≤ 4,096 B;
- each capability true.

Every value rule has `tolerance: 0.1`. `rule_standing` (crates/toolbox-derive/src/lib.rs:1227–1262)
computes `|a − b| ≤ 0.1 × max(|a|, |b|)` as parity, as the README says.

### 3.2 What was not in the frozen README

| Item | In RESULTS.md's Deviations? | Changes a standing? |
|---|---|---|
| Bars for the README's "—" rows: adopter files ≤ 10, adopter lines ≤ 500, upgrade lines ≤ 10, packages ≤ 1,000, `knownVulns == 0`, contributors ≥ 10 | Yes, and the list is complete | No. Both sides have values, so values decide (lib.rs:1240–1251) |
| **The scorecard file itself.** `portal/standards/backstage-comparison.yaml`, `portal/comparisons/` and `docs/design/backstage-comparison.md` are not in `dfeeea4`, although its README links to two of them. They are **still untracked at HEAD**: `git status` shows `??`. The rules, their `better` directions and their grouping into scorecards were never frozen | **No** | Not by itself: I re-derived every standing by hand from the README's table (§2) |
| **A ninth factor.** The README's capabilities row includes "extension cost". The scorecards move it into `vs-backstage-extensibility`, which turns two Toolbox wins into a separate factor | **No** | No: capabilities is behind either way |
| **`facts.yaml` decides 24 of 48 rules**: 4 in security, 2 in change safety, 2 in org fit, 1 in agent surface, 2 in extensibility and 13 in capabilities. It was written after the freeze (RESULTS.md:142). RESULTS.md:143 says it "measures nothing", which understates its role: these are post-freeze judgements, and three of them favour Toolbox (§5) | Partly | Not to *behind* (§5) |
| **Undisclosed pre-freeze product changes.** README:64–75 lists four smoke-round defects and says "the security probes … needed no change". In fact `5ef6263` is a fifth performance change. `3cf4dba` and `e06d6a7` changed the security defaults *because of this comparison*: D01M857ZB3FZJZWX8G3BF5W1RNR §1 says "The Backstage comparison … found the shipped image worse than Backstage's production template". Without `3cf4dba`, surface.py's Toolbox arm, which binds `0.0.0.0` with no mode set, would have trusted `x-toolbox-user` and failed the anonymous-read and forged-identity probes | **No** | They decided the standings (security would have been *behind*), but they are pre-freeze, general and named outside uses, so they are legitimate. The fault is the disclosure, not the changes |
| `measure.py:165` starts Backstage with `app-config.yaml` + the bench overlay only. `surface.py:99–100` adds `app-config.production.yaml`, and README:20 says "the template's configs" | No | Unlikely: the bench overlay replaces the database and catalog. Not tested |
| D01M857ZB468ZM2YS6GNSEG6YB9 §2 vs README: when one side lacks a value on a value rule, `rule_standing` falls back to pass/fail outcomes (lib.rs:1252–1261). The README says a missing value is *unknown* | No | No: snapshot.py exits if any value is missing, and none is |

## 4. Fit audit (P5) and threats

### 4.1 Product commits in `main..HEAD`

`git log main..HEAD -- crates/` lists 16 commits. Four of them belong to other benchmarks
(gate-repair-ab and pin-repair-ab), touch `crates/toolbox-cli/src/causes.rs` and
`toolbox-connector-workspace/src/workflows.rs`, and nothing on any read path measured here.

| Commit | Files | Harness touched? | Outside use named | Timing | Verdict |
|---|---|---|---|---|---|
| `76a1140`, `c1f9b22`, `8a76f24`, `9bac4f3` | causes.rs, report.rs, workflows.rs, packages, portal results | No | gate and pin hints (other benchmarks) | Before this benchmark | Unrelated |
| `3cf4dba` | authz/authn.rs, cli/server.rs, a decision, v1-spec | No | "Every deployment of the image, and every `serve --listen 0.0.0.0`" | Pre-freeze (22:21) | **Fit**, general (default auth by bind address). Found by this comparison, so it should be disclosed in the README |
| `e06d6a7` | api/connectors.rs, connector-control.md | No | Every in-image connector | Pre-freeze (22:21) | **Fit**. It is what makes `extensionCredentialIsolated` true, but see §5 |
| `05c947e` | derive, model, schema, report, report JS, a decision | No | Any comparative scorecard | Pre-freeze (22:27) | **Fit**. This is the scorer, written by the designer. My hand re-derivation agrees with it on every rule |
| `9bf06d7` | api/routes.rs, tests | No | Any filtered list with authz | Pre-freeze | **Fit**, general (README bullet 1) |
| `41d1564`, then reverted by `3b3ff8b` | authz/openfga.rs | No | — | Pre-freeze | Net zero (README bullet 4) |
| `a62d92f` | api/access.rs, state.rs, tests, authorization.md | No | Every checked read | Pre-freeze | **Fit**, general: decisions are kept per snapshot and stay exact. But it turns the harness's single repeated URL into a cache hit (§4.2), which README:93 misdescribes |
| `127b08e` | authz/openfga.rs | No | Every OpenFGA check | Pre-freeze | **Fit**, general (README bullet 2) |
| `6c5eb4a` | api/routes.rs | No | Any paged list | Pre-freeze | **Fit**, general (README bullet 3) |
| `5ef6263` | api/access.rs, routes.rs | No | Any kind without owner-only fields | Pre-freeze (22:50) | **Fit** and correct: `facet_needs` (access.rs:219–225) yields only `admin`, which is shape-independent, or `can_view_details`, which `hides_details` tests. **Not disclosed** in the README |
| **`916f596`** | api/routes.rs, api/state.rs, api/tests/api.rs, expr/analysis.rs | **No** | MCP `query`, `toolbox query` (main.rs:567 → mcp.rs:292–299 → `/v1/entities`) and the UI catalog filter (ui/src/main.tsx:176). **"Owner and team pages" is wrong**: the team view reads `/v1/views/home` (ui/src/main.tsx:89, reads.rs:509), not this list | **Between rounds** (23:14, after round 1 ended at 23:08) | **Fit, general, correct for every case the harness or any shipped portal can produce** (below) |
| `4713309` | api, authz, cli: extraction and doc comments | No | — | After round 2 (23:47); not measured | Behaviour-preserving, by reading the diff |

**`916f596` in detail.**

- **General, not shaped to the harness.** `required_equalities` (expr/analysis.rs) takes every
  top-level `&&` conjunct of the form `entity.<f> == '<string>'`, either way round.
  `list_entities` uses it for any kind named in `kind=` and any field that kind *declares*, and
  starts from the smallest posting list. Nothing names `owner`, `Component`, `group:` or the
  corpus. The unit test uses the benchmark's literal `'group:g3'`, which is cosmetic. It supports
  only the simplest equality shape, which happens to be the benchmark's query. That is narrow but
  common, and `||`, `in`, numbers and nested selects fall back to the full scan.
- **Correctness argument.**
  - The filter sees `visible_fields`. That is `e.fields` copied verbatim, or replaced by a
    `redacted` object (access.rs:202–214, 256–286). The index reads the same `e.fields` values
    with `as_str`. A visible field therefore matches iff it is in the posting, and a redacted
    field is an object, which the full filter rejects anyway.
  - Undeclared fields, `id` and `kind` are never indexed.
  - The cursor works because postings are built by iterating a `BTreeMap`, so they are in id
    order, and `partition_point(id <= cursor)` keeps them sorted.
  - The index is per `Published` snapshot, so it is never stale.
  - The kind filter still runs case-insensitively after narrowing.
- **Black-box check on the measured binary** (built 23:14). I paged through the narrowed filter
  and its un-narrowed twin (`… || false`) with limits 1, 7 and 100, for five cases:
  `kind=Component` and `kind=component`, swapped operands with a conjunction, an empty value, and
  `kind=System`. **15/15 page sequences were identical.**
- **Hidden facets.** `a_filter_cannot_probe_a_hidden_facet` (tests/authz.rs:248) filters
  `kind=User` on `entity.email == …`, which is exactly the narrowed path. Results cannot leak.
  Latency can differ by whether a hidden value exists, but `9bf06d7` already had that timing
  channel.
- **Two edge-case bugs that no shipped portal and no harness can hit:**
  1. The `kindName` pattern allows kinds that differ only in case (`Api` and `API`), and
     validate.rs:148's duplicate check is exact. `find(eq_ignore_ascii_case)` then narrows to the
     first such kind only, while the unindexed path returns both.
  2. The `fieldName` pattern allows a declared field named `id`. The filter sees `visible["id"]` =
     the entity id (routes.rs `matches`), but the index uses the field's value.

  Fix both in the product. Neither affects these results.

### 4.2 Threats to validity the README does not list, with what I measured

All probes ran on this VM after the rounds, through `measure.py`'s own start and load code. They
are not part of the pre-registered measure.

| Threat | Evidence | Effect on a standing |
|---|---|---|
| **Request logging is asymmetric.** Toolbox runs with `RUST_LOG=warn` (measure.py:113). Backstage runs at its default `info`: the template's `rootHttpRouter` mounts `middleware.logging()`, one JSON line per request (backend-defaults `rootHttpRouterServiceFactory.cjs.js:117`). The round logs bear this out: `/tmp/bc/r/perf-backstage-10000-1.log` has 47,994 `incomingRequest` lines, and the Toolbox logs are empty | `LOG_LEVEL=warn`, 2 runs each, alternating, 10k: get 3,597 and 3,899/s (default) vs 4,265 and 4,221/s (warn), about +13%; get p95 6.67 and 6.09 vs 5.64 and 5.75 ms. List was unchanged (1,323 and 1,335 vs 1,326 and 1,376/s). My default runs reproduce round 2's Backstage medians | None: Toolbox still has 58,576/s and 3,900/s |
| **One repeated URL per read** (`component:c5000` and owner `g3`). Combined with `a62d92f`'s per-snapshot decisions, Toolbox's reads are authorization-cache hits after the first. README:93 says "Toolbox checks every read in OpenFGA", which is wrong in steady state: neither arm consults an authorization service per read. `a62d92f`'s own note records about 1,300 uncached checked reads/s at 1k (before `127b08e`) | Reads spread uniformly over all 10,000 ids and all 50 owners, 2 runs each: **Toolbox get 36,260 and 37,221/s (p95 1.08 and 1.06 ms), list 3,662 and 3,648/s (p95 6.86 and 6.82 ms). Backstage get 3,719 and 3,679/s (p95 6.38 and 6.43 ms), list 1,306 and 1,333/s (p95 19.3 and 18.1 ms)** | None. Spreading the reads costs Toolbox about 37% of its get throughput, and it is still 10× ahead. Not tested: many users, and snapshot churn (each publish empties the decision cache) |
| Load generator on the same 4 vCPU, with no `taskset` (the brief's §1.1 requires it); a closed loop at 16 connections for 10 s, with no warm-up, against the brief's open-loop 50–500 RPS, 60 s warm-up, 300 s measured and p99 | measure.py:23–24, 84–100 | Unknown in size. oha competes hardest at Toolbox's rates, so this probably *understates* Toolbox. p95 from a closed loop hides coordinated omission on both sides |
| One Node process (single-threaded JavaScript) against multi-threaded Rust on 4 vCPU. Backstage's documented scaling is identical replicas sharing the database (RESEARCH.md §2). CPU per request was not recorded | — | Pre-registered (one replica). Three or four Backstage replicas might close the 2.8–3× list lead, moving list to parity, not behind. Not tested |
| The brief's equivalence gate (same count and SHA over (ref, owner, relations)) is not implemented, and response sizes are not recorded. "Loaded" means `totalItems ≥ n` for Backstage but `component:c{n-1}` readable for Toolbox | measure.py:133–134, 174–177 | None expected: g3 owns 200 components at 10k, so both pages hold 100. Unverified per response |
| Allow-all, single replica, and the capability scenarios' origin | README:88–98 | Already listed |

### 4.3 Measures that compare unlike things

- **Supply chain counts only `Cargo.lock` for Toolbox** (footprint.py:81–85, 124). The install it
  sizes also carries the embedded UI (`ui/package-lock.json`: 75 packages, 0 known
  vulnerabilities by osv-scanner 2.2.4) and the OpenFGA binary. OpenFGA has 144 Go packages, and
  osv-scanner's `go/binary` plugin finds 3 unrated advisories: GO-2026-6859 in stdlib, and
  GO-2026-6860 and GO-2026-6866 in `golang.org/x/net`. Counted, Toolbox has about 607 packages
  and 5 known vulnerabilities (0 rated high), against 2,791 and 34 (16 high). Still ahead on all
  four rules.

  Toolbox's packages also come from HEAD's `Cargo.lock` (388), while its install facts are from
  v0.1.1, whose lock has 371.
- **Upgrade lines are not like for like.** Toolbox's 0 is "the v0.1.0 starter still validates and
  tests" (upgrade.py:69–77), which is lines *required*. Backstage's 47.3 is the create-app
  template diff per release, which is lines the Upgrade Helper *suggests*. The brief's F9
  prescribes `versions:bump` plus hand edits to green for both. The rule is plausibly still ahead
  (145 `**BREAKING**` entries over 12 releases), but this measure does not show it.

  The label is also wrong: RESULTS.md:119 and upgrade.py:132 say `v0.1.1..HEAD`, but the starter
  is taken from `v0.1.0` (upgrade.py:31, 69).
- **Adopter files compare different portals.** Toolbox's 2 files and 230 lines are
  `examples/repo-portal-starter`, a repository-scored portal for `toolbox report`, not a served
  catalog. Backstage's are the whole create-app tree, frontend included. Toolbox's served bench
  portal (`portal/toolbox.yaml` plus `model.fga`, 2 files and 81 lines) would give the same
  standing.
- **Rollback.** Toolbox's rollback is executed: old → new → old on one database. Backstage's is
  counted from documented migrations, not run. That is consistent with RESEARCH.md §2's
  `manual-knex-rollback.md` quote.

## 5. P4: friction left in the Backstage arm, and facts scored in Toolbox's favour

**Backstage arm (P4).** The configuration is otherwise what a competent operator would run:
production mode, the production bundle built as the Dockerfile does, Postgres 16, and the
template's plugins. I verified that `kubernetes-backend`, `notifications-backend`, `search-*`,
`techdocs-backend` and `mcp-actions-backend` are in the generated
`packages/backend/package.json`. Two frictions remain:

1. **Logging.** See §4.2. It is measured and changes no standing. It should still be fixed:
   either both arms at their defaults or both at `warn`.
2. **The client token can delete.** `app-config.bench.yaml` issues a static `externalAccess`
   token with no restriction, so `clientCannotDelete` is false for Backstage. That reflects the
   harness's choice, not the product. A competent operator would give a read-only load client
   a restricted token. To my knowledge, Backstage's service-to-service auth supports
   `accessRestrictions` per `externalAccess` entry. *I did not re-read that doc in this session;
   treat it as unverified.* If so, the rule moves to parity. Toolbox passes by having no DELETE
   route at all (405).

**`facts.yaml`, Toolbox side.** I read every cited file, and every citation exists:

- `listen_addr`, `facets_actions_and_revocation`, `a_filter_cannot_probe_a_hidden_facet`,
  `connectors_inherit_no_server_credentials`, and `env_clear()` at connectors.rs:303–305;
- `plan --local`, `toolbox report` and `/mcp`;
- 4 contributors (GitHub API, `anon=1`, last page 4).

Two Toolbox claims are overstated:

- **`loopbackByDefault: true`** (facts.yaml:6–11) holds for the bare binary (server.rs:5–16). It
  is false for the shipped image: `image/Dockerfile:24` sets `ENV PORT=8080`, so `listen_addr`
  binds `0.0.0.0`. D01M857ZB3FZJZWX8G3BF5W1RNR §1 says so itself. Backstage is scored on its
  production template, and the brief's F6 says to probe "the template's production config and
  image" for both. Judged like for like, both are false, so this rule is **parity**. The basis
  favours Toolbox.
- **`extensionCredentialIsolated: true`** (facts.yaml:23–29) holds against *accidental*
  inheritance only. Connectors are spawned as the server's UID, with no UID change and no
  `PR_SET_DUMPABLE` anywhere in `crates/`. A same-UID process can read its parent's
  `/proc/<pid>/environ`, which still holds `DATABASE_URL` and the PATs. I checked that this
  kernel allows such a read between same-UID siblings. Off token mode, on loopback, a connector
  can also call the API with `x-toolbox-user`. The cited test checks only the allowlist
  constant. Against the brief's malicious canary (F7 attempts 1 and 3), Toolbox is not isolated
  either: **parity**. `extensionProcessIsolated` stays a fair *ahead*.

**`facts.yaml`, Backstage side.** The facts are consistent with RESEARCH.md §2:

- loopback, sandboxing, credentials, repo scope, federation;
- `catalog validate --instance`;
- provenance: I re-checked npm, and no version of `create-app`, `backend-defaults` or
  `plugin-catalog-backend` carries `dist.attestations`;
- 2,826 contributors (GitHub API).

One claim is not:

- **`capScorecards: false`** (facts.yaml:161–165) contradicts the brief. RESEARCH.md §1.2 F1 says
  "Backstage may use community plugins (for example Tech Insights for scorecards). Record each one
  as an add-on". facts.yaml's header changed the definition to "default install or first-party
  package" after the freeze. Under the brief the rule is **parity**.

`changePreview: false` and `extensionAnyLanguage: false` are absence claims. They are plausible
but weakly evidenced: URL locations let any-language services feed the catalog. Only
`capPermissionRule` is scored in Backstage's favour, as RESULTS.md says. That makes
RESULTS.md:131–133 ("Where the definition was ambiguous, they are scored in Backstage's favour")
**inaccurate as a general statement**.

**After these corrections:**

- Security: 2 ahead (owner-only fields, process isolation) and 6 parity, so **ahead**.
- Capabilities: 2 / 6 / 7 in the README grouping, so **behind**.
- Nothing else changes.

## 6. What to fix in the write-up

1. RESULTS.md:17: round 1's performance is **5 / 0 / 2**, not 6 / 0 / 1.
2. RESULTS.md:10: the README pre-registered **eight** factors, with extension cost inside
   capabilities. Report 7 of 8 ahead and capabilities behind, and list the extensibility split as
   a deviation.
3. RESULTS.md:71 (and `916f596`'s message): drop "owner and team pages". The outside uses are the
   MCP `query` tool, `toolbox query` and the UI catalog filter.
4. RESULTS.md:131–133: say that `loopbackByDefault`, `extensionCredentialIsolated` and
   `capScorecards` are scored on bases that favour Toolbox (§5), or rescore them.
5. Deviations (RESULTS.md:135–147):
   - the scorecards, product records and design doc were never frozen and are not committed;
     commit them;
   - `facts.yaml` decides 24 of 48 rules;
   - the extensibility split;
   - `measure.py` omits `app-config.production.yaml`;
   - the derive fallback to outcomes when a value is missing.
6. README:64–75 (or a RESULTS.md note): disclose `5ef6263`, `3cf4dba`, `e06d6a7` and `05c947e` as
   pre-freeze changes made for this comparison. "The security probes … needed no change" is
   contradicted by D01M857ZB3FZJZWX8G3BF5W1RNR.
7. Add the §4.2 and §4.3 threats:
   - the logging asymmetry;
   - the single-URL cache hits, and that README:93 is wrong in steady state;
   - oha on the same CPUs without `taskset`;
   - 10 s closed-loop against the brief's 300 s open-loop;
   - `Cargo.lock`-only supply chain, with OpenFGA and the UI omitted;
   - the unlike upgrade measure and its `v0.1.0`/`v0.1.1` label;
   - the unrestricted static token.
8. Before the next round: set both arms' log level the same; restrict the Backstage client token;
   spread reads over ids and owners; implement the brief's equivalence gate; and run the Toolbox
   image, not the bare binary, for the security probes.
9. Product, outside the write-up:
   - fix `916f596`'s two edge cases (kinds that differ only in case; a declared `id` field);
   - make `rule_standing` return `unknown` when one side lacks a value on a value rule;
   - move the orphaned doc comment at toolbox-derive/src/lib.rs:1221–1223 back to
     `level_ignoring_unknown`;
   - consider running connectors under their own UID, or non-dumpable, before claiming credential
     isolation.
