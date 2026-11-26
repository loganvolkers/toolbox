# Seikika vs Backstage, beyond DX

Is Seikika at least at parity with Backstage on the factors a platform team weighs besides
developer experience? The factors are performance, footprint, security, supply chain, change
safety, fit to the organisation, the agent surface and capabilities. This is about the box
(running, pinning, reporting, distributing), not a tool. The design and its grounding are in
[backstage-comparison.md](../../docs/design/backstage-comparison.md); the research brief is
[RESEARCH.md](./RESEARCH.md).

No agent is in the scored path: every measure is a script. The comparative scorecards in
[portal/standards/backstage-comparison.yaml](../../portal/standards/backstage-comparison.yaml)
apply the parity rule below (D01M857ZB468ZM2YS6GNSEG6YB9), and the Seikika for Seikika report shows
them under *Compare*.

## Arms

| | Seikika | Backstage |
|---|---|---|
| Version | this repository at the frozen commit (release build); install facts from release v0.1.1 | `@backstage/create-app` 0.9.4 (`backstage.json` 1.57.0: `backend-defaults` 0.19.0, `plugin-catalog-backend` 4.0.5) on Node 24.22.3 |
| Run as | `seikika serve portal/ --database-url …` with token auth, a PAT for the client and the OpenFGA model in [portal/](./portal) authorizing every read (each decision checked once per snapshot, then cached); `RUST_LOG=warn` | `NODE_ENV=production LOG_LEVEL=warn node packages/backend` with `app-config.yaml`, `app-config.production.yaml` and [app-config.bench.yaml](./backstage/app-config.bench.yaml): Postgres, the catalog file, a static `externalAccess` token restricted to catalog reads for the client, and an unrestricted one for MCP, because Backstage does not delegate a permission-restricted token to the plugins MCP calls (a plugin-scoped one works, and can delete) |
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
| | `getP95Ms`, `getRps`: one entity by id, over 1,000 ids drawn with a fixed seed | lower, higher | ≤ 50 ms, ≥ 1,000/s |
| | `listP95Ms`, `listRps`: the first 100 components of one owner, over all 50 owners | lower, higher | ≤ 500 ms, ≥ 20/s |
| | `recoveredS`: `kill -9`, restart, every component readable | lower | ≤ 60 s |
| Footprint | `rssIdleMb`, `rssLoadedMb`: process tree resident memory (measure.py) | lower | ≤ 512 MB, ≤ 1,024 MB |
| | `installMb`: bytes on disk to run (footprint.py) | lower | ≤ 1,000 MB |
| | `adopterFiles`, `adopterLines`: what the adopter keeps for a first portal | lower | — |
| Security | `anonymousReadRefused`, `forgedIdentityRefused`, `anonymousMcpRefused`, `clientCannotDelete` (surface.py) | pass | each true |
| | `loopbackByDefault`, `ownerOnlyFields`, `extensionProcessIsolated`, `extensionCredentialIsolated` (facts.yaml, with evidence) | pass | each true |
| Supply chain | `thirdPartyPackages`, `knownVulns`, `highVulns` over everything each ships: Seikika's `Cargo.lock`, the OpenFGA binary's Go modules and the UI's `package-lock.json`; Backstage's `yarn.lock` (footprint.py; osv-scanner 2.2.4) | lower | high = 0 |
| | `provenancePercent`: release artifacts with verifiable provenance | higher | 100 |
| Change safety | `upgradeLinesPerRelease` (upgrade.py) | lower | — |
| | `rollbackManualStepsPerRelease`: steps that fail rolling back by binary; for Backstage, its documented `knex migrate:down` per migration its template's plugins added (upgrade.py) | lower | 0 |
| | `offlineValidate`, `changePreview` (facts.yaml) | pass | true |
| Org fit | `repoScopedWithoutServer`, `federates` (facts.yaml) | pass | true |
| Agent surface | `mcpByDefault` (facts.yaml), `mcpToolsListBytes`, `mcpGetEntityBytes` (surface.py) | pass, lower, lower | true, ≤ 32 KB, ≤ 4 KB |
| Capabilities | twelve scenarios and extension cost (facts.yaml), `contributors` | pass, higher | true |

Before any timing, measure.py pages through every owner's components on both arms and stops unless
the owner and component pairs equal the corpus's (`ownersDigest`), so both arms are known to answer
the same reads. Performance and footprint are the medians of 5 runs per arm per size, with the arms
alternating.
The scorecards use the 10k corpus; 1k is published beside it. Single runs are enough for the probes
and static facts, which don't vary.

