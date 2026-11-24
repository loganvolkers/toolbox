# Results: OpenFGA v1.23.0 against SpiceDB v1.57.1

The experiment is about the box: it is the comparison Toolbox's report shows for
[examples/openfga-portal](../../examples/openfga-portal/README.md). The two products are the tools
measured. Method and bars: [README.md](./README.md), pre-registered before the run, with
clarifications under its *Deviations* heading. The run was on 2026-11-24 and took 1,774 s of wall
time: 5 rounds per arm, alternating, then the facts. Raw output is in [raw/](./raw). The product
records are
[comparisons/openfga.yaml](../../examples/openfga-portal/comparisons/openfga.yaml) and
[comparisons/spicedb.yaml](../../examples/openfga-portal/comparisons/spicedb.yaml).

## Conclusions

**Under the pre-registered tally, OpenFGA is ahead on four factors, at parity on one and behind on
one.** It is behind on performance, where SpiceDB answers checks about ten times faster and
writes about fifteen times faster on the memory datastore. It leads on footprint, supply chain,
modelling and ecosystem, and ties on operability.

| Factor | OpenFGA ahead | Parity | OpenFGA behind | OpenFGA is |
|---|---|---|---|---|
| Performance | 2 | 0 | 3 | **behind** |
| Footprint | 3 | 0 | 0 | **ahead** |
| Supply chain | 4 | 0 | 0 | **ahead** |
| Modelling | 2 | 6 | 1 | **ahead** |
| Operability | 1 | 3 | 1 | **parity** |
| Ecosystem | 3 | 0 | 1 | **ahead** |

Absolute bars, on 30 rules:

- **OpenFGA fails 4:** `check-latency` (p95 20.07 ms against ≤ 10 ms), `check-throughput` (1,473/s
  against ≥ 2,000/s), `consistency-tokens` and `datastores` (2 against ≥ 3).
- **SpiceDB fails 8:** `known-vulns` (7), `provenance`, `sbom`, `contextual-tuples`, `multi-tenant`,
  `http-by-default`, `contributors` (86 against ≥ 100) and `foundation`.
- Both pass every performance bar except OpenFGA's two check bars, and every footprint bar.

Per factor:

- **Performance (behind, 2–3).** OpenFGA starts faster (0.059 s against 0.141 s) and lists one user's
  documents faster (p95 3.24 ms against 9.07 ms). SpiceDB writes 49,399 relationships/s against
  3,160/s, and answers checks at p95 2.05 ms and 15,662/s against 20.07 ms and 1,473/s. Both run as
  shipped: SpiceDB's dispatch cache is on by default, and OpenFGA's check query cache is off by
  default (see *Limitations*).
- **Footprint (ahead, 3–0).** The binary is 67.4 MB against 82.3 MB, idle RSS 53.4 MB against
  84.5 MB, and RSS after load 101.5 MB against 306.4 MB.
- **Supply chain (ahead, 4–0).** OpenFGA has 142 Go modules against 190, and 0 govulncheck findings
  against 7. Only OpenFGA has verifiable SLSA provenance for the linux_amd64 archive, and only
  OpenFGA attaches an SBOM.
- **Modelling (ahead, 2–1, six ties).** Only OpenFGA has contextual tuples and multiple isolated
  stores on one server. Only SpiceDB has consistency tokens (ZedTokens). Both have conditions
  (SpiceDB calls them caveats), wildcards, exclusion and intersection, list-users (OpenFGA
  ListUsers, SpiceDB LookupSubjects), a change feed (ReadChanges, Watch) and model tests (`fga
  model test`, `zed validate`).
- **Operability (parity, 1–1, three ties).** OpenFGA serves HTTP/JSON by default and SpiceDB does
  not. SpiceDB supports 4 production datastores against OpenFGA's 2. Both expose Prometheus metrics
  by default, export OpenTelemetry traces and ship a migrate command.
- **Ecosystem (ahead, 3–1).** OpenFGA has 122 contributors against 86, 28 releases in the year
  against 15, and CNCF governance against vendor ownership. SpiceDB has one more official SDK (6
  against 5).

**The report shows a harsher headline than this tally.** Toolbox's Compare view uses its own verdict
rule: a factor is behind if OpenFGA is behind on any one of its rules (`toolbox-report.js`). Under
that rule only footprint and supply chain are ahead, and performance, modelling, operability and
ecosystem show as behind. The 30 per-rule standings are identical under both rules: `toolbox
report` on the portal agreed with `snapshot.py` on every rule. The verdict in this document is the
pre-registered one. The gap between the two rules is a finding about the box, not about either
product.

