# Adjudication: Seikika against Backstage, beyond DX

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

**I uphold the headline. After round 2, Seikika is at least at parity with Backstage on every
pre-registered factor except capabilities, where it is behind. After round 1 it was also behind on
performance.** The numbers in RESULTS.md recompute exactly from the raw JSON, and the between-rounds
fix (`916f596`) is general, correct and fit under P5. But the write-up overstates its
pre-registration in several places, and some facts are scored in Seikika's favour:

- the scorecards were never frozen, and they are still not committed;
- `facts.yaml` was written after the freeze, and it decides 24 of the 48 rules;
- three facts are scored in Seikika's favour against the brief's own protocol;
- round 1's performance tally is miscounted.

None of these moves any factor to *behind*. Every correction below turns a rule from *ahead* into
*parity*, and every factor except capabilities keeps at least one well-evidenced rule ahead.

| Factor (README §Measures) | Round 2 (`916f596`) | Round 1 (`5ef6263`) | Robust to the corrections below? |
|---|---|---|---|
| Performance | **ahead** (7 ahead / 0 parity / 0 behind) | **behind** (5 / 0 / 2: `listP95Ms` and `listRps`) | Yes. It also holds on a spread-out workload that I ran (§4.2) |
| Footprint | **ahead** (5 / 0 / 0) | **ahead** (5 / 0 / 0) | Yes |
| Security | **ahead** (5 / 3 / 0 as scored) | **ahead** (same) | Yes, but at 2 / 6 / 0 once loopback, credential isolation and the delete probe are judged like for like (§5) |
| Supply chain | **ahead** (4 / 0 / 0) | **ahead** | Yes, with the omitted Seikika components counted (§4.3) |
| Change safety | **ahead** (4 / 0 / 0) | **ahead** | Yes, though `upgrade-lines` compares unlike measures (§4.3) |
| Org fit | **ahead** (1 / 1 / 0) | **ahead** | Yes |
| Agent surface | **ahead** (2 / 1 / 0) | **ahead** | Yes |
| Capabilities, with extension cost as the README groups it | **behind** (3 / 5 / 7) | **behind** | Behind either way. Corrected: 2 / 6 / 7 |
| *Extensibility, a scorecard the README does not have* | *ahead (2 / 0 / 0)* | *ahead* | *Not a pre-registered factor (§3.2)* |

The claim "Seikika is at least at parity on every factor except capabilities" is **supported** for
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
- `portal/comparisons/{seikika,backstage}.yaml` equal the round-2 10k medians and the r2
  surface, footprint and upgrade JSON, field for field.
- The probes and static facts are byte-identical between r1 and r2, as RESULTS.md:103 says.
- Timeline, from the raw files in `/tmp/bc`:
  - freeze `dfeeea4` at 22:56:23;
  - round 1 from about 22:56:30 to 23:08:36;
  - `916f596` committed at 23:14:36, and `target/release/seikika` built at 23:14;
  - round 2 from about 23:18:25 to 23:30:28.

  The fix came after round 1's results and before round 2 started.
- The live Compare view (`seikika report portal --skip runner,github-actions -o json`, written to
  `/tmp`) gives the same per-rule standings as RESULTS.md's round-2 table. Note that
  `--fixtures` reads `portal/fixtures/workspace.ndjson`, not `portal/comparisons/`, and shows
  every rule *ahead*. Don't use it to check this comparison.

**Mismatch.** RESULTS.md:17 gives round 1's performance as "6 / 0 / 1". Both list rules were
behind: p95 118.05 ms against 21.09 ms, and 197.6/s against 1,207.2/s. The tally is **5 / 0 / 2**.
The standing, behind, is unaffected.

## 2. Verdicts under the pre-registered rule

The rule: a value rule is ahead or behind when the medians differ by more than 10% of the larger,
otherwise parity. A boolean rule goes to the passing side. A factor is behind if any rule is
behind, and a missing value is unknown. All values are present. Seikika's value comes first in each
pair.

| Factor | Rules, round 2 (Seikika vs Backstage) | Round 1 difference |
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

Every value rule has `tolerance: 0.1`. `rule_standing` (crates/seikika-derive/src/lib.rs:1227–1262)
computes `|a − b| ≤ 0.1 × max(|a|, |b|)` as parity, as the README says.