**Parity rule (pre-registered).** On a rule with values, Seikika is *ahead* or *behind* when the
medians differ by more than 10% of the larger, and at *parity* otherwise. On a rule without values,
the outcomes decide: pass beats fail. A factor is *behind* when any rule is behind, and Seikika is at
parity on a factor when no rule is behind. When a value is missing on either side, the outcomes
decide, as D01M857ZB468ZM2YS6GNSEG6YB9 says; a rule with neither values nor outcomes is *unknown*,
never parity. Rounds 1 to 3 froze an earlier sentence here, which made any missing value unknown;
no rule in those rounds lacked a value, so the two readings score them the same.

**Before the scored run.** Four Seikika commits made for this comparison predate the first
freeze (`dfeeea4`): token auth off loopback (`3cf4dba`, D01M857ZB3FZJZWX8G3BF5W1RNR), the connector
environment allowlist (`e06d6a7`), comparative scorecards (`05c947e`) and evaluating a list
filter once when the kind has no owner-only fields (`5ef6263`). Smoke rounds on this harness found four defects in Seikika, fixed in their own
commits before the run was frozen. Each names a use outside the harness: any portal with an OpenFGA
model.

- With authorization on, a filtered list asked OpenFGA about every candidate, even those the
  filter could not match.
- OpenFGA scanned every derived tuple for each read of a check.
- A list page read all candidates even once it was full.
- Turning on OpenFGA's TTL caches hid a new entity read through `min_snapshot` for 10 s. They were
  replaced by decisions kept per snapshot (`a62d92f`).

**Round 3.** The adjudication of rounds 1 and 2 ([ADJUDICATION.md](./ADJUDICATION.md)) led to a
second freeze. Its harness changes: the equivalence gate; reads spread over 1,000 ids and 50 owners
instead of one URL each; the Backstage list filter, which matched no entity in rounds 1 and 2;
`app-config.production.yaml` and equal log levels; the read-only client token; supply chain over
OpenFGA and the UI too; and facts judged as each product ships. Its Seikika changes: for any
deployment with supervised connectors, the server is non-dumpable (`71d8655`), and OpenFGA gets no
server credentials and requires a per-start key (`26b77f3`); for any filtered list, index
narrowing applies only when `kind` names exactly one kind, and never on `id` (`5968a55`).

## Run

```bash
python3 gen.py 1000 /tmp/bc/n1000 && python3 gen.py 10000 /tmp/bc/n10000
OHA=/path/to/oha SEIKIKA_OPENFGA_BIN=/path/to/openfga ./run.sh   # raw results in /tmp/bc/r
python3 snapshot.py /tmp/bc/r                                    # writes portal/comparisons/*.yaml
```

`BENCH_PG`, `BS_APP`, `BS_SRC`, `NODE_BIN`, `TOOLS` and `BS_REPO` (a `backstage/backstage` clone,
for the template diff) point the scripts at their inputs.

## Threats to validity

- The Backstage arm is configured by Seikika's maintainers. Its configuration is published here so a
  Backstage maintainer can review it.
- One VM, one replica each: nothing here says how either scales out.
- Backstage's `allow-all` policy makes no authorization calls on a read; Seikika checks each
  decision in OpenFGA once per snapshot and then serves it from a cache, so spread reads over 1,000
  ids are mostly cache hits. A Backstage policy that checks ownership would make it slower, not
  faster.
- `client-cannot-delete` is scored on Backstage's read-only REST token. Its MCP token, which cannot
  be read-only, can delete; Seikika uses one token for both.
- The harness passes Seikika's database URL with `--database-url`, which every local user can read
  in `/proc/<pid>/cmdline`. The image uses the environment instead.
- Seikika's upgrade and rollback rest on two releases; Backstage's template diff covers six.
- The capability scenarios are drawn from the Backstage parity audit (D47), so they favour what
  Backstage ships, not what Seikika does.
- oha runs on the same 4 vCPUs as the servers, with no CPU pinning, and loads each read for 10 s in
  a closed loop at 16 connections; the brief asked for 300 s open-loop. Throughput is a ceiling
  under contention, not a capacity.
- The equivalence gate compares owner and component pairs, not relations or whole entities.
- The security probes run on the release binary listening on every interface, not on the image.
- Upgrade cost is unlike: Seikika's is required config lines between its releases, Backstage's is
  the template diff its upgrade helper shows.
- `facts.yaml` decides 24 of the 48 rules; each fact carries its evidence for review.

## Results

[RESULTS.md](./RESULTS.md).