**Sensitivity.** Counting OpenFGA's SQLite engine, which its README labels beta, gives 3 datastores.
OpenFGA would then pass the `datastores` bar but stay behind SpiceDB's 4, so no verdict changes.
Excluding SpiceDB's MySQL (documented as "not recommended") gives 3, and nothing changes either.

## Every fact

Timed and memory facts are the median of 5 rounds, with min–max across rounds. Counted facts are
single runs. MB is 10⁶ bytes on disk; RSS is VmRSS from `/proc/<pid>/status` in MiB.

| Fact | OpenFGA | SpiceDB | Bar | OpenFGA is |
|---|---|---|---|---|
| `readyS` | 0.059 (0.048–0.063) | 0.141 (0.121–0.168) | ≤ 5 | ahead |
| `writeTuplesPerS` | 3,160 (2,950–3,166) | 49,399 (46,443–53,145) | ≥ 1,000 | behind |
| `checkP95Ms` | 20.07 (19.53–27.63) | 2.05 (1.86–5.45) | ≤ 10 | behind |
| `checkRps` | 1,473 (1,234–1,507) | 15,662 (9,850–16,482) | ≥ 2,000 | behind |
| `listP95Ms` | 3.24 (3.03–3.77) | 9.07 (8.04–10.69) | ≤ 100 | ahead |
| `binaryMb` | 67.4 | 82.3 | ≤ 100 | ahead |
| `rssIdleMb` | 53.4 (53.0–54.5) | 84.5 (84.2–84.8) | ≤ 100 | ahead |
| `rssLoadedMb` | 101.5 (100.7–102.3) | 306.4 (303.8–307.2) | ≤ 500 | ahead |
| `goModules` | 142 | 190 | ≤ 300 | ahead |
| `knownVulns` | 0 | 7 | 0 | ahead |
| `provenance` | true | false | true | ahead |
| `sbom` | true | false | true | ahead |
| `conditions` | true | true | true | parity |
| `contextualTuples` | true | false | true | ahead |
| `wildcards` | true | true | true | parity |
| `exclusion` | true | true | true | parity |
| `listUsers` | true | true | true | parity |
| `consistencyTokens` | false | true | true | behind |
| `changeFeed` | true | true | true | parity |
| `modelTests` | true | true | true | parity |
| `multiTenant` | true | false | true | ahead |
| `prometheusMetrics` | true | true | true | parity |
| `otelTracing` | true | true | true | parity |
| `httpByDefault` | true | false | true | ahead |
| `migrateCommand` | true | true | true | parity |
| `datastores` | 2 | 4 | ≥ 3 | behind |
| `contributors` | 122 | 86 | ≥ 100 | ahead |
| `releasesLastYear` | 28 | 15 | ≥ 12 | ahead |
| `officialSdks` | 5 | 6 | ≥ 5 | behind |
| `foundation` | true | false | true | ahead |

Supporting measures, not scored (median, min–max):

| Measure | OpenFGA | SpiceDB |
|---|---|---|
| check p50 ms | 10.20 (9.97–11.29) | 0.56 (0.55–0.79) |
| check p99 ms | 25.02 (24.37–40.08) | 14.45 (13.47–20.75) |
| list p50 ms | 2.26 (2.07–2.67) | 2.14 (1.70–5.61) |
| RSS right after the check load, MiB | 102.3 (101.2–102.6) | 306.4 (301.5–307.2) |
| equivalence pass (2,000 checks + 50 lists), s | 1.51 (1.44–2.51) | 2.26 (2.08–3.16) |
| check errors / answers that disagreed with the sample, all rounds | 0 / 0 | 0 / 0 |

`readyS` uses three decimals because both products are ready in well under a second.
`rssLoadedMb` is read at the end of the round, after the check load and the list timings.

### Counted facts: how

