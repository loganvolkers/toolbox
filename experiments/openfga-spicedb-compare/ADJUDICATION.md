# Adjudication: OpenFGA v1.23.0 against SpiceDB v1.57.1

*2026-11-24. Written by an agent acting as the independent adjudicator (dx-benchmarks.md §2,
"Independent truth"; P7). I did not design or run this experiment. I created this file and changed
nothing else in the repository: no commits and no edits. Every probe below wrote only under
`/tmp/adj-ofs`.*

*Inputs: the pre-registration [README.md](./README.md), [RESULTS.md](./RESULTS.md), the files in
[raw/](./raw), [facts.yaml](./facts.yaml), the harness, the records in
`examples/openfga-portal/comparisons/`, the scorecards in
`examples/openfga-portal/standards/vs-spicedb.yaml`, D01M857ZB468ZM2YS6GNSEG6YB9, and both products'
repositories at their pinned tags (openfga `d67ba7b`, spicedb `7074da7`). While I worked, another
agent rebased this branch. The pre-registration commit `c03624c` became `774f45a`, and the results
commit `de653eb` became `6d83040`. Their content is identical (`git diff --quiet` on the README, the
scorecards, the experiment directory and the records). This file cites the current hashes.*

## Conclusion

**I uphold the published verdict. Under the pre-registered tally, OpenFGA is behind on performance,
ahead on footprint, supply chain, modelling and ecosystem, and at parity on operability.**

- Every timed number recomputes exactly from `raw/`.
- No bar, rule, tolerance, factor or tally rule changed after the run.
- Both arms gave the same answers as an independent oracle in all ten rounds.
- Each timed standing is the same in all 25 pairings of an OpenFGA round with a SpiceDB round.

I correct two facts, both on SpiceDB's side and both in SpiceDB's favour, and several statements in
RESULTS.md. No factor verdict changes, under either counting rule.

The performance verdict holds as pre-registered: defaults, on the memory datastore. It is not a
statement about the two check engines, and the headline should say so. Two things decide it (§6):

- **Cache defaults.** SpiceDB ships a check cache that is on by default; OpenFGA's is off. The
  harness cycles one 2,000-check sample, so after warm-up an enabled cache serves almost every
  check. In my post-hoc probes, the check rules reverse when the caches are made the same.
- **OpenFGA's memory adapter.** Its README calls it "for development only", and it scans every
  stored tuple on each read and each write.

| Factor | Pre-registered tally, as published | Pre-registered tally, adjudicated | Report rule (any rule behind), adjudicated | Robust? |
|---|---|---|---|---|
| Performance | **behind** (2 / 0 / 3) | **behind** (2 / 0 / 3) | behind | Yes under the pre-registered arms, with no overlap between rounds. Reverses if both products run with caches on, or both with caches off (§6) |
| Footprint | **ahead** (3 / 0 / 0) | **ahead** (3 / 0 / 0) | ahead | Yes |
| Supply chain | **ahead** (4 / 0 / 0) | **ahead** (3 / 1 / 0) | ahead | Yes. `sbom` goes to parity, and `known-vulns` stays ahead at 0 against 3 |
| Modelling | **ahead** (2 / 6 / 1) | **ahead** (2 / 6 / 1) | behind | Fragile: it rests on two OpenFGA-named features, both verified (§4) |
| Operability | **parity** (1 / 3 / 1) | **parity** (1 / 3 / 1) | behind | Fragile: one rule either way, both verified |
| Ecosystem | **ahead** (3 / 0 / 1) | **ahead** (3 / 0 / 1) | behind | Yes |

Counts are OpenFGA ahead / parity / behind. On the absolute bars, OpenFGA fails 4 of 30 rules, as
published. SpiceDB fails **7**, not 8, because `sbom` passes.

### Corrections