### 3.2 What was not in the frozen README

| Item | In RESULTS.md's Deviations? | Changes a standing? |
|---|---|---|
| Bars for the README's "—" rows: adopter files ≤ 10, adopter lines ≤ 500, upgrade lines ≤ 10, packages ≤ 1,000, `knownVulns == 0`, contributors ≥ 10 | Yes, and the list is complete | No. Both sides have values, so values decide (lib.rs:1240–1251) |
| **The scorecard file itself.** `portal/standards/backstage-comparison.yaml`, `portal/comparisons/` and `docs/design/backstage-comparison.md` are not in `dfeeea4`, although its README links to two of them. They are **still untracked at HEAD**: `git status` shows `??`. The rules, their `better` directions and their grouping into scorecards were never frozen | **No** | Not by itself: I re-derived every standing by hand from the README's table (§2) |
| **A ninth factor.** The README's capabilities row includes "extension cost". The scorecards move it into `vs-backstage-extensibility`, which turns two Seikika wins into a separate factor | **No** | No: capabilities is behind either way |
| **`facts.yaml` decides 24 of 48 rules**: 4 in security, 2 in change safety, 2 in org fit, 1 in agent surface, 2 in extensibility and 13 in capabilities. It was written after the freeze (RESULTS.md:142). RESULTS.md:143 says it "measures nothing", which understates its role: these are post-freeze judgements, and three of them favour Seikika (§5) | Partly | Not to *behind* (§5) |
| **Undisclosed pre-freeze product changes.** README:64–75 lists four smoke-round defects and says "the security probes … needed no change". In fact `5ef6263` is a fifth performance change. `3cf4dba` and `e06d6a7` changed the security defaults *because of this comparison*: D01M857ZB3FZJZWX8G3BF5W1RNR §1 says "The Backstage comparison … found the shipped image worse than Backstage's production template". Without `3cf4dba`, surface.py's Seikika arm, which binds `0.0.0.0` with no mode set, would have trusted `x-seikika-user` and failed the anonymous-read and forged-identity probes | **No** | They decided the standings (security would have been *behind*), but they are pre-freeze, general and named outside uses, so they are legitimate. The fault is the disclosure, not the changes |
| `measure.py:165` starts Backstage with `app-config.yaml` + the bench overlay only. `surface.py:99–100` adds `app-config.production.yaml`, and README:20 says "the template's configs" | No | Unlikely: the bench overlay replaces the database and catalog. Not tested |
| D01M857ZB468ZM2YS6GNSEG6YB9 §2 vs README: when one side lacks a value on a value rule, `rule_standing` falls back to pass/fail outcomes (lib.rs:1252–1261). The README says a missing value is *unknown* | No | No: snapshot.py exits if any value is missing, and none is |

## 4. Fit audit (P5) and threats

### 4.1 Product commits in `main..HEAD`