| Fact | Command | OpenFGA | SpiceDB |
|---|---|---|---|
| `binaryMb` | size of the extracted server binary | 67,387,552 B | 82,337,952 B |
| `goModules` | `go version -m bin/<server>`, `dep` lines | 142 (built with go1.27.4) | 190 (built with go1.27.3) |
| `knownVulns` | `govulncheck -mode=binary` (v1.10.0, DB updated 2026-11-24): distinct OSV ids with a vulnerable symbol in the binary | none | GO-2026-5932 (x/crypto/openpgp), GO-2026-6859 (net/http, fixed in go1.27.4), GO-2026-6860 and GO-2026-6866 (x/net/html), GO-2026-6865 (klauspost/compress), GO-2026-6974 and GO-2026-6975 (OpenTelemetry) |
| `provenance` | `gh attestation verify <archive> -R <repo>`; `slsa-verifier verify-artifact` (v2.7.1) against any `.intoto.jsonl` asset | gh: HTTP 404, no attestation; slsa-verifier: PASSED, builder slsa-github-generator generic v2.1.0 at commit d67ba7b; `cosign verify-blob` of `checksums.txt` with its `.sig`/`.pem`: Verified OK | gh: HTTP 404; no `.intoto.jsonl`, `.sig`, `.pem` or `.bundle` asset |
| `sbom` | release assets matching sbom/spdx/cyclonedx for the archive | `openfga_1.23.0_linux_amd64.tar.gz.sbom.json` | none |
| `contributors` | `gh api --paginate 'repos/<repo>/contributors?anon=1&per_page=100'` | 122 (8 anonymous) | 86 (2 anonymous) |
| `releasesLastYear` | releases published 2025-11-24T22:11Z to 2026-11-24T22:11Z | 28 (0 pre-releases) | 15 (0 pre-releases) |

The commands' full output is in [raw/facts.json](./raw/facts.json), and the govulncheck findings
are in [raw/govulncheck-spicedb.json](./raw/govulncheck-spicedb.json) and
[raw/govulncheck-openfga.json](./raw/govulncheck-openfga.json).

## Equivalence

Both arms, in every round, gave the same answers to the 2,000 checks (1,041 allowed) and the same
documents for the 50 users (223 to 443 each). The run would have stopped otherwise.

- `decisionsDigest` `a782efbc57133080f399d7783c9dad723e1a42c43383bb76ab713e2ef0743a8d`
- `listsDigest` `4831183fa7d30a6becf0ff50b0815ecff747fd50eab6ebbf85a59c67587c4d89`

A third opinion agreed: [gen.py](./gen.py) evaluates the model directly in Python, and both digests
equal its own (`oracleMatch: true` in every round file). The corpus is 33,200 relationships: 2,000
group memberships, 1,000 folder owners, 100 folder parents, 30,000 document relationships (folder,
viewer and owner) and 100 public wildcards.

## Environment

| | |
|---|---|
| Machine | 4 vCPU `Intel(R) Xeon(R) Processor`, 16 GB, Linux 6.12.113+; one VM shared with another agent's work |
| Load | Each round waited for a 1-minute load average below 1.0 (waits 60–380 s, never gave up). The load average was 0.84–0.98 at round starts and 2.4–3.7 at round ends, the rise being the round's own server and 16 load workers. |
| Load generator | [load/](./load) in Go 1.27.4, grpc-go 1.85.0, `github.com/openfga/api/proto` at the commit compiled into openfga v1.23.0, `github.com/authzed/authzed-go` v1.11.0 (the version compiled into spicedb v1.57.1); one gRPC connection per arm |
| Servers | `bin/openfga run --datastore-engine memory --grpc-addr 127.0.0.1:18081 --http-addr 127.0.0.1:18080 --metrics-addr 127.0.0.1:18112`; `bin/spicedb serve --datastore-engine memory --grpc-preshared-key <key> --grpc-addr 127.0.0.1:18051 --metrics-addr 127.0.0.1:18090` |
| Releases | Checksums verified against each release's `checksums.txt`; OpenFGA's `checksums.txt` signature verified with cosign v2.6.1 |
| Tools | govulncheck v1.10.0, slsa-verifier v2.7.1, cosign v2.6.1, gh 2.103.0, fga v0.9.0, zed v1.2.1 |
| Harness | `a411799+uncommitted`: the harness files were not yet committed when it ran |

How each round reads: exec the server, then poll a check every 10 ms until one succeeds (`readyS`)
and take RSS. Write the corpus in 100-relationship requests from one writer. Run the equivalence
at each product's strongest consistency (SpiceDB `fully_consistent`, OpenFGA `HIGHER_CONSISTENCY`).
Then 16 workers cycle the 2,000-check sample at default consistency for 5 s of warm-up and 30 s
measured; requests that start in the measured window count. Then list each of the 50 users'
documents three times, one request at a time, at default consistency (150 samples). Take RSS and
stop the server.

## Citations for the yes/no and declared facts