| # | Where | Published | Corrected | Evidence | Changes a verdict? |
|---|---|---|---|---|---|
| 1 | `knownVulns`, SpiceDB (spicedb.yaml; RESULTS.md:32, 46, 88, 130) | 7 | **3** (GO-2026-6859, GO-2026-6974, GO-2026-6975) | The README defines the fact as "findings in called code". Both release binaries are stripped (no `.symtab`). On a stripped binary, govulncheck v1.10.0 "report[s] vulnerabilities at the go.mod-level precision" (`x/vuln@v1.10.0/internal/vulncheck/binary.go:107–111`). Every trace in `raw/govulncheck-spicedb.json` has length 1, and GO-2026-5932's symbol is the placeholder `openpgp/*`. SpiceDB's `.gopclntab` has no function from `x/net/html`, `x/crypto/openpgp`, `klauspost/compress/fse` or `huff0`. Source mode at v1.57.1, with the release's `-tags memoryprotection` and go1.27.3, finds exactly three called (§4) | No. 0 against 3 is still *ahead*, and SpiceDB still fails `known-vulns == 0` |
| 2 | `sbom`, SpiceDB (spicedb.yaml; RESULTS.md:32, 47, 90, 132, 200) | false | **true** | The README (line 64) and the scorecard define `sbom` as "the release publishes an SBOM"; only `provenance` names the archive. GoReleaser publishes the v1.57.1 release's images (`.goreleaser.yml:115–138`). `ghcr.io/authzed/spicedb:v1.57.1` carries a BuildKit SPDX SBOM attestation (syft v1.51.0; 222 packages, 203 Go purls, including `authzed-go v1.11.0`). Its `spicedb` binary is byte-identical to the archive's (sha256 `1ab32536…`). Under a strict archive-only reading, false stands, but the README would then have to say so | No. Supply chain goes from 4 / 0 / 0 to 3 / 1 / 0, still *ahead* |
| 3 | RESULTS.md:227 | "Binary mode reports vulnerable symbols linked into the binary" | On these stripped binaries, it reports at module precision | As in row 1 | No |
| 4 | RESULTS.md:229–230 | SpiceDB's images "may carry build attestations; they were not checked" | They carry SLSA v1 provenance and an SPDX SBOM, both unsigned | BuildKit attestation manifests in the image index. The provenance has an empty `builder.id`. There is no cosign `sha256-….sig` tag, the registry gives 404 for referrers, and the GitHub attestations API gives 404 for both the index and the amd64 digest | No. `provenance` asks for signed, verifiable provenance, so false stands |
| 5 | RESULTS.md:222 | "consistent with log sampling (unverified)" | **Verified** | OpenFGA builds its logger with `zap.NewProductionConfig()` (`pkg/logger/logger.go:178`). In zap v1.28.0, which OpenFGA's go.mod pins, that config sets `Sampling{Initial: 100, Thereafter: 100}` | No |
| 6 | RESULTS.md:220 | SpiceDB logged "about 140 MB per round" | 120–217 MB (139, 211, 209, 217 and 120 MB) | `logs/round-*-spicedb.log` sizes | No |
| 7 | RESULTS.md:238–241 | The scorecard description and track text "say '23k-relationship corpus'" | Both say 33k. `6d83040` changed them, after the run | `git show 6d83040 -- examples/openfga-portal/standards/vs-spicedb.yaml examples/openfga-portal/toolbox.yaml` | No, but it is a post-run edit to a pre-registered file and belongs under *Deviations* (§1) |
| 8 | RESULTS.md:61 | The any-rule verdict is in `toolbox-report.js` | Derive computes it (`crates/toolbox-derive/src/lib.rs:763–766`, per D01M857ZB468ZM2YS6GNSEG6YB9), and the report renders it | Source | No |
| 9 | RESULTS.md:163, *Deviations* | The harness was `a411799+uncommitted` | Also, two harness files changed **after** the run: `load/main.go` at 22:19:35 and `facts.py` at 22:19:34. The run ended at 22:11:41, `load/loadgen` was rebuilt at 22:19:37, and `6d83040` was committed at 22:24:10. The committed harness is not proven to be the one that ran | File mtimes. The round and facts JSON have exactly the fields the committed code writes | Unknown, probably no. Disclose it |
| 10 | run.sh:39 at `6d83040` | Tools pinned | The run's harness installed govulncheck `@latest`, unlike slsa-verifier and cosign. The version that ran is recorded (v1.10.0, DB 2026-11-24T15:56Z), and `c2f7219` pinned it to v1.10.0 after the run | Source | No. My rerun with the same version and DB reproduces all 7 binary-mode ids, and 0 for OpenFGA |

## 1. Pre-registration integrity

**Nothing pre-registered changed after the run.** `git log -p` on both files:

- `README.md`: two commits. `774f45a` (21:20:42) creates it, and `6d83040` (22:24:10) only appends
  `## Deviations`, 21 lines. No line of the pre-registration is edited.
- `vs-spicedb.yaml`: two commits. `6d83040` changes one line, the performance scorecard's
  description, from "23k-relationship corpus" to "33k-relationship corpus". No `check`, `value`,
  `better`, `tolerance`, level or factor grouping changed.