`git log main..HEAD -- crates/` lists 16 commits. Four of them belong to other benchmarks
(gate-repair-ab and pin-repair-ab), touch `crates/seikika-cli/src/causes.rs` and
`seikika-connector-workspace/src/workflows.rs`, and nothing on any read path measured here.

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
| **`916f596`** | api/routes.rs, api/state.rs, api/tests/api.rs, expr/analysis.rs | **No** | MCP `query`, `seikika query` (main.rs:567 → mcp.rs:292–299 → `/v1/entities`) and the UI catalog filter (ui/src/main.tsx:176). **"Owner and team pages" is wrong**: the team view reads `/v1/views/home` (ui/src/main.tsx:89, reads.rs:509), not this list | **Between rounds** (23:14, after round 1 ended at 23:08) | **Fit, general, correct for every case the harness or any shipped portal can produce** (below) |
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
| **Request logging is asymmetric.** Seikika runs with `RUST_LOG=warn` (measure.py:113). Backstage runs at its default `info`: the template's `rootHttpRouter` mounts `middleware.logging()`, one JSON line per request (backend-defaults `rootHttpRouterServiceFactory.cjs.js:117`). The round logs bear this out: `/tmp/bc/r/perf-backstage-10000-1.log` has 47,994 `incomingRequest` lines, and the Seikika logs are empty | `LOG_LEVEL=warn`, 2 runs each, alternating, 10k: get 3,597 and 3,899/s (default) vs 4,265 and 4,221/s (warn), about +13%; get p95 6.67 and 6.09 vs 5.64 and 5.75 ms. List was unchanged (1,323 and 1,335 vs 1,326 and 1,376/s). My default runs reproduce round 2's Backstage medians | None: Seikika still has 58,576/s and 3,900/s |
| **One repeated URL per read** (`component:c5000` and owner `g3`). Combined with `a62d92f`'s per-snapshot decisions, Seikika's reads are authorization-cache hits after the first. README:93 says "Seikika checks every read in OpenFGA", which is wrong in steady state: neither arm consults an authorization service per read. `a62d92f`'s own note records about 1,300 uncached checked reads/s at 1k (before `127b08e`) | Reads spread uniformly over all 10,000 ids and all 50 owners, 2 runs each: **Seikika get 36,260 and 37,221/s (p95 1.08 and 1.06 ms), list 3,662 and 3,648/s (p95 6.86 and 6.82 ms). Backstage get 3,719 and 3,679/s (p95 6.38 and 6.43 ms), list 1,306 and 1,333/s (p95 19.3 and 18.1 ms)** | None. Spreading the reads costs Seikika about 37% of its get throughput, and it is still 10× ahead. Not tested: many users, and snapshot churn (each publish empties the decision cache) |
| Load generator on the same 4 vCPU, with no `taskset` (the brief's §1.1 requires it); a closed loop at 16 connections for 10 s, with no warm-up, against the brief's open-loop 50–500 RPS, 60 s warm-up, 300 s measured and p99 | measure.py:23–24, 84–100 | Unknown in size. oha competes hardest at Seikika's rates, so this probably *understates* Seikika. p95 from a closed loop hides coordinated omission on both sides |
| One Node process (single-threaded JavaScript) against multi-threaded Rust on 4 vCPU. Backstage's documented scaling is identical replicas sharing the database (RESEARCH.md §2). CPU per request was not recorded | — | Pre-registered (one replica). Three or four Backstage replicas might close the 2.8–3× list lead, moving list to parity, not behind. Not tested |
| The brief's equivalence gate (same count and SHA over (ref, owner, relations)) is not implemented, and response sizes are not recorded. "Loaded" means `totalItems ≥ n` for Backstage but `component:c{n-1}` readable for Seikika | measure.py:133–134, 174–177 | None expected: g3 owns 200 components at 10k, so both pages hold 100. Unverified per response |
| Allow-all, single replica, and the capability scenarios' origin | README:88–98 | Already listed |

### 4.3 Measures that compare unlike things

- **Supply chain counts only `Cargo.lock` for Seikika** (footprint.py:81–85, 124). The install it
  sizes also carries the embedded UI (`ui/package-lock.json`: 75 packages, 0 known
  vulnerabilities by osv-scanner 2.2.4) and the OpenFGA binary. OpenFGA has 144 Go packages, and
  osv-scanner's `go/binary` plugin finds 3 unrated advisories: GO-2026-6859 in stdlib, and
  GO-2026-6860 and GO-2026-6866 in `golang.org/x/net`. Counted, Seikika has about 607 packages
  and 5 known vulnerabilities (0 rated high), against 2,791 and 34 (16 high). Still ahead on all
  four rules.

  Seikika's packages also come from HEAD's `Cargo.lock` (388), while its install facts are from
  v0.1.1, whose lock has 371.
- **Upgrade lines are not like for like.** Seikika's 0 is "the v0.1.0 starter still validates and
  tests" (upgrade.py:69–77), which is lines *required*. Backstage's 47.3 is the create-app
  template diff per release, which is lines the Upgrade Helper *suggests*. The brief's F9
  prescribes `versions:bump` plus hand edits to green for both. The rule is plausibly still ahead
  (145 `**BREAKING**` entries over 12 releases), but this measure does not show it.

  The label is also wrong: RESULTS.md:119 and upgrade.py:132 say `v0.1.1..HEAD`, but the starter
  is taken from `v0.1.0` (upgrade.py:31, 69).
- **Adopter files compare different portals.** Seikika's 2 files and 230 lines are
  `examples/repo-portal-starter`, a repository-scored portal for `seikika report`, not a served
  catalog. Backstage's are the whole create-app tree, frontend included. Seikika's served bench
  portal (`portal/seikika.yaml` plus `model.fga`, 2 files and 81 lines) would give the same
  standing.
- **Rollback.** Seikika's rollback is executed: old → new → old on one database. Backstage's is
  counted from documented migrations, not run. That is consistent with RESEARCH.md §2's
  `manual-knex-rollback.md` quote.

## 5. P4: friction left in the Backstage arm, and facts scored in Seikika's favour

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
   treat it as unverified.* If so, the rule moves to parity. Seikika passes by having no DELETE
   route at all (405).

**`facts.yaml`, Seikika side.** I read every cited file, and every citation exists:

- `listen_addr`, `facets_actions_and_revocation`, `a_filter_cannot_probe_a_hidden_facet`,
  `connectors_inherit_no_server_credentials`, and `env_clear()` at connectors.rs:303–305;
- `plan --local`, `seikika report` and `/mcp`;
- 4 contributors (GitHub API, `anon=1`, last page 4).

Two Seikika claims are overstated:

- **`loopbackByDefault: true`** (facts.yaml:6–11) holds for the bare binary (server.rs:5–16). It
  is false for the shipped image: `image/Dockerfile:24` sets `ENV PORT=8080`, so `listen_addr`
  binds `0.0.0.0`. D01M857ZB3FZJZWX8G3BF5W1RNR §1 says so itself. Backstage is scored on its
  production template, and the brief's F6 says to probe "the template's production config and
  image" for both. Judged like for like, both are false, so this rule is **parity**. The basis
  favours Seikika.
- **`extensionCredentialIsolated: true`** (facts.yaml:23–29) holds against *accidental*
  inheritance only. Connectors are spawned as the server's UID, with no UID change and no
  `PR_SET_DUMPABLE` anywhere in `crates/`. A same-UID process can read its parent's
  `/proc/<pid>/environ`, which still holds `DATABASE_URL` and the PATs. I checked that this
  kernel allows such a read between same-UID siblings. Off token mode, on loopback, a connector
  can also call the API with `x-seikika-user`. The cited test checks only the allowlist
  constant. Against the brief's malicious canary (F7 attempts 1 and 3), Seikika is not isolated
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
   MCP `query` tool, `seikika query` and the UI catalog filter.
4. RESULTS.md:131–133: say that `loopbackByDefault`, `extensionCredentialIsolated` and
   `capScorecards` are scored on bases that favour Seikika (§5), or rescore them.
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
   spread reads over ids and owners; implement the brief's equivalence gate; and run the Seikika
   image, not the bare binary, for the security probes.
9. Product, outside the write-up:
   - fix `916f596`'s two edge cases (kinds that differ only in case; a declared `id` field);
   - make `rule_standing` return `unknown` when one side lacks a value on a value rule;
   - move the orphaned doc comment at seikika-derive/src/lib.rs:1221–1223 back to
     `level_ignoring_unknown`;
   - consider running connectors under their own UID, or non-dumpable, before claiming credential
     isolation.

## Round 3

*2026-11-23 (VM clock 2026-11-24 UTC). The same adjudicator, again independent of the design. I
appended this section and changed nothing else in the repository; every probe wrote only under
`/tmp/adj3`, and the only build output is `target/`. Inputs: the harness frozen at `e66526c`, the
scorecards frozen at `581be50`, [results/r3](./results/r3), RESULTS.md and README.md at HEAD
(`ddde8ce`), `portal/comparisons/`, and the product commits `5968a55`, `71d8655`, `26b77f3`,
`afc8ca8` and, after the run, `d289b1a`.*

### R3.1 Conclusion

**I uphold the factor verdict: after round 3, Seikika is ahead on seven of the eight pre-registered
factors and behind on capabilities. I do not uphold one rule: `extension-credential-isolated` is not
a pass on the benchmark's own VM.** On this VM, a process with the server's user reads OpenFGA's
memory and gets the datastore password and the per-start OpenFGA key, and OpenFGA accepts the key
(R3.3). Scored as a fail, that rule is at parity, and security goes from 3 / 5 / 0 to 2 / 6 / 0: still
*ahead*, on owner-only fields and process isolation. No factor changes standing.

The numbers recompute exactly, and the freeze preceded the run. The harness changes made after
rounds 1 and 2 correct defects, and together they move the result toward Backstage, not toward
Seikika (R3.4). The two-token arrangement is fair to Backstage, but its stated reason is imprecise.
Round 1's loss at 2.4× holds: I reproduced Backstage's real lists under round 1's conditions (R3.6).

**I correct my own round 1–2 adjudication.** Backstage's round 1 and 2 lists were empty. Their
filter, `spec.owner=group:default/g3`, returns 0 items (a 41-byte body) on the live install, and
`spec.owner=group:g3` returns 20 items at 1k. My §4.2 equivalence row ("None expected … both pages
hold 100. Unverified per response") was wrong. So were the Backstage *list* figures in my §4.2
logging and spread probes (about 1.3k/s), which used the same filter (`/tmp/adj/rand-backstage-1.list.urls`).
Their get figures stand. Round 1 is still *behind* on performance and round 2 still *ahead* (R3.6).

| Factor | Round 3 as scored | Round 3 as adjudicated | Robust? |
|---|---|---|---|
| Performance | **ahead** (7 / 0 / 0) | **ahead** (7 / 0 / 0) | Yes. Smallest margin: 10k list 3,790 vs 474 req/s. Reads are still mostly decision-cache hits (R3.2, fix 7) |
| Footprint | **ahead** (5 / 0 / 0) | **ahead** (5 / 0 / 0) | Yes |
| Security | **ahead** (3 / 5 / 0) | **ahead** (2 / 6 / 0) | Yes. `extension-credential-isolated` is a fail on this VM, so it is at parity (R3.3) |
| Supply chain | **ahead** (4 / 0 / 0) | **ahead** (4 / 0 / 0) | Yes, now with OpenFGA's Go modules and the UI counted |
| Change safety | **ahead** (4 / 0 / 0) | **ahead** (4 / 0 / 0) | Yes, though `upgrade-lines` still compares unlike measures (disclosed) |
| Org fit | **ahead** (1 / 1 / 0) | **ahead** (1 / 1 / 0) | Yes |
| Agent surface | **ahead** (2 / 1 / 0) | **ahead** (2 / 1 / 0) | Yes, also against a plugin-scoped Backstage MCP token (14,744 vs 6,473 bytes; R3.5) |
| Capabilities | **behind** (0 / 6 / 7) | **behind** (0 / 6 / 7) | Yes |
| *Extensibility (not pre-registered)* | *ahead (2 / 0 / 0)* | *ahead (2 / 0 / 0)* | *Disclosed as a deviation* |

### R3.2 What holds

- **Recomputation.** From the 20 files in `results/r3`, every min, median and max in RESULTS.md's
  round 3 table matches: 9 facts × 2 sizes × 2 arms. Each committed file equals its raw file in
  `/tmp/bc/r3`. Every read has `ok` = 1.0 and `distinct` = 1,000 ids (get) or 50 owners (list).
- **Equivalence gate.** Each size has one `ownersDigest` across both arms and all runs: `d3508386…`
  at 1k and `d12a71a1…` at 10k.
- **Product records and report.** `portal/comparisons/` carries the 10k medians and the probe,
  footprint and upgrade values. `seikika report portal --skip runner,github-actions -o md`, debug
  build at HEAD, gives the same standing on all 48 rules as my hand derivation. RESULTS.md:17–27
  reproduces that standing.
- **Freeze before the run.** `e66526c` and `581be50` were committed at 00:26:59. The first raw file
  is from 00:27:53, and the run log ends `RUN-DONE 0`. The 48 rule ids match rounds 1–2; I kept no
  copy of the old expressions, so I cannot diff those.
- **The measured binary.** `target/release/seikika` was built at 00:20:59. The only later code
  commit before the freeze is `afc8ca8`, a doc comment. I did not rebuild to prove byte identity,
  but the binary behaves like `581be50`: it is non-dumpable and starts OpenFGA with a preshared key
  (R3.3). `d289b1a` came after the run. It is a refactor with no behaviour change: same
  `matches`/`may_match` logic, with the policy value now built only when a filter is present.
- **`5968a55`** fixes both edge cases from my §3 (fix 9). Narrowing applies only when `kind` names
  exactly one kind, and never on `id`. `field_index_narrowing_respects_kind_case_and_the_entity_id`
  passes. It names outside uses and touches no harness file (P5). The benchmark's list (one kind,
  field `owner`) takes the same path as before.
- **The requested tests pass:**
  - `SEIKIKA_OPENFGA_BIN=/tmp/tools/openfga SEIKIKA_REQUIRE_OPENFGA=1 cargo test -p seikika-authz openfga`: 3 passed;
  - `cargo test -p seikika-cli --test serve`: 1 passed;
  - `cargo test -p seikika-api --test api`: passed.
- **What the security probes measure.** The restricted client token gets 403 on DELETE, refresh
  and location POST, and the entity remains.
- **Most of the §6 fixes are applied** (table below).

| §6 fix | Status at HEAD |
|---|---|
| 1. Round 1 tally | Applied: no tally is stated, and round 1's table is labelled |
| 2. Eight factors | Applied (RESULTS.md:10–12, 233–234) |
| 3. "Owner and team pages" | Applied (RESULTS.md:131–132; design doc §4) |
| 4. Facts in Seikika's favour | Applied for loopback and scorecards, rescored to parity. Credential isolation was re-evidenced, but the evidence does not hold (R3.3) |
| 5. Deviations | Scorecards frozen and committed before the run; design doc committed (`586d178`, after the run); 24 of 48 rules from facts; the split; the production config. The derive fallback is disclosed (RESULTS.md:234–235), **but README.md:65 still says the opposite** |
| 6. Pre-freeze commits | Applied, **but README.md:69–70 calls `5ef6263` "per-snapshot authorization decisions"**. `5ef6263` is "evaluate a list filter once when the kind has no owner-only fields"; per-snapshot decisions are `a62d92f`, already listed among the smoke-round fixes |
| 7. Threats | Applied: logging (fixed in the harness), oha pinning, 10 s closed loop, supply chain (fixed), the unlike upgrade measure, the token (fixed), and the gate's limits. **Not applied: "Seikika checks every read in OpenFGA"** (README.md:20, 105; RESULTS.md:60). Decisions are kept per snapshot, and round 3 reads 1,000 ids for 10 s at about 56k req/s, so each id is checked once and then served from cache |
| 8. Next round | Same log level, restricted token, spread reads, gate: done. Spread is 1,000 of 10,000 ids, not all. **Security probes on the image: not done**, disclosed (README.md:114) |
| 9. Product | Edge cases fixed (`5968a55`); doc comment fixed (`afc8ca8`); server non-dumpable (`71d8655`). `rule_standing` is unchanged and disclosed instead, which D01M857ZB468ZM2YS6GNSEG6YB9 allows. Connectors still share the UID of a dumpable OpenFGA (R3.3) |

### R3.3 Credential isolation (F7), against `581be50`

The rule's check is boolean: an extension gets none of the server's credentials. In the image, an
in-image connector runs as UID 65532, like the server and OpenFGA (`image/Dockerfile`: `USER
65532:65532`). I attacked it as such a process: same user, not an ancestor of OpenFGA.

| Claim (RESULTS.md:203–220, facts.yaml) | Probe | Result |
|---|---|---|
| The server's environment and memory are unreadable | `/proc/<server>/environ`, release binary | **Holds**: `PermissionError`. `serve.rs` test passes |
| OpenFGA gets an allowlisted environment, and no URI in argv | `openfga_gets_no_server_credentials`; `/proc/<openfga>/cmdline` | **Holds** |
| The config file is gone before a connector can look | `OpenFga::start` drops it after `wait_healthy`; `spawn_in_image` needs the `AppState` built after `start_authz` (server.rs:204–221) | **Holds** |
| OpenFGA refuses calls without the key | `GET /stores` with no header | **Holds**: 401 |
| A same-user connector cannot read OpenFGA's memory "under the usual default of 1" | This VM has no Yama (`/proc/sys/kernel/yama` is absent). I read `/proc/<openfga>/mem` | **Does not hold here.** OpenFGA's memory has its `config.yaml` text: `uri: "postgres://postgres:S3cretProbePW@…"` and `authn: preshared: keys: ["tbx_50b5…"]`. `GET /stores` with that key returns **200**. The image cannot choose the host kernel's Yama setting; with Yama built in, the default is 1, but this host has none, and other container hosts are unverified |
| (not claimed) argv | `/proc/<server>/cmdline` with `--database-url postgres://postgres:S3cretProbePW@…` | **Leaks to every local user**, any UID, since non-dumpable does not cover cmdline. The harness passes the URL this way (measure.py:151–152), and AGENTS.md documents `serve --database-url`. The image passes it in the environment, so it is unaffected |
| A connector can write any file the server's user can | not probed | Unverified whether `/portal` is writable in the image |

**Verdict on F7.** Seikika now has a real barrier, and Backstage has none (plugins share one
process). But the rule asks for a pass. On the measurement host, a connector gets the database
password and the authorization key in a few lines of code. The pass depends on the host's kernel,
which the image does not ship. Judged as each product ships, and on the host where it was measured,
the fact is `false`: the rule is at parity (both fail), and security is 2 / 6 / 0, still ahead. To
earn the pass, take the connector out of OpenFGA's reach:
- a separate UID;
- or a connector without `/proc` access to its siblings, such as Landlock or a PID namespace;

then rerun the canary on the image. (The canary is the F7 test: can a connector reach the server's
credentials?)

### R3.4 Do the post-hoc harness changes bias the result?

No. Each change answers a finding of the adjudication, and the net direction is toward Backstage:

| Change | Effect on the standing | Direction |
|---|---|---|
| Filter fixed (`spec.owner=group:gN`) | Backstage's 10k list falls from 1,279 to 474 req/s, because it now returns items | Against Backstage, but it repairs an invalid measure: the gate shows both arms now page out the same pairs |
| Equivalence gate | Stops a run on a mismatch | Neutral |
| Reads spread over 1,000 ids and 50 owners | Seikika's 10k get goes from 58.6k to 56.5k req/s; Backstage's 10k list is unchanged (my probe: 457 req/s spread, 457 median single-owner) | Neutral; still mostly cache hits for Seikika |
| Production config, `LOG_LEVEL=warn` | Backstage get +13% (my §4.2 probe) | Toward Backstage |
| Read-only client token | `client-cannot-delete` goes from ahead to parity | Toward Backstage |
| Loopback judged on the image, `capScorecards` counting add-ons | Two rules go from ahead to parity | Toward Backstage |
| Credential isolation re-evidenced | Stays ahead | Toward Seikika, and it does not hold (R3.3) |
| Supply chain widened | Seikika goes from 388 to 605 packages and from 2 to 5 vulns. Backstage's Node runtime is still not scanned | Against Seikika |
| Upgrade window `v0.1.0..HEAD` | 0 either way | Neutral |

RESULTS.md:241–243 ("each moves a measure toward Backstage or toward like-for-like") is accurate,
and "What changed" (RESULTS.md:36–42) says the filter fix went against Backstage.

### R3.5 Is the two-token arrangement fair to Backstage (P4)?

Yes, but the reason given for it is imprecise. With the restricted token, MCP's `tools/list` and
`tools/call` return 403 ("restricted to plugin(s) 'catalog'"). Adding `{plugin: mcp-actions}` gives
an empty tool list. The cause is `DefaultAuthService.getPluginRequestToken`
(backend-defaults, `DefaultAuthService.cjs.js:81–103`). It delegates a restricted service token only
when the target plugin's restriction has no permission names or attributes: "is restricted and
cannot be delegated".

I tried a plugin-scoped token, restricted to `{plugin: catalog}` and `{plugin: mcp-actions}` with
no permission attributes. MCP works with it, listing 7 tools in 14,744 bytes, measured as
`surface.py` measures. The same token can still DELETE an entity: 204. So:

- "Backstage does not delegate a restricted token" (README.md:20, RESULTS.md:49–50, the research
  log row) is imprecise. It does not delegate a *permission*-restricted one.
- "an agent token that can use MCP cannot be made read-only" holds.
- Scoring `client-cannot-delete` on the read-only REST token favours Backstage. Its MCP token can
  delete; Seikika uses one PAT for both, and it cannot. The unrestricted MCP token adds the five
  scaffolder tools to Backstage's list, which a plugin-scoped token would drop (20,500 vs 14,744
  bytes). Seikika is ahead either way, so no standing depends on it, but say so.

