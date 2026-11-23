# Toolbox vs Backstage, beyond DX

Is Toolbox at least at parity with Backstage on the factors a platform team weighs besides
developer experience? The factors are performance, footprint, security, supply chain, change
safety, fit to the organisation, the agent surface and capabilities. This is about the box
(running, pinning, reporting, distributing), not a tool. The design and its grounding are in
[backstage-comparison.md](../../docs/design/backstage-comparison.md); the research brief is
[RESEARCH.md](./RESEARCH.md).

No agent is in the scored path: every measure is a script. The comparative scorecards in
[portal/standards/backstage-comparison.yaml](../../portal/standards/backstage-comparison.yaml)
apply the parity rule below (D01M857ZB468ZM2YS6GNSEG6YB9), and the Toolbox for Toolbox report shows
them under *Compare*.

## Arms

| | Toolbox | Backstage |
|---|---|---|
| Version | this repository at the frozen commit (release build); install facts from release v0.1.1 | `@backstage/create-app` 0.9.4 (`backstage.json` 1.57.0: `backend-defaults` 0.19.0, `plugin-catalog-backend` 4.0.5) on Node 24.22.3 |
| Run as | `toolbox serve portal/ --database-url …` with token auth, a PAT for the client and the OpenFGA model in [portal/](./portal) checking every read | `NODE_ENV=production node packages/backend` with the template's configs plus [app-config.bench.yaml](./backstage/app-config.bench.yaml): Postgres, a static `externalAccess` token, the catalog file |
| Authorization | OpenFGA 1.22.1, Postgres datastore reading the derived tuples | the template's `allow-all` permission policy |
| Catalog | the same [gen.py](./gen.py) corpus, posted to `/v1/ingest` | the same corpus as a `file` location |

Both run on one VM (4 vCPU Intel Xeon, Linux 6.12), against one Postgres 16, loaded by oha 1.10.0
at 16 connections. The Backstage install is built as its Dockerfile does: `yarn build:backend`,
then `yarn workspaces focus --all --production` in the unpacked bundle.

## Measures and bars

Each fact is one field of a `product` record in [portal/comparisons/](../../portal/comparisons).
A rule passes or fails a product on an absolute bar, and compares the two on a value where there is
one.

| Factor | Fact (script) | Better | Bar for a pass |
|---|---|---|---|
| Performance | `readyS`: exec → ready (measure.py) | lower | ≤ 10 s |
| | `catalogReadableS`: exec → every component readable | lower | ≤ 60 s |
| | `getP95Ms`, `getRps`: one entity by id | lower, higher | ≤ 50 ms, ≥ 1,000/s |
| | `listP95Ms`, `listRps`: 100 components of one owner | lower, higher | ≤ 500 ms, ≥ 20/s |
| | `recoveredS`: `kill -9`, restart, every component readable | lower | ≤ 60 s |
| Footprint | `rssIdleMb`, `rssLoadedMb`: process tree resident memory (measure.py) | lower | ≤ 512 MB, ≤ 1,024 MB |
| | `installMb`: bytes on disk to run (footprint.py) | lower | ≤ 1,000 MB |
| | `adopterFiles`, `adopterLines`: what the adopter keeps for a first portal | lower | — |
| Security | `anonymousReadRefused`, `forgedIdentityRefused`, `anonymousMcpRefused`, `clientCannotDelete` (surface.py) | pass | each true |
| | `loopbackByDefault`, `ownerOnlyFields`, `extensionProcessIsolated`, `extensionCredentialIsolated` (facts.yaml, with evidence) | pass | each true |
| Supply chain | `thirdPartyPackages`, `knownVulns`, `highVulns` (footprint.py; osv-scanner 2.2.4) | lower | high = 0 |
| | `provenancePercent`: release artifacts with verifiable provenance | higher | 100 |
| Change safety | `upgradeLinesPerRelease` (upgrade.py) | lower | — |
| | `rollbackManualStepsPerRelease`: steps that fail rolling back by binary; for Backstage, its documented `knex migrate:down` per migration its template's plugins added (upgrade.py) | lower | 0 |
| | `offlineValidate`, `changePreview` (facts.yaml) | pass | true |
| Org fit | `repoScopedWithoutServer`, `federates` (facts.yaml) | pass | true |
| Agent surface | `mcpByDefault` (facts.yaml), `mcpToolsListBytes`, `mcpGetEntityBytes` (surface.py) | pass, lower, lower | true, ≤ 32 KB, ≤ 4 KB |
| Capabilities | twelve scenarios and extension cost (facts.yaml), `contributors` | pass, higher | true |

Performance and footprint are the medians of 5 runs per arm per size, with the arms alternating.
The scorecards use the 10k corpus; 1k is published beside it. Single runs are enough for the probes
and static facts, which don't vary.

**Parity rule (pre-registered).** On a rule with values, Toolbox is *ahead* or *behind* when the
medians differ by more than 10% of the larger, and at *parity* otherwise. On a rule without values,
the outcomes decide: pass beats fail. A factor is *behind* when any rule is behind, and Toolbox is at
parity on a factor when no rule is behind. A missing value is *unknown*, never parity.

**Before the scored run.** Smoke rounds on this harness found four defects in Toolbox, fixed in
their own commits before the run was frozen. Each names a use outside the harness: any portal with
an OpenFGA model.

- With authorization on, a filtered list asked OpenFGA about every candidate, even those the
  filter could not match.
- OpenFGA scanned every derived tuple for each read of a check.
- A list page read all candidates even once it was full.
- Turning on OpenFGA's TTL caches hid a new entity read through `min_snapshot` for 10 s. They were
  replaced by decisions kept per snapshot.

The smoke rounds also showed that the security probes and the agent surface needed no change.

## Run

```bash
python3 gen.py 1000 /tmp/bc/n1000 && python3 gen.py 10000 /tmp/bc/n10000
OHA=/path/to/oha TOOLBOX_OPENFGA_BIN=/path/to/openfga ./run.sh   # raw results in /tmp/bc/r
python3 snapshot.py /tmp/bc/r                                    # writes portal/comparisons/*.yaml
```

`BENCH_PG`, `BS_APP`, `BS_SRC`, `NODE_BIN`, `TOOLS` and `BS_REPO` (a `backstage/backstage` clone,
for the template diff) point the scripts at their inputs.

## Threats to validity

- The Backstage arm is configured by Toolbox's maintainers. Its configuration is published here so a
  Backstage maintainer can review it.
- One VM, one replica each: nothing here says how either scales out.
- Backstage's `allow-all` policy makes no authorization calls on a read; Toolbox checks every read
  in OpenFGA. A Backstage policy that checks ownership would make it slower, not faster.
- Toolbox's upgrade and rollback rest on two releases; Backstage's template diff covers six.
- The capability scenarios are drawn from the Backstage parity audit (D47), so they favour what
  Backstage ships, not what Toolbox does.

## Results

[RESULTS.md](./RESULTS.md).