- `examples/openfga-portal/toolbox.yaml`: the track text got the same 23k → 33k change in
  `6d83040`. Later, `c21955f` added a "Not covered" sentence that names the datastores, nodes and
  durations the comparison omits. Both are descriptions only.

**Timeline.** Git timestamps, plus file mtimes where git has none. The mtimes are weaker evidence.

| Time (UTC) | Event | Evidence |
|---|---|---|
| 21:20:42 | Pre-registration and scorecards committed | `774f45a` |
| 21:29–21:41 | `gen.py`, the models, `load/*.go`, `facts.yaml` (21:40:34) and `run.sh` (21:41:25) written | mtimes |
| 21:36:41 | `downloads/` created; the first product download | mtime |
| 21:42:07 | `run.sh` starts | `raw/env.json` |
| 21:46:11–22:11:17 | Ten timed rounds | `startedAt` and the round files' mtimes |
| 22:11:41 | Facts done; `RUN_EXIT=0` | `logs/run.log` |
| 22:16:31 | `snapshot.py` writes the records | mtimes |
| 22:19:34–37 | `facts.py` and `load/main.go` edited, `loadgen` rebuilt | mtimes (correction 9) |
| 22:24:10 | Harness, raw files, records and RESULTS.md committed | `6d83040` |

The pre-registration came before any product was on disk, as the README says. As in the Backstage
comparison, it did not freeze the harness or `facts.yaml`. `facts.yaml` decides only three rules on
its own, for both arms: `datastores`, `official-sdks` and `foundation`. Probes decide the other ten
yes/no facts. facts.py reports no disagreement between a probe and `facts.yaml`
(`raw/facts.json`: `"disagreements": []`).

**The tally rule.** README:77–79 says a factor is ahead when OpenFGA "wins more of its rules than
it loses", but not how a single rule is won. snapshot.py and derive use the scorecards'
pre-registered `tolerance` (0.1, and 0.05 for binary size), and pass/fail for boolean rules. Both
were in `774f45a`, so this is pre-registered, if in a different file.

**The four disclosed deviations.** None biases a result toward OpenFGA:

| Deviation | Effect |
|---|---|
| Listen addresses on 127.0.0.1:18xxx | Neutral: the same for both arms, and the default-port probes decide `httpByDefault` and `prometheusMetrics` |
| Equivalence after the write, not before any timing | Neutral: the same order in both arms, and the gate still stops the run on any difference (run.sh:113–118) |
| `readyS` includes setup | Slightly against OpenFGA: it makes two RPCs (CreateStore, WriteAuthorizationModel), where SpiceDB makes one (WriteSchema). OpenFGA is still ahead |
| Load gate before each round | Neutral: symmetric, and all ten rounds started below 1.0 (§3) |

**Not disclosed:**

- the post-run edit to a pre-registered scorecard (correction 7);
- the post-run harness edits (correction 9);
- `sbom` checked for the archive although the README defines it for the release (correction 2);
- govulncheck installed unpinned (correction 10, since fixed);
- three list passes per round, which RESULTS.md describes but the README does not.

None of these bars or rescores a result.

## 2. Recomputation

My own script (`/tmp/adj-ofs/recompute.py`) reads the ten round files and takes the median, min and
max over the five rounds of each arm. It also recomputes throughput from raw counts:
`check.count / 30` and `write.relationships / write.seconds`.

| Fact | OpenFGA, median (min–max) | SpiceDB, median (min–max) | RESULTS.md and records |
|---|---|---|---|
| `readyS` | 0.059 (0.048–0.063) | 0.141 (0.121–0.168) | match |
| `writeTuplesPerS` | 3,160 (2,950–3,166) | 49,399 (46,443–53,145) | match |
| `checkP95Ms` | 20.07 (19.53–27.63) | 2.05 (1.86–5.45) | match |
| `checkRps` | 1,473.4 (1,233.8–1,507.4) | 15,662.1 (9,849.5–16,481.5) | match; the records round to 1,473 and 15,662 |
| `listP95Ms` | 3.24 (3.03–3.77) | 9.07 (8.04–10.69) | match |
| `rssIdleMb` | 53.4 (53.0–54.5) | 84.5 (84.2–84.8) | match |
| `rssLoadedMb` | 101.5 (100.7–102.3) | 306.4 (303.8–307.2) | match |
| check p50 / p99 ms | 10.20 (9.97–11.29) / 25.02 (24.37–40.08) | 0.56 (0.55–0.79) / 14.45 (13.47–20.75) | match |
| list p50 ms | 2.26 (2.07–2.67) | 2.14 (1.70–5.61) | match |
| RSS after the check load | 102.3 (101.2–102.6) | 306.4 (301.5–307.2) | match |
| equivalence s | 1.51 (1.44–2.51) | 2.26 (2.08–3.16) | match |

