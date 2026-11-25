# OpenFGA vs SpiceDB, for an OpenFGA portal

Is OpenFGA at least at parity with SpiceDB on the factors a team weighs when it picks a
Zanzibar-style authorization server? The factors are performance, footprint, supply chain,
modelling, operability and ecosystem. This experiment exists for the box, not for either tool:
it is the second comparison Toolbox's report shows, after
[backstage-compare](../backstage-compare/README.md). It checks that the comparative scorecards
(D01M857ZB468ZM2YS6GNSEG6YB9) and the report's Compare view work for a portal that is not
Toolbox's own ([design-loop-efficiency-2026-11-24.md](../../docs/audits/design-loop-efficiency-2026-11-24.md),
[design-audit.md](../../docs/design/design-audit.md)).

The portal that scores it is [examples/openfga-portal](../../examples/openfga-portal/README.md).
Its comparative scorecards are
[vs-spicedb.yaml](../../examples/openfga-portal/standards/vs-spicedb.yaml), and its product
records are in [examples/openfga-portal/comparisons/](../../examples/openfga-portal/comparisons).

No agent is in the scored path: each timed or counted fact comes from a script, and each yes/no
fact cites a primary source at the pinned tag.

## Pre-registration

The pre-registration was written and committed before any product was installed or measured.
The bars below are absolute. They are not fitted to either product, and they are not changed
after the run. A loss is published as written.

### Arms

| | OpenFGA | SpiceDB |
|---|---|---|
| Version | v1.23.0 release archive, `openfga_1.23.0_linux_amd64.tar.gz` | v1.57.1 release archive, `spicedb_1.57.1_linux_amd64.tar.gz` |
| Run as | `openfga run --datastore-engine memory`, defaults otherwise | `spicedb serve --datastore-engine memory --grpc-preshared-key <key>`, defaults otherwise |
| API under load | gRPC `openfga.v1.OpenFGAService` through `github.com/openfga/api` | gRPC `authzed.api.v1` through `github.com/authzed/authzed-go` |

Both run as shipped, with their own default caches, on one VM, loaded by one Go program at 16
concurrent workers. Memory is the datastore both ship as their default for trying them out. A
Postgres run is out of scope and is listed as a limitation.

### Corpus and model

[gen.py](./gen.py), seed 7: 1,000 users, 100 groups (each user in 2), 1,000 folders (each owned by
a group, one in ten nested under another folder), and 10,000 documents (each in a folder, with one
direct viewer and one owner; 1% public). The same model is written in each product's language
([model.fga](./model.fga), [schema.zed](./schema.zed)): a document is viewable by its direct
viewers, its owner, anyone when public, and whoever can view its folder; a folder is viewable by its
viewers, its owning group's members, and whoever can view its parent.

Before any timing, both arms answer the same 2,000 checks (a fixed sample of users and documents)
and list the documents of the same 50 users. The run stops unless both arms give the same answers
(`decisionsDigest`, `listsDigest`).

### Measures and bars

| Factor | Fact | Better | Bar for a pass |
|---|---|---|---|
| Performance | `readyS`: exec until the first check succeeds | lower | ≤ 5 s |
| | `writeTuplesPerS`: the corpus written 100 relationships per request, one writer | higher | ≥ 1,000/s |
| | `checkP95Ms`, `checkRps`: checks over the sample, 16 workers, 30 s after 5 s of warm-up | lower, higher | ≤ 10 ms, ≥ 2,000/s |
| | `listP95Ms`: every document one user can view (ListObjects, LookupResources), over the 50 users | lower | ≤ 100 ms |
| Footprint | `binaryMb`: the server binary on disk | lower | ≤ 100 MB |
| | `rssIdleMb`, `rssLoadedMb`: resident memory when ready, and after the corpus and the check load | lower | ≤ 100 MB, ≤ 500 MB |
| Supply chain | `goModules`: modules compiled into the binary (`go version -m`) | lower | ≤ 300 |
| | `knownVulns`: govulncheck `-mode=binary` findings in called code | lower | 0 |
| | `provenance`: the release archive has a signed build provenance a user can verify | pass | true |
| | `sbom`: the release publishes an SBOM | pass | true |
| Modelling | `conditions` (ABAC), `contextualTuples`, `wildcards`, `exclusion`, `listUsers`, `consistencyTokens`, `changeFeed`, `modelTests`, `multiTenant` | pass | true |
| Operability | `prometheusMetrics`, `otelTracing`, `httpByDefault`, `migrateCommand` | pass | true |
| | `datastores`: production datastores supported | higher | ≥ 3 |
| Ecosystem | `contributors` (GitHub, anonymous included) | higher | ≥ 100 |
| | `releasesLastYear`: GitHub releases in the year before the run | higher | ≥ 12 |
| | `officialSdks`: client SDKs the project maintains | higher | ≥ 5 |
| | `foundation`: governed by a neutral foundation | pass | true |