Each value comes from [facts.yaml](./facts.yaml). A probe decides it where one can, and facts.py
recorded no disagreement between a probe and facts.yaml. Probes ran against each server started
with its defaults: modelling probes over gRPC with the models in [probes/](./probes), HTTP and
metrics on the default ports, traces into a local OTLP/gRPC collector, the model tests with each
project's CLI, and migrate with `--help`.

| Fact | OpenFGA | Sources | SpiceDB | Sources |
|---|---|---|---|---|
| `conditions` | true, probe: Check with hour=10 true, hour=20 false | [proto](https://github.com/openfga/api/blob/f153694bfc20f7be303e33cabe72b668596c5a06/openfga/v1/openfga.proto#L88), [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/modeling/conditions.mdx) | true (caveats), probe: same | [proto](https://github.com/authzed/authzed-go/blob/v1.11.0/proto/authzed/api/v1/core.pb.go#L219), [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/concepts/caveats/page.mdx) |
| `contextualTuples` | true, probe: false, then true with a contextual tuple, then false again | [proto](https://github.com/openfga/api/blob/f153694bfc20f7be303e33cabe72b668596c5a06/openfga/v1/openfga_service.proto#L1014), [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/interacting/contextual-tuples.mdx) | false: no request field for request-scoped relationships in CheckPermission, LookupResources or LookupSubjects, and none in the source or CHANGELOG at v1.57.1; caveat context only feeds caveats on relationships already written | [proto](https://github.com/authzed/authzed-go/blob/v1.11.0/proto/authzed/api/v1/permission_service.pb.go#L1096), [CHANGELOG](https://github.com/authzed/spicedb/blob/v1.57.1/CHANGELOG.md) |
| `wildcards` | true, probe and corpus (`user:*`) | [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/modeling/public-access.mdx) | true, probe and corpus (`user:*`) | [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/concepts/schema/page.mdx#L128-L156) |
| `exclusion` | true, probe: `but not` and `and` | [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/configuration-language.mdx#L698) | true, probe: `-` and `&` | [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/concepts/schema/page.mdx#L206) |
| `listUsers` | true, probe: ListUsers gives `[user:carol]` | [proto](https://github.com/openfga/api/blob/f153694bfc20f7be303e33cabe72b668596c5a06/openfga/v1/openfga_service.proto#L942), [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/getting-started/perform-list-users.mdx) | true, probe: LookupSubjects gives `[user:carol]` | [proto](https://github.com/authzed/authzed-go/blob/v1.11.0/proto/authzed/api/v1/permission_service.pb.go#L1946), [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/concepts/querying-data/page.mdx#L87) |
| `consistencyTokens` | false: `WriteResponse` has no fields; reads take only MINIMIZE_LATENCY or HIGHER_CONSISTENCY; the docs list zookies as future work | [proto](https://github.com/openfga/api/blob/f153694bfc20f7be303e33cabe72b668596c5a06/openfga/v1/openfga_service.proto#L1315), [enum](https://github.com/openfga/api/blob/f153694bfc20f7be303e33cabe72b668596c5a06/openfga/v1/openfga_service_consistency.proto), [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/interacting/consistency.mdx#L60-L64) | true, probe: write returned a ZedToken, check `at_least_as_fresh` it | [proto](https://github.com/authzed/authzed-go/blob/v1.11.0/proto/authzed/api/v1/core.pb.go#L386), [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/concepts/consistency/page.mdx) |
| `changeFeed` | true, probe: ReadChanges from a continuation token got the next write | [proto](https://github.com/openfga/api/blob/f153694bfc20f7be303e33cabe72b668596c5a06/openfga/v1/openfga_service.proto#L787), [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/interacting/read-tuple-changes.mdx) | true, probe: Watch from a ZedToken streamed the next write | [proto](https://github.com/authzed/authzed-go/blob/v1.11.0/proto/authzed/api/v1/watch_service.pb.go#L83), [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/concepts/watch/page.mdx) |
| `modelTests` | true, probe: `fga model test --tests model.fga.yaml` exit 0, flipped assertions exit non-zero | [CLI](https://github.com/openfga/cli/blob/v0.9.0/README.md#L460), [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/modeling/testing.mdx) | true, probe: `zed validate schema.validation.yaml` exit 0, flipped assertions exit non-zero | [CLI](https://github.com/authzed/zed/blob/v1.2.1/internal/cmd/validate.go#L48), [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/modeling/validation-testing-debugging/page.mdx) |
| `multiTenant` | true, probe: two stores with different models on one server; a tuple in one is invisible in the other | [proto](https://github.com/openfga/api/blob/f153694bfc20f7be303e33cabe72b668596c5a06/openfga/v1/openfga_service.proto#L806), [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/concepts.mdx#L131-L136) | false: `WriteSchemaRequest` has only `schema`, one schema per datastore; the docs' answer is prefixed definitions in that one schema; isolated permission systems are AuthZed's managed offering | [proto](https://github.com/authzed/authzed-go/blob/v1.11.0/proto/authzed/api/v1/schema_service.pb.go#L123), [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/concepts/schema/page.mdx#L38), [managed](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/authzed/concepts/deployments/page.mdx) |
| `prometheusMetrics` | true, probe: `:2112/metrics` 200, 64 families | [config](https://github.com/openfga/openfga/blob/v1.23.0/pkg/server/config/config.go#L939-L941), [flag](https://github.com/openfga/openfga/blob/v1.23.0/cmd/run/run.go#L270) | true, probe: `:9090/metrics` 200, 185 families | [flag](https://github.com/authzed/spicedb/blob/v1.57.1/pkg/cmd/serve.go#L243), [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/ops/observability/page.mdx) |
| `otelTracing` | true, probe: `--trace-enabled` sent spans to the collector | [flag](https://github.com/openfga/openfga/blob/v1.23.0/cmd/run/run.go#L256), [docs](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/docs-site/getting-started/configure-telemetry.mdx) | true, probe: `--otel-provider otlpgrpc` sent spans to the collector | [flags](https://github.com/authzed/spicedb/blob/v1.57.1/pkg/cmd/server/otel.go#L51-L56) |
| `httpByDefault` | true, probe: `GET :8080/stores` 200 JSON | [config](https://github.com/openfga/openfga/blob/v1.23.0/pkg/server/config/config.go#L897-L898), [flag](https://github.com/openfga/openfga/blob/v1.23.0/cmd/run/run.go#L174) | false, probe: `:8443` refused; the gateway needs `--http-enabled` | [flag](https://github.com/authzed/spicedb/blob/v1.57.1/pkg/cmd/serve.go#L92) |
| `migrateCommand` | true, probe: `openfga migrate --help` | [source](https://github.com/openfga/openfga/blob/v1.23.0/cmd/migrate/migrate.go#L29) | true, probe: `spicedb datastore migrate --help` | [source](https://github.com/authzed/spicedb/blob/v1.57.1/pkg/cmd/migrate.go#L39), [group](https://github.com/authzed/spicedb/blob/v1.57.1/pkg/cmd/datastore.go#L23) |
| `datastores` | 2: PostgreSQL, MySQL (SQLite is beta per the README) | [README](https://github.com/openfga/openfga/blob/v1.23.0/README.md#L226), [switch](https://github.com/openfga/openfga/blob/v1.23.0/cmd/run/run.go#L530-L545) | 4: CockroachDB, Spanner, PostgreSQL, MySQL | [README](https://github.com/authzed/spicedb/blob/v1.57.1/README.md#L279), [docs](https://github.com/authzed/docs/blob/ff2f067f53de6e691748a82a11ad764d52435f22/app/spicedb/concepts/datastores/page.mdx#L14-L18) |
| `officialSdks` | 5: go, js, python, java, dotnet | [go](https://github.com/openfga/go-sdk), [js](https://github.com/openfga/js-sdk), [python](https://github.com/openfga/python-sdk), [java](https://github.com/openfga/java-sdk), [dotnet](https://github.com/openfga/dotnet-sdk) | 6: go, node, python, java, ruby, dotnet | [go](https://github.com/authzed/authzed-go), [node](https://github.com/authzed/authzed-node), [py](https://github.com/authzed/authzed-py), [java](https://github.com/authzed/authzed-java), [rb](https://github.com/authzed/authzed-rb), [dotnet](https://github.com/authzed/authzed-dotnet) |
| `foundation` | true: CNCF Incubation | [announcement](https://github.com/openfga/openfga.dev/blob/a13c70e9b21d0516b3fae7a729557d371c7ccaee/blog/incubation-announcement.md), [TOC vote](https://github.com/cncf/toc/issues/1287#issuecomment-3458442973) | false: "Copyright 2022 Authzed, Inc", CLA to AuthZed | [NOTICE](https://github.com/authzed/spicedb/blob/v1.57.1/NOTICE), [CONTRIBUTING](https://github.com/authzed/spicedb/blob/v1.57.1/CONTRIBUTING.md#L74-L78) |
| `provenance` | true: slsa-verifier PASSED | [workflow](https://github.com/openfga/openfga/blob/v1.23.0/.github/workflows/release.yaml#L140-L176), [release](https://github.com/openfga/openfga/releases/tag/v1.23.0) | false: no asset, no attestation, no signing step | [GoReleaser](https://github.com/authzed/spicedb/blob/v1.57.1/.goreleaser.yml), [workflow](https://github.com/authzed/spicedb/blob/v1.57.1/.github/workflows/release.yaml), [release](https://github.com/authzed/spicedb/releases/tag/v1.57.1) |
| `sbom` | true: `.sbom.json` per archive | [GoReleaser](https://github.com/openfga/openfga/blob/v1.23.0/.goreleaser.yaml#L133-L135), [release](https://github.com/openfga/openfga/releases/tag/v1.23.0) | false: no SBOM asset, no `sboms` section | [GoReleaser](https://github.com/authzed/spicedb/blob/v1.57.1/.goreleaser.yml), [release](https://github.com/authzed/spicedb/releases/tag/v1.57.1) |

Docs repositories are not tagged per release, so docs links are pinned to the commit read on the
run date (openfga/openfga.dev `a13c70e`, authzed/docs `ff2f067`). API links are pinned to the proto
versions compiled into each server binary.

## Limitations

- **Memory datastore only.** Neither product is tuned for it, and production runs on PostgreSQL,
  MySQL, CockroachDB or Spanner. The write and check numbers say nothing about those datastores.
- **Caches as shipped decide the check result.** SpiceDB's dispatch cache is on by default.
  OpenFGA's check query cache and check iterator cache are off by default (`CheckQueryCache.Enabled:
  false` in its startup config). The sample cycles 2,000 checks, so a cache that is on serves most
  of them. OpenFGA with `--check-query-cache-enabled` was not measured; the bar is about the
  defaults, so it would be a different experiment.
- **One shared 4-vCPU VM.** The load generator, the server and another agent's processes shared the
  CPUs. The load-average gate kept round starts below 1.0, but nothing isolated the cores, and the
  spread in SpiceDB's check p95 (1.86–5.45 ms) likely reflects that. Medians of 5 runs damp it; they
  do not remove it.
- **Defaults include costs that are not the product's core.** Both servers log requests at info
  level to the harness's log file. SpiceDB logged every request (about 140 MB per round). OpenFGA
  logged a fraction of them (4,188 Check lines for more than 46,000 checks in round 1), which is
  consistent with log sampling (unverified). SpiceDB's `readyS` includes its startup release check
  against GitHub, about 60 ms in its logs, and it schedules telemetry to telemetry.authzed.com by
  default. All of these are counted, because the bar is the default install.
- **`knownVulns` depends on the scan date and the build.** SpiceDB v1.57.1 is ten days older than
  OpenFGA v1.23.0, and it was built with go1.27.3, while GO-2026-6859 is fixed in go1.27.4, which
  OpenFGA was built with. Binary mode reports vulnerable symbols linked into the binary; it cannot
  prove a call path the way source mode can.
- **`provenance` is about the archive.** SpiceDB's container images may carry build attestations;
  they were not checked, because the arm is the linux_amd64 archive.
- **Counts depend on definitions.** `datastores` (SQLite beta excluded, MySQL "not recommended"
  included; see *Sensitivity*), `officialSdks` (repositories the org describes as official client
  libraries) and `contributors` (GitHub's contributors API, which counts commit authors on the
  default branch) all rest on stated choices. They are recorded in facts.yaml.
- **The modelling list.** As the pre-registration says, contextual tuples and multiple stores are
  OpenFGA features, and consistency tokens and the change feed are SpiceDB strengths. Six of the
  nine modelling facts are ties.
- **The corpus is 33,200 relationships, not 23k.** The counts the README pre-registered give 33,200.
  The vs-spicedb scorecard's description and the portal's track text say "23k-relationship corpus",
  an arithmetic slip in those descriptions, which this experiment does not own. The records report
  `corpus: 33200`.
- **One machine, one corpus, one model shape.** Deeper nesting, larger fan-out or many more users
  could change any timed result.
- **Adjudication is pending.** The README calls for an agent that did not run the harness to
  recompute every number from `raw/` and uphold or rescore the verdict in `ADJUDICATION.md`. That
  has not happened yet.