### R3.6 Is "round 1's list loss stands at 2.4×" fair?

Yes, as an estimate, and my probe confirms it. RESULTS.md:92–95 compares Seikika's round 1 (198
req/s, one owner, info-level Backstage logs, no production config) with Backstage's round 3 (474
req/s, 50 owners, warn logs, production config). I reran Backstage under round 1's conditions with
the filter fixed:
- the `dfeeea4` bench config;
- default logging;
- no `app-config.production.yaml`;
- the unrestricted token;
- 10k components, oha 10 s at 16 connections, two fresh starts.

| Backstage, round 1 conditions, filter fixed | Run 1 | Run 2 |
|---|---|---|
| Owner `g3` only (round 1's URL): req/s, p95 | 482.2, 56.4 ms | 432.0, 66.7 ms |
| 50 owners: req/s, p95 | 456.8, 59.0 ms | 453.4, 59.3 ms |

So Seikika at `5ef6263` (197.6 req/s, p95 118 ms) was behind by 2.2–2.4× on throughput and about
2× on p95. Round 1's *behind* stands, and round 2 (3,900 req/s) is *ahead* by more than RESULTS.md
said for round 2. The write-up should say the 2.4× is a cross-round estimate, and cite this check
(2 runs, not 5).

### R3.7 Other observations

- Backstage's 1k list is slower than its 10k list: 332 against 474 req/s, p95 76 against 59 ms.
  Both answers pass the gate. This is unexplained, possibly a query plan, and affects no standing.
- One Backstage 10k run listed at 117 req/s with a p95 of 306 ms. The median absorbs it, and
  RESULTS.md reports it.
- The design doc §1, the site page ("ahead (3 rules), parity on 5") and the research-log row were
  written before this adjudication. They state credential isolation as a win.

### R3.8 Required corrections

1. **`extension-credential-isolated`**: set the Seikika fact to `false`, with R3.3's evidence, at
   the next freeze, and regenerate the records. Security becomes 2 / 6 / 0, still ahead. Or make it
   hold (connectors under their own UID, or without access to their siblings' `/proc`) and rerun
   the canary on the image. Until then, correct RESULTS.md:20 and 203–220, the design doc §1, and the
   site page's security row.
2. **RESULTS.md:216–218**: say that the measurement VM has no Yama, and that the residual was
   exploited there: the datastore password and the OpenFGA key came out of OpenFGA's memory, and
   the key was accepted. Drop "usual default", or mark it unverified for container hosts.
3. **Disclose the `--database-url` exposure.** It is world-readable in `/proc/<pid>/cmdline`, and
   the harness uses it. Product, outside the write-up: take the URL from the environment or a file
   only, or warn when it is on argv.
4. **README.md:69–70**: `5ef6263` is the list-filter commit, not per-snapshot decisions (`a62d92f`).
5. **Name `5968a55`** among round 3's product changes (README.md:81–87, RESULTS.md:239–240).
6. **"Seikika checks every read in OpenFGA"** (README.md:20, 105; RESULTS.md:60): say that each
   decision is checked once per snapshot and then cached, so round 3's 1,000-id spread is mostly
   cache hits. My round-2 probe over all 10,000 ids gave about 36–37k get/s.
7. **The two-token reason** (README.md:20, RESULTS.md:48–50, the research-log row): Backstage
   delegates a plugin-scoped token, not a permission-scoped one. An MCP-capable token can therefore
   delete. With a plugin-scoped token, Backstage's tool list is 14,744 bytes.
8. **The 2.4×** (RESULTS.md:40–41, 92–95): label it a cross-round estimate, confirmed by R3.6.
9. **README.md:65 against RESULTS.md:234–235 and D01M857ZB468ZM2YS6GNSEG6YB9**: align the README's
   missing-value rule at the next freeze.
10. **Still open from §6:** run the security probes and the F7 canary on the image.