The yes/no facts are defined in [vs-spicedb.yaml](../../examples/openfga-portal/standards/vs-spicedb.yaml).
Each one is decided from the product's documentation or source at the pinned tag and cited in
`facts.yaml`. Where a probe against the running server can decide it, the probe decides it.

The performance and memory facts are the medians of 5 runs per arm, with the arms alternating.
The counted facts are single runs. A factor is *ahead* when OpenFGA wins more of its rules than it
loses, *parity* when it wins as many as it loses, and *behind* otherwise. Factors are tallied, not
weighted. An independent agent that did not run the harness recomputes every number from the raw
output and either upholds the verdict or rescores it in `ADJUDICATION.md`.

### Known biases

- The factors come from Toolbox's maintainers, the same people who own the portal that shows them.
- The modelling list was written from OpenFGA's feature names. Two of the facts, contextual
  tuples and multiple stores, are OpenFGA features, while consistency tokens and the change feed are
  SpiceDB strengths. The list may still favour OpenFGA.
- The memory datastore is no production setup, and both products are tuned for Postgres or
  CockroachDB in production.

## How to check it

```bash
cd experiments/openfga-spicedb-compare
./run.sh            # downloads both pinned releases, verifies checksums, runs 5 alternating rounds
python3 snapshot.py # writes examples/openfga-portal/comparisons/{openfga,spicedb}.yaml
```

Results are in [RESULTS.md](./RESULTS.md) once the run is done.

## Deviations

Appended after the run. No bar, arm, measure or tally rule changed. These clarify points the text
above could not meet as written.

- **Listen addresses.** "Defaults otherwise" holds except for listen addresses. The VM is shared,
  so in the timed rounds each server listens on 127.0.0.1 ports in the 18xxx range (OpenFGA
  `--grpc-addr`, `--http-addr`, `--metrics-addr`; SpiceDB `--grpc-addr`, `--metrics-addr`). The
  probes that decide `httpByDefault` and `prometheusMetrics` start each server with no flags but
  the ones in the Arms table, on its default ports.
- **"Before any timing".** The equivalence check needs the corpus written, so it can't come before
  `readyS` and `writeTuplesPerS` are timed. In each round it runs after the write and before the
  check and list timings, and the run stops unless every round of either arm gives the same
  digests.
- **"The first check succeeds".** A check needs a model. `readyS` covers exec, then creating a
  store and writing the model (OpenFGA) or writing the schema (SpiceDB), then the first successful
  check, polled every 10 ms.
- **Load on the shared VM.** Before each timed round the harness waits until the 1-minute load
  average is below 1.0, polling every 10 s and proceeding after 10 minutes. It records the load
  average at the start and end of each round.

Found by the [adjudication](./ADJUDICATION.md) (§1), and disclosed here after it:

- **Post-run edits to pre-registered files.** `6d83040` changed the performance scorecard's
  description and the portal's track text from "23k-relationship corpus" to "33k": descriptions
  only, no bar. After the adjudication, the vs-spicedb scorecards gained `verdict: majority`, which
  makes the report count factors by the tally above instead of its default
  (D01M87WF2QZVPTD8CGDZKY133CD). It encodes the pre-registered rule; no bar or rule changed.
- **Harness edits after the run.** `facts.py` and `load/main.go` were edited, and `load/loadgen`
  rebuilt, after the run ended (22:11:41) and before `6d83040` committed them. The committed
  harness is not proven to be the one that ran, though the round and facts files have exactly the
  fields it writes.
- **An unpinned tool.** `run.sh` installed govulncheck `@latest`. v1.10.0 ran, as recorded, and
  `c2f7219` pinned it.
- **Three list passes.** Each round lists every user's documents three times, which RESULTS.md
  describes and this README did not.
- **`sbom` was checked on the archive.** The definition above is "the release publishes an SBOM";
  facts.py looked only at release assets, so it missed the SPDX SBOM on SpiceDB's release images.