Throughput recomputed from raw counts agrees with the stored `rps` and `perS` to rounding. Check
errors, check mismatches and list errors are 0 in all ten rounds.

**The counted facts reproduce:**

- `binaryMb` 67.4 and 82.3 (67,387,552 and 82,337,952 bytes, the same sha256 as `raw/facts.json`);
- `goModules` 142 and 190 (`go version -m`, `dep` lines);
- `releasesLastYear` 28 and 15, over the same window;
- `contributors` 122 and 86, recounted after the run;
- govulncheck in binary mode, 0 and 7 ids, the same as `raw/`, but see correction 1.

**The records and the report agree.** `comparisons/openfga.yaml` and `spicedb.yaml` hold these
medians and facts, field for field. `target/debug/toolbox report .. -o json --skip
runner,github-actions`, run from the portal's checkout with output to `/tmp`, gives the same 30
per-rule standings as RESULTS.md. My own standing code, which does not reuse snapshot.py, agrees
with both.

**All 25 round pairings agree.** For every timed rule, pairing each OpenFGA round with each SpiceDB
round gives one standing every time. The rounds never overlap: OpenFGA's best check rate, 1,507/s,
is below SpiceDB's worst, 9,850/s, and SpiceDB's fastest ready time, 0.121 s, is slower than
OpenFGA's slowest, 0.063 s. The medians are not hiding a close call.

**No mismatch** between the raw files, RESULTS.md and the records.

## 3. Equivalence of the arms

| Check | Result |
|---|---|
| Same corpus | `gen.py`, rerun in `/tmp`, writes a `corpus.json` byte-identical to the one used. It has 33,200 relationships: 2,000 memberships, 1,000 folder owners, 100 parents, 10,000 each of document folder, viewer and owner, and 100 `user:*` |
| Same answers | All ten rounds have `decisionsDigest a782efbc…` and `listsDigest 4831183f…`, with 1,041 allowed, identical `listSizes` and `oracleMatch: true`. List passes returned 51,096 results in every round (3 × 17,032), so neither arm returned duplicates |
| Same releases as shipped | Both archives pass `sha256sum -c` against their `checksums.txt`. The extracted binaries match `bin/` and `raw/facts.json`. Both were the latest releases at the run: SpiceDB v1.57.1 (2026-11-10) and OpenFGA v1.23.0 (2026-11-20) |
| Same client | One Go program (`load/`) with one gRPC connection per arm and 16 workers. Timed phases use each API's default consistency (OpenFGA `UNSPECIFIED`, SpiceDB unset). The equivalence pass uses each one's strongest (`HIGHER_CONSISTENCY`, `fully_consistent`) |
| Alternation | Order by `startedAt`: OpenFGA then SpiceDB in rounds 1, 3 and 5, and SpiceDB then OpenFGA in rounds 2 and 4, as `run.sh:123–127` intends |
| Load gating | Every round started with a 1-minute load average of 0.84–0.98 and ended at 2.43–3.73. Waits were 60–380 s, and none gave up |

Asymmetries that remain:

- OpenFGA's checks pass `authorization_model_id`, the documented fast path; SpiceDB has no
  equivalent.
- `readyS` setup differs, as described in §1.
- SpiceDB's 140,000-plus log lines per round are a cost OpenFGA's sampled log does not pay (§6).

None of these favours OpenFGA enough to matter.

## 4. Spot checks against primary sources

I read each source at the pinned tag (`git clone --branch v1.23.0`, `--branch v1.57.1`), in the
proto module compiled into each binary, or through `gh api` on the pinned docs commits.

