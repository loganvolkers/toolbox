# Toolbox against Backstage, beyond DX

*Status: v1 design and round 3 results, 2026-11-23 (D01M857ZB468ZM2YS6GNSEG6YB9,
D01M857ZB3FZJZWX8G3BF5W1RNR). About the box (running, pinning, reporting, distributing), not a
tool: it says what the box must do as well as Backstage, and records a measured comparison in the
box's own scorecards. The harness is [experiments/backstage-compare](../../experiments/backstage-compare/README.md).
This is the top-down end of the focus (D01M80Y0XEK069GD08VCCNF0WB8): the central portal against
its strongest rival.*

---

## 1. Conclusion

**Toolbox is ahead of Backstage on performance, footprint, security (owner-only fields and
connectors as separate processes; the other secure defaults are at parity, and neither keeps an
extension from the server's credentials), supply chain, change safety, repo
scope, the agent surface and extensibility. It is at parity on federation (neither has it), and
behind on capabilities and community.** Each claim is a rule in a comparative scorecard
(`portal/standards/backstage-comparison.yaml`), scored on a product record written by a script
from a measured run (`portal/comparisons/`). The Toolbox for Toolbox report shows it under
*Compare*.

The DX Benchmarks ([dx-benchmarks.md](./dx-benchmarks.md)) prove the agent claims; this proves the
rest. A platform team weighs fourteen factors (§2), and DX is one of them.

Getting there took eight product fixes, each with a use outside the benchmark (§4). Four were
security: an image off loopback trusted an identity header; in-image connectors inherited the
server's database credentials; a same-user connector could read the server's environment through
`/proc`; and OpenFGA took the datastore URI on its command line and answered anyone. Four were read
performance with authorization on. The first round's loss on filtered lists at 10k entities is
published as written, with the fix and the rerun. So is a harness defect: Backstage's lists matched
nothing in rounds 1 and 2, which round 3 caught with an equivalence gate
([RESULTS.md](../../experiments/backstage-compare/RESULTS.md)). Both products listen on every
interface as shipped. And the round 3 adjudication found that a same-user connector still reads
OpenFGA's memory on a host without Yama, so credential isolation fails for both products.

## 2. Factors

Grounded in ISO/IEC 25010:2023, which names nine quality characteristics; all apply except Safety.
Each factor names the software-design or engineering-management argument for why Toolbox should be
better, and the parity rule that could falsify it. The full brief, with sources and procedures, is
[RESEARCH.md](../../experiments/backstage-compare/RESEARCH.md).

| # | Factor | ISO 25010 / framework | Why Toolbox should be better | Scorecard |
|---|---|---|---|---|
| F1 | Functional completeness | Functional suitability | It shouldn't: Backstage ships TechDocs, templates, search and 160+ plugins | `vs-backstage-capabilities` |
| F2 | Installability | Flexibility: installability | One binary or image, configured in code (charter goal 1) | `vs-backstage-footprint` (install, adopter files) |
| F3 | Footprint and cold start | Performance efficiency: resource use. USE method (Gregg) | Rust, no Node runtime, derived state computed once | `vs-backstage-footprint`, `vs-backstage-performance` |
| F4 | Read latency and capacity | Time behaviour, capacity. Tail at Scale (Dean & Barroso) | Reads served from an in-memory snapshot | `vs-backstage-performance` |
| F5 | Freshness | Functional correctness | Push ingest, full recompute in seconds | `vs-backstage-performance` (catalog readable) |
| F6 | Secure defaults | Security. CISA Secure by Design, ASVS 5.0 V13 | One enforcement point; deny by default | `vs-backstage-security` |
| F7 | Extension isolation | Security: integrity; modularity. Parnas | Connectors are processes with their own environment | `vs-backstage-security` |
| F8 | Supply chain | Security: resistance. SLSA, SSDF PS/PW | One binary, signed attestations | `vs-backstage-supply-chain` |
| F9 | Upgrade cost | Maintainability: modifiability. Lehman I/II; the Churn Rule (SWE at Google) | The adopter keeps a manifest, not an app | `vs-backstage-change-safety` |
| F10 | Analysability | Analysability, testability. DORA change fail rate; the Beyoncé Rule | Validate, test and plan offline | `vs-backstage-change-safety` |
| F11 | Extensibility | Modularity, adaptability. Ousterhout's deep modules | Connectors speak a protocol in any language, no core rebuild | `vs-backstage-extensibility` |
| F12 | Org fit: repo scope, federation | Compatibility. Conway; Team Topologies TVP; DORA loosely coupled teams | Repo portals need no server; federation designed (D68) | `vs-backstage-org-fit` |
| F13 | Agent surface | Interaction capability. SPACE efficiency | MCP by default with small, typed tools | `vs-backstage-agent-surface` |
| F14 | Recoverability | Reliability: recoverability. DORA recovery time; SSDF 1.2 PS.4 | State is derived, so recovery is a recompute and rollback is an image | `vs-backstage-performance` (recovery), `vs-backstage-change-safety` (rollback) |

DORA's warning applies: no single metric, and none set as a goal. So each factor is its own
scorecard, and the overall standing per factor is a verdict, not a sum.

## 3. Comparative scorecards

A comparison is a scorecard that names a subject and its peers (D01M857ZB468ZM2YS6GNSEG6YB9):

```yaml
- name: vs-backstage-performance
  compare: {subject: product:toolbox, peers: [product:backstage]}
  rules:
    - name: list-throughput
      check: entity.listRps >= 20.0
      value: entity.listRps
      better: higher
      unit: req/s
      tolerance: 0.1
```

- The `check` is an absolute bar, set before the run. A product passes or fails it like any entity.
- The `value` compares the subject with each peer. Within `tolerance` of the larger value is
  parity; otherwise the better side by `better` is ahead. With no value, outcomes decide.
- A factor is behind when any rule is behind, unknown when any rule is unknown, ahead when any rule
  is ahead, and at parity otherwise. A rule with no value on either side and no outcome is unknown,
  never parity.
- Peers are left out of the ratchet: a rival scoring worse is not this portal's improvement.

The products are `Product` records in `portal/comparisons/`, written by
`experiments/backstage-compare/snapshot.py` from the raw results, with the harness commit, the
corpus and the run count. A comparison is re-run, not hand-edited. The meta standards hold these
scorecards to the same bar as any other: each rule is tested on fixtures that pass and fail it,
and names its fix.

## 4. What changed in Toolbox to reach parity

| Fix | Factor | Use outside the benchmark |
|---|---|---|
| Off loopback, `serve` defaults to token auth; no trusted header (D01M857ZB3FZJZWX8G3BF5W1RNR) | F6 | every image deployment, every `serve --listen 0.0.0.0` |
| In-image connectors start from an allowlisted environment | F7 | every portal that runs a connector next to `DATABASE_URL` |
| The server is non-dumpable, so a same-user process cannot read its environment or memory | F7 | every deployment with supervised connectors |
| OpenFGA gets no server credentials: its datastore URI is in a private file removed once it is healthy, and every call needs a per-start preshared key | F7 | every deployment with an OpenFGA model and supervised connectors |
| After round 3: each in-image connector starts in its own Landlock domain, so it cannot read OpenFGA's or another connector's memory; `serve` warns when the database URL is on argv. Not yet scored: the fact stays `false` until a round reruns the canary on the image | F7 | every deployment with in-image connectors; every `serve --database-url` |
| A list filter drops candidates it rejects in both shapes before asking OpenFGA | F4 | any portal with an OpenFGA model and filtered lists |
| OpenFGA decisions kept per snapshot, not in a TTL cache | F4 | every authorized read; no stale read after `min_snapshot` |
| Indexes on the columns OpenFGA's tuple view computes | F4 | every check against the Postgres datastore |
| A full list page stops reading candidates; a kind with no owner-only fields is filtered once | F4 | every list |
| A filter's required `entity.<field> == '<value>'` comparisons narrow candidates through a per-snapshot index | F4 | the MCP `query` tool, `toolbox query`, the UI catalog filter |

## 5. Results

See [RESULTS.md](../../experiments/backstage-compare/RESULTS.md) for the tables, spreads, the
first round's loss and what is not yet measured.

## 6. Not measured yet

- F2 wall time from a clean VM to a first portal (needs a pinned clean image).
- F5 commit-to-visible freshness through the pull modes.
- F11 the reference extension's lines of code.
- F13 the 20-question call count.
- F12 federation, until D68 is built. Neither product federates today, so it is parity, not a win.

## Sources

The source table is in [RESEARCH.md §3](../../experiments/backstage-compare/RESEARCH.md#3-sources).
The primary ones are ISO/IEC 25010:2023 (abstract and foreword), dora.dev (metrics, platform
engineering, loosely coupled teams), the CNCF TAG App Delivery platforms white paper and maturity
model, Parnas (1972), Conway (1968), Lehman (1980), *Software Engineering at Google* ch. 1, Dean &
Barroso (2013), Gregg's USE method, SLSA v1.2, NIST SP 800-218, CISA Secure by Design, OWASP ASVS
5.0 V13, and the `backstage/backstage` repository at `v1.57.1` (`9342a5de`).