| Fact | Published | What I found | Agree? |
|---|---|---|---|
| OpenFGA `consistencyTokens` | false | At `openfga/api@f153694` (OpenFGA's go.mod:33), `message WriteResponse {}` (openfga_service.proto:1315). `ConsistencyPreference` has only `UNSPECIFIED`, `MINIMIZE_LATENCY` and `HIGHER_CONSISTENCY`. The docs at `a13c70e`, under "Future work", say zookies are something "OpenFGA is considering". Close call: with its cache off by default, an OpenFGA check on one primary reads fresh data, but it cannot pin a read to a write | **Yes** |
| SpiceDB `contextualTuples` | false | authzed-go v1.11.0 (spicedb go.mod:14). `CheckPermissionRequest` has consistency, resource, permission, subject, context and with_tracing. `LookupResourcesRequest` and `LookupSubjectsRequest` likewise have no relationship list. The only "contextual" type is `ContextualizedCaveat`, which is on stored relationships. Nothing in the v1.57.1 source or CHANGELOG mentions contextual tuples or relationships | **Yes** |
| SpiceDB `multiTenant` | false | `WriteSchemaRequest` has only `schema`. The schema docs at `ff2f067` offer prefixes (`docs/document`, `iam/user`) in one schema, which is naming, not isolation | **Yes** |
| `httpByDefault` | true / false | OpenFGA: `HTTP: {Enabled: true, Addr: "0.0.0.0:8080"}` (config.go:897–898). SpiceDB: `RegisterHTTPServerFlags(…, ":8443", false)` (serve.go:92), where the last argument is the default of `--http-enabled` (util.go:366, 374) | **Yes** |
| `datastores` | 2 / 4 | OpenFGA README:226: "PostgreSQL 14+, MySQL 8, SQLite (beta)"; run.go:525–545 has memory, mysql, postgres and sqlite. SpiceDB README:279: Spanner, CockroachDB, MySQL and PostgreSQL; the docs mark MySQL "Not recommended". Counted the other way (SQLite in, MySQL out), it is 3 against 4 or 2 against 3: still *behind* | **Yes** |
| `contributors` | 122 / 86 | Recounted with `gh api --paginate 'repos/<repo>/contributors?anon=1&per_page=100'`. OpenFGA: 112 accounts, 8 anonymous, 2 bots (dependabot, Copilot). SpiceDB: 82 accounts, 2 anonymous, 2 bots (dependabot, renovate). It counts commit authors on one repository's default branch, for both. With bots and anonymous entries excluded, it is 112 against 82: OpenFGA still passes ≥ 100 and SpiceDB still fails | **Yes** |
| `officialSdks` | 5 / 6 | The unarchived openfga org repositories are go-, js-, python-, java- and dotnet-sdk (plus sdk-generator, not counted). In the authzed org, six repositories are described as "Official SpiceDB client library": go, py, rb, node, java and dotnet. `gochugaru` and `spicedb-clients-prototype` are not so described. The rule has no tolerance, so 5 against 6 is *behind* although both pass | **Yes** |
| SpiceDB `knownVulns` | 7 | **3 called**: GO-2026-6859 (net/http, `spicedb.main` → … → `http.response.WriteString`; fixed in go1.27.4), GO-2026-6974 (otel/sdk, via the Spanner reader in `datastore gc`) and GO-2026-6975 (otel baggage, from `envconfig.init`). Four are in required modules but not compiled in: GO-2026-5932 (x/crypto/openpgp), GO-2026-6860 and GO-2026-6866 (x/net/html), and GO-2026-6865 (klauspost/compress). `go mod why golang.org/x/net/html` says "main module does not need package", and the binary's function table agrees | **No**: correction 1. No verdict change |
| OpenFGA `knownVulns` | 0 | 0 even at module precision. The binary has x/net v0.62.0 and otel v1.47.0, the fixed versions, is built with go1.27.4, and has no x/crypto or klauspost/compress | **Yes** |
| `provenance` | true / false | OpenFGA: I reran `slsa-verifier verify-artifact` (v2.7.1): PASSED. `cosign verify-blob checksums.txt`: Verified OK. SpiceDB: the release assets are archives, packages, `development.wasm` and two checksum files. `.goreleaser.yml` and `release.yaml` have no signing or attestation step. Its images' provenance is unsigned (correction 4) | **Yes** |
| `sbom` | true / false | OpenFGA: `openfga_1.23.0_linux_amd64.tar.gz.sbom.json`, listed in the signed `checksums.txt`. SpiceDB: no archive SBOM, but the release images carry an SPDX SBOM covering the identical binary | OpenFGA yes; **SpiceDB no**: correction 2 |
| `releasesLastYear` | 28 / 15 | Recounted over the same window: 28 and 15, none pre-release | **Yes** |

Two of RESULTS.md's limitations are also confirmed:

- OpenFGA's check caches are off by default: `DefaultCheckQueryCacheEnabled = false` and
  `DefaultCheckIteratorCacheEnabled = false` (config.go:43, 46). SpiceDB's dispatch cache is on, as
  the startup log shows (`DispatchCacheConfig.Enabled: true`).
- SpiceDB's startup release check took 58 ms in round 1, from "configured logging" at
  21:48:09.043 to "this is the latest released version" at 21:48:09.101.

## 5. Verdicts, two ways

**The pre-registered tally** (README:77–79). A factor is ahead when it has more rules ahead than
behind, parity when equal, and behind otherwise. As published: performance behind (2 / 0 / 3),
footprint ahead (3 / 0 / 0), supply chain ahead (4 / 0 / 0), modelling ahead (2 / 6 / 1),
operability parity (1 / 3 / 1) and ecosystem ahead (3 / 0 / 1). Adjudicated, only supply chain's
counts move, to 3 / 1 / 0, and it stays ahead. **OpenFGA is ahead on four factors, at parity on
one and behind on one, as RESULTS.md says.**

**The report's rule** (D01M857ZB468ZM2YS6GNSEG6YB9 §1; derive, lib.rs:763–766). A factor is behind if
any of its rules is behind, else unknown if any is, else ahead if any is, else parity. Footprint and
supply chain are ahead. Performance, modelling, operability and ecosystem are behind. This is the
same before and after my corrections.

**Is the portal's headline fair to the reader?** It is accurate under its stated rule, and at HEAD
(`c21955f`) it is open about the rule. The lead number and the colour, however, invert the
experiment's pre-registered result. The Compare view (`report-ui/src/views/compare.js:178–191`)
reads:

> Ahead on 2 of 6 factors and 15 of 30 rules; behind on 4 factors and 6 rules. Behind on
> Ecosystem (1 of 4 rules), Modelling (1 of 9 rules), Operability (1 of 5 rules), Performance (3 of
> 5 rules)

With my corrections, the rule counts become 14 ahead and 10 at parity, with 6 still behind. Since
`c21955f`, the view also:

- marks a factor with rules on both sides "behind, mixed";
- states that "a factor counts as behind if any one of its rules is behind";
- warns about the `+uncommitted` harness.

The hero is still coloured *behind*: `lean` is behind, because 4 factors are behind and 2 ahead
(compare.js:192, 248). A reader who stops at the headline learns "behind on 4 of 6". The
pre-registered result is "ahead on 4 of 6". Three of those four "behind" factors lose a single rule
each, and each loss is narrow:

- consistency tokens, against OpenFGA's own documented future work;
- datastores, 2 against 4;
- SDKs, 5 against 6, with no tolerance.

The any-rule rule suits a ratchet on the subject's own portal, where any regression matters. It is
a poor summary of a pre-registered A/B. This is a finding about the box, not about either product,
and RESULTS.md (lines 60–66) already reports it. The fix is for a comparison to state its tally
rule and for the report to lead with it, or to show both.

**Fragility of the pre-registered tally.** Modelling is ahead on `contextual-tuples` and
`multi-tenant`. Both are OpenFGA feature names, a bias the README discloses. If either were scored
at parity, modelling would fall to parity. Operability would fall to behind if SpiceDB's
`httpByDefault` were true. I verified all three in §4, and they hold.

## 6. Threats to validity

The four probes below were run after the rounds, through the harness's own `loadgen round`, on ports
28xxx, behind the same load gate. They write to `/tmp/adj-ofs`, run once each and are **not** part of
the pre-registered measure. All four matched the oracle, with no errors and no mismatches.

| Probe | write/s | check p50 / p95 ms | checks/s | list p95 ms |
|---|---|---|---|---|
| OpenFGA, one tenth of the corpus (3,320 relationships), defaults | 27,270 | 1.87 / 3.66 | 7,809 | 1.19 |
| SpiceDB, one tenth of the corpus, defaults | 48,034 | 0.52 / 1.26 | 24,775 | 2.78 |
| OpenFGA, full corpus, `--check-query-cache-enabled --check-iterator-cache-enabled` | 3,194 | 0.46 / 1.10 | 26,035 | 2.85 |
| SpiceDB, full corpus, `--dispatch-cache-enabled=false` | 46,634 | 13.10 / 24.11 | 1,165 | 9.11 |
| *Published medians, for reference: OpenFGA / SpiceDB* | *3,160 / 49,399* | *10.20 / 0.56 and 20.07 / 2.05* | *1,473 / 15,662* | *3.24 / 9.07* |

| Threat | Evidence | Could it flip a verdict? |
|---|---|---|
| **Cache defaults decide the check rules.** The sample is 2,000 checks cycled for 35 s, so after warm-up an enabled cache answers nearly every check | The probes: with caches on for both, OpenFGA is ahead on checks (26,035/s and p95 1.10 ms, against 15,662/s and 2.05 ms). With caches off for both, OpenFGA is also ahead (1,473/s and 20.07 ms, against 1,165/s and 24.11 ms). Both differences are more than 10% | **Yes, under arms that were not pre-registered.** With equal caching, both check rules go to OpenFGA, and performance becomes 4 / 0 / 1, *ahead*. Under the pre-registered defaults it stays behind. Turning a cache on trades freshness: OpenFGA's flag help says it makes Check "eventually consistent". SpiceDB's default reads at a quantized revision (not re-read here) |
| **OpenFGA's memory adapter scans every tuple.** Its README says "Memory storage adapter is **for development only**" | `pkg/storage/memory/memory.go`: `read` (299–321), `ReadUserTuple` (494–512), `ReadUsersetTuples` (515–530) and `ReadStartingWithUser` (558–571) each loop over every stored tuple. `Write` compares each new tuple with every stored one (396–403), so writing the corpus is quadratic. SpiceDB's memdb uses indexed `go-memdb` tables. At one tenth of the corpus, OpenFGA writes 8.6× faster and checks 5.3× faster; SpiceDB's write rate does not change | **Possibly, on a production datastore; not tested.** `write-throughput` measures this adapter, not OpenFGA's Postgres or MySQL path. The README lists a Postgres run as out of scope, and the track text now says so |
| One shared 4-vCPU VM | Load averages reached 2.4–3.7 during rounds; SpiceDB's rounds 1 and 5 were slower (p95 4.95 and 5.45 ms) | No. All 25 pairings agree (§2) |
| The load generator on the same CPUs | 16 workers in the same VM. They cost more CPU at SpiceDB's 15,000/s than at OpenFGA's 1,500/s | No: if anything, it understates SpiceDB |
| Logging is asymmetric | SpiceDB logs every request: round 1 has 357,494 lines, 356,935 of them CheckPermission, and 120–217 MB per round. OpenFGA's zap sampling logged 4,188 Check lines in round 1 (correction 5) | No: it is a cost SpiceDB pays, and it can only widen SpiceDB's lead. Counted, as the bar is the default install |
| `readyS` includes SpiceDB's release check | 58 ms in round 1 (§4) | No: without it, SpiceDB's median is about 0.083 s against 0.059 s, still more than 10% apart |
| RSS is in MiB, labelled MB | `rssMb` divides kB by 1024 (`load/main.go:291–303`). In MB, OpenFGA idles at 56.0, SpiceDB at 88.6, and SpiceDB loaded is 321.3 | No: every bar still passes, and the ratios are unchanged |
| SpiceDB sizes its memory by the host | It set `GOMEMLIMIT` to 14,136,193,843 B from the 16 GB VM (round 1 log), and cache `MaxCost` is a share of memory | Unlikely. Even with the dispatch cache off, SpiceDB's loaded RSS was 285 MiB, far beyond 10% of OpenFGA's 101.5 |
| `knownVulns` depends on the scan date and the build | SpiceDB is 10 days older and built with go1.27.3; two of its three called findings are in OpenTelemetry, fixed in v1.47.0 | No. If SpiceDB had 0, supply chain would be 2 / 2 / 0, still ahead |
| Not a production setup: memory datastore, one node, one model shape, short runs | Pre-registered limitations; the track text at `c21955f` lists them | Unknown for performance; no for the other factors, which do not depend on the datastore |
| The harness was not frozen and was edited after the run | Correction 9 | Not as far as I can tell: the output schemas match, and every number recomputes |

**Net.** The threats that could flip a verdict all touch performance, and none is inside the
pre-registered method. They change what the performance verdict means, not whether it holds.
"SpiceDB answers checks about ten times faster" (RESULTS.md:15) is true of the defaults on this
workload. It should say that this is SpiceDB's on-by-default cache on a repeated sample, against
OpenFGA's off-by-default cache over a development-only adapter.

## 7. What to fix

1. **Records.** Set SpiceDB `knownVulns: 3` and `sbom: true` at the next run, or change the
   README's definitions to match what was measured: module precision, and an archive-only SBOM.
   Either way, update RESULTS.md:32, 45–47, 88, 90, 130 and 132, and the limitation at 225–228.
2. **Harness.** Make `facts.py` run govulncheck in source mode at the tag, with the release's
   build tags and Go version, or note that binary mode on a stripped binary is module precision.
   Commit the harness before the run, and do not edit it afterwards. The govulncheck pin is done
   (`c2f7219`).
3. **RESULTS.md text:**
   - the log sampling is verified;
   - SpiceDB's logs were 120–217 MB per round;
   - the 23k slip was fixed in `6d83040`;
   - SpiceDB's images carry unsigned provenance and an SPDX SBOM;
   - the any-rule verdict is computed in derive.
4. **Deviations.** Add the post-run edit to `vs-spicedb.yaml` and `toolbox.yaml`, the post-run
   harness edits, and the three list passes.
5. **The performance headline.** Qualify it with the cache defaults and the memory adapter. The
   cache probes above are a cheap way to show readers both configurations. A Postgres arm, which
   the README defers, is the run that would decide what OpenFGA's engine is worth.
6. **The box.** Let a comparative scorecard set declare its tally rule, so the Compare view can lead
   with the pre-registered verdict, and colour the hero by it.

## Sources

- This experiment at `6d83040`: [README.md](./README.md), [RESULTS.md](./RESULTS.md),
  [raw/](./raw), [facts.yaml](./facts.yaml), [run.sh](./run.sh), [gen.py](./gen.py),
  [facts.py](./facts.py), [snapshot.py](./snapshot.py), [load/](./load); the pre-registration at
  `774f45a`.
- [examples/openfga-portal/standards/vs-spicedb.yaml](../../examples/openfga-portal/standards/vs-spicedb.yaml),
  [comparisons/](../../examples/openfga-portal/comparisons),
  [toolbox.yaml](../../examples/openfga-portal/toolbox.yaml).
- [D01M857ZB468ZM2YS6GNSEG6YB9](../../docs/decisions/D01M857ZB468ZM2YS6GNSEG6YB9.md);
  `crates/toolbox-derive/src/lib.rs:755–766, 1224–1262`; `report-ui/src/views/compare.js` at
  `c21955f`; [dx-benchmarks.md](../../docs/design/dx-benchmarks.md) §2.
- OpenFGA at [v1.23.0](https://github.com/openfga/openfga/tree/v1.23.0) (`d67ba7b`): `README.md:226`,
  `cmd/run/run.go:525–545`, `pkg/server/config/config.go:43, 46, 893–900, 936–942`,
  `pkg/storage/memory/memory.go`, `pkg/logger/logger.go:178`, `go.mod`; the
  [release](https://github.com/openfga/openfga/releases/tag/v1.23.0) assets.
- [openfga/api@f153694](https://github.com/openfga/api/tree/f153694bfc20f7be303e33cabe72b668596c5a06):
  `openfga/v1/openfga_service.proto:1014, 1315`, `openfga_service_consistency.proto`;
  [openfga.dev@a13c70e](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/interacting/consistency.mdx)
  `docs-site/interacting/consistency.mdx` ("Future work").
- SpiceDB at [v1.57.1](https://github.com/authzed/spicedb/tree/v1.57.1) (`7074da7`): `README.md:279`,
  `pkg/cmd/serve.go:92, 146–158`, `pkg/cmd/util/util.go:366–374`, `.goreleaser.yml:11–28, 115–138`,
  `Dockerfile.release`, `.github/workflows/`, `go.mod:14`; the
  [release](https://github.com/authzed/spicedb/releases/tag/v1.57.1) assets; the image index and
  attestations of `ghcr.io/authzed/spicedb:v1.57.1` (index
  `sha256:9ea6df2d3cca23ed63aa7205639cf6babaaf1dc9a264ffb1f50c6455658fd6ba`).
- authzed-go v1.11.0: `proto/authzed/api/v1/permission_service.pb.go:1096–1114`,
  `schema_service.pb.go:123–127`, `core.pb.go:216–219`;
  [authzed/docs@ff2f067](https://github.com/authzed/docs/tree/ff2f067f53de6e691748a82a11ad764d52435f22):
  `app/spicedb/concepts/schema/page.mdx:38`, `app/spicedb/concepts/datastores/page.mdx:14–18`.
- `golang.org/x/vuln@v1.10.0/internal/vulncheck/binary.go:107–111, 178–206`;
  `go.uber.org/zap@v1.28.0/config.go` (`NewProductionConfig`, sampling 100 and 100).
- The GitHub REST API, read-only: contributors with `anon=1`, releases, org repositories, and
  attestations (404 for both SpiceDB image digests).
- Tools: govulncheck v1.10.0 (vuln DB 2026-11-24T15:56Z) in binary and source mode, slsa-verifier
  v2.7.1, cosign v2.6.1, gh 2.103.0, go1.27.3 and go1.27.4, and `target/debug/toolbox` (built
  22:24, before `c21955f`; its derive output is unaffected by that commit).
