# Results: Toolbox against Backstage, beyond DX

*Round 3 ran 2026-11-23 on the harness frozen at `e66526c`, with the scorecards frozen at `581be50`,
and measured Toolbox at `581be50`. Rounds 1 and 2 ran on the harness frozen at `dfeeea4`. The
adjudication of rounds 1 and 2 is [ADJUDICATION.md](./ADJUDICATION.md); round 3 answers it. Raw
results for every round are in [results/](./results); the product records the scorecards read are
[portal/comparisons/](../../portal/comparisons). The adjudication of round 3 is
[ADJUDICATION.md § Round 3](./ADJUDICATION.md#round-3). It upholds the factor verdict and fails one
rule, credential isolation, which this write-up and the records now score as it found.*

## Verdict

**After round 3, Toolbox is ahead of Backstage on seven of the eight pre-registered factors and
behind on one, capabilities.** Extensibility, which the README counts inside capabilities, is
scored as a ninth scorecard (a deviation, below); Toolbox is ahead on it. This is the Compare view
of the Toolbox for Toolbox report:

| Factor (scorecard) | Standing | Rules ahead / parity / behind |
|---|---|---|
| Performance (`vs-backstage-performance`) | **ahead** | 7 / 0 / 0 |
| Footprint (`vs-backstage-footprint`) | **ahead** | 5 / 0 / 0 |
| Security (`vs-backstage-security`) | **ahead** | 2 / 6 / 0 |
| Supply chain (`vs-backstage-supply-chain`) | **ahead** | 4 / 0 / 0 |
| Change safety (`vs-backstage-change-safety`) | **ahead** | 4 / 0 / 0 |
| Org fit (`vs-backstage-org-fit`) | **ahead** | 1 / 1 / 0 (federation: neither) |
| Agent surface (`vs-backstage-agent-surface`) | **ahead** | 2 / 1 / 0 |
| Extensibility (`vs-backstage-extensibility`) | **ahead** | 2 / 0 / 0 |
| Capabilities (`vs-backstage-capabilities`) | **behind** | 0 / 6 / 7 |

The capabilities Toolbox lacks are docs search, docs, templates, API docs, Kubernetes,
notifications and community (4 contributors against 2,826). Templates are self-service actions,
which are paused (D64). Toolbox's own bars fail in five places: five known vulnerabilities, no
federation, listening on every interface in its image, a connector that can read OpenFGA's
credentials (below), and the capabilities above. Each is published as a loss, not waived.
Security stays ahead on owner-only fields and process isolation.

**What changed since round 2.** Three corrections from the adjudication, each against Toolbox,
and one harness defect that flattered Backstage:

- **Backstage's lists were empty in rounds 1 and 2.** Its filter named the owner as
  `group:default/g3`; `spec.owner` holds `group:g3`, so every list returned no items, fast. Round 3
  filters on the stored value, and an equivalence gate now stops a run unless both products page out
  the same owner and component pairs. With real items, Backstage's 10k list fell from 1,279 to 474
  req/s: round 1's loss on lists stands at about 2.4× rather than 6×, and round 2's margin was
  understated. The 2.4× is a cross-round estimate; the adjudicator reran Backstage under round 1's
  conditions with the filter fixed and got 432 to 482 req/s (two runs, ADJUDICATION.md R3.6).
- **Loopback is judged on the image.** The image sets `PORT=8080`, so Toolbox listens on every
  interface, like Backstage. The rule is now *parity*, both failing, not *ahead*.
- **Scorecards count add-ons.** Backstage's Tech Insights community plugin counts, as the brief
  says, so the scorecards rule is *parity*, not *ahead*.
- **The client token is read-only.** Backstage's static token now carries `accessRestrictions` for
  catalog reads, so its client can no longer delete an entity: *parity*, not *ahead*. MCP gets a
  second, unrestricted token. Backstage delegates a token to the plugins MCP calls only when its
  restriction names no permissions, so a read-only token gets 403 on MCP. A token scoped to
  plugins works, but it can still delete an entity (204, ADJUDICATION.md R3.5): an agent token
  that can use MCP cannot be made read-only. The rule is scored on the read-only REST token, which
  favours Backstage; Toolbox uses one token for both, and it cannot delete. With a plugin-scoped
  token, Backstage's tool list is 14,744 bytes rather than 20,500; Toolbox is ahead either way.

Credential isolation was *ahead* in rounds 1 and 2 on evidence the adjudicator found too thin. Two
Toolbox changes made before the round 3 freeze close the two leaks it named (below), but the round 3
adjudication found a third: a same-user connector reads OpenFGA's memory. **The rule is a fail, so
security is 2 / 6 / 0, not the 3 / 5 / 0 this round's records first said.**

## Performance and memory

Medians of 5 runs per arm, arms alternating, oha 1.10.0 at 16 connections for 10 s per read, on
one 4-vCPU VM. Toolbox authorizes every read through OpenFGA, checking each decision once per
snapshot and then serving it from a cache; Backstage runs its template's `allow-all` policy. Round 3
spreads reads over 1,000 ids and all 50 owners; rounds 1 and 2 read one id and one owner each. So
round 3's Toolbox reads are mostly cache hits: the adjudicator's probe over all 10,000 ids gave
about 36,000 to 37,000 gets per second, against 56,500 here and Backstage's 4,150.

### Round 3: Toolbox at `581be50`

| Fact | Size | Toolbox min / median / max (n) | Backstage min / median / max (n) |
|---|---|---|---|
| `readyS` | 1000 | 0.9 / 0.92 / 0.93 (5) | 3.24 / 3.3 / 3.33 (5) |
| `catalogReadableS` | 1000 | 1.53 / 1.54 / 1.55 (5) | 6.27 / 6.43 / 6.48 (5) |
| `rssIdleMb` | 1000 | 89.4 / 89.8 / 90.7 (5) | 439.0 / 451.8 / 453.4 (5) |
| `rssLoadedMb` | 1000 | 118.9 / 125.2 / 128.2 (5) | 536.9 / 543.7 / 545.6 (5) |
| `recoveredS` | 1000 | 0.77 / 0.77 / 0.77 (5) | 2.23 / 2.25 / 2.28 (5) |
| `getP95Ms` | 1000 | 0.23 / 0.23 / 0.24 (5) | 5.81 / 5.9 / 6.32 (5) |
| `getRps` | 1000 | 145340.1 / 147346.9 / 147711.5 (5) | 3864.2 / 4088.7 / 4119.2 (5) |
| `listP95Ms` | 1000 | 1.04 / 1.05 / 1.06 (5) | 72.41 / 75.98 / 81.7 (5) |
| `listRps` | 1000 | 24014.4 / 24191.7 / 24394.0 (5) | 310.5 / 331.9 / 376.3 (5) |
| `readyS` | 10000 | 0.89 / 0.9 / 0.92 (5) | 3.21 / 3.33 / 3.39 (5) |
| `catalogReadableS` | 10000 | 2.13 / 2.15 / 2.17 (5) | 26.3 / 26.61 / 27.03 (5) |
| `rssIdleMb` | 10000 | 89.1 / 89.4 / 90.4 (5) | 438.5 / 451.5 / 453.6 (5) |
| `rssLoadedMb` | 10000 | 164.1 / 166.4 / 168.1 (5) | 803.8 / 805.7 / 849.9 (5) |
| `recoveredS` | 10000 | 1.13 / 1.13 / 1.13 (5) | 2.26 / 2.28 / 2.29 (5) |
| `getP95Ms` | 10000 | 0.45 / 0.49 / 0.53 (5) | 5.63 / 5.82 / 5.89 (5) |
| `getRps` | 10000 | 48759.5 / 56546.9 / 57953.7 (5) | 4033.2 / 4150.7 / 4245.8 (5) |
| `listP95Ms` | 10000 | 6.47 / 6.59 / 6.7 (5) | 58.29 / 59.06 / 305.81 (5) |
| `listRps` | 10000 | 3718.6 / 3789.8 / 3883.8 (5) | 116.9 / 473.6 / 482.4 (5) |

Every run of both arms passed the equivalence gate, with one digest per corpus size. One Backstage
run at 10k listed at 117 req/s with a 306 ms p95; the median is unaffected. Toolbox's medians moved
by no more than run-to-run spread from round 2, apart from the 10k get (58.6k to 56.5k req/s),
which now reads 1,000 ids instead of one.

**Rounds 1 and 2 below measured Backstage's lists on empty responses** (the filter defect above).
Their Backstage `listP95Ms` and `listRps` rows are not comparable with round 3. Round 1's list loss
stands, at a smaller margin: Toolbox's 198 req/s at 10k is behind the 474 req/s Backstage reaches
with real items, about 2.4× rather than 6×. That figure sets one round against another; the
adjudicator confirmed it by rerunning Backstage under round 1's conditions with the filter fixed
(432 to 482 req/s over two runs, R3.6). The fix in `916f596` took Toolbox's 10k list to 3,900 req/s.

### Round 1: Toolbox at `5ef6263` (Backstage lists empty)

| Fact | Size | Toolbox min / median / max (n) | Backstage min / median / max (n) |
|---|---|---|---|
| `readyS` | 1000 | 0.99 / 1.0 / 1.02 (5) | 3.51 / 3.61 / 3.95 (5) |
| `catalogReadableS` | 1000 | 1.61 / 1.63 / 1.65 (5) | 6.95 / 7.14 / 8.07 (5) |
| `rssIdleMb` | 1000 | 89.3 / 89.7 / 91.6 (5) | 440.6 / 452.1 / 455.6 (5) |
| `rssLoadedMb` | 1000 | 123.0 / 124.7 / 127.6 (5) | 543.7 / 546.9 / 562.2 (5) |
| `recoveredS` | 1000 | 0.78 / 0.78 / 0.78 (5) | 2.35 / 2.48 / 2.6 (5) |
| `getP95Ms` | 1000 | 0.23 / 0.23 / 0.23 (5) | 6.98 / 7.4 / 8.48 (5) |
| `getRps` | 1000 | 141951.7 / 147854.1 / 150371.1 (5) | 3004.4 / 3183.6 / 3404.5 (5) |
| `listP95Ms` | 1000 | 11.74 / 12.36 / 13.98 (5) | 52.93 / 54.89 / 62.64 (5) |
| `listRps` | 1000 | 1777.5 / 2003.4 / 2092.7 (5) | 391.6 / 444.3 / 527.1 (5) |
| `readyS` | 10000 | 0.89 / 0.96 / 1.01 (5) | 3.35 / 3.56 / 3.73 (5) |
| `catalogReadableS` | 10000 | 2.14 / 2.22 / 2.32 (5) | 26.77 / 28.29 / 29.21 (5) |
| `rssIdleMb` | 10000 | 88.2 / 89.6 / 89.9 (5) | 451.4 / 453.0 / 454.0 (5) |
| `rssLoadedMb` | 10000 | 166.7 / 167.6 / 173.7 (5) | 781.5 / 811.8 / 879.2 (5) |
| `recoveredS` | 10000 | 1.13 / 1.15 / 1.24 (5) | 2.29 / 2.39 / 2.5 (5) |
| `getP95Ms` | 10000 | 0.46 / 0.47 / 0.48 (5) | 6.63 / 7.08 / 7.44 (5) |
| `getRps` | 10000 | 56441.7 / 57228.5 / 58523.9 (5) | 3351.1 / 3434.5 / 3594.6 (5) |
| **`listP95Ms`** | 10000 | **113.81 / 118.05 / 126.76 (5)** | 18.21 / 21.09 / 22.14 (5) |
| **`listRps`** | 10000 | **189.2 / 197.6 / 204.9 (5)** | 1153.2 / 1207.2 / 1268.9 (5) |

**The loss, as recorded then.** A filtered list of one owner's components at 10k was 6× slower than
Backstage's, while every other measure was ahead by 3× or more. Backstage's side returned no items,
so the comparison was not like for like; Toolbox's own 118 ms p95 was still a defect. The cause: the filter `entity.owner == 'group:g3'`
was evaluated in CEL against every component, 10,000 evaluations per page, where Backstage's
catalog answers the same query from an indexed table.

**The fix** (`916f596`, in `crates/toolbox-expr` and `crates/toolbox-api`). A filter's top-level
`entity.<field> == '<string>'` conjuncts now pick candidates from an index of that declared field's
values. The index is built once per snapshot, the first time a list filters on that field. The
full filter still runs on every candidate. A redacted field is an object, never a string, so a
filter still cannot probe a hidden facet (`a_filter_cannot_probe_a_hidden_facet` passes
unchanged). The uses outside the benchmark are the MCP `query` tool, `toolbox query` and the UI
catalog filter, which filter on an owner or a team. It touches no harness file.

### Round 2: Toolbox at `916f596` (Backstage lists empty)

| Fact | Size | Toolbox min / median / max (n) | Backstage min / median / max (n) |
|---|---|---|---|
| `readyS` | 1000 | 0.81 / 0.97 / 1.0 (5) | 3.45 / 3.5 / 3.66 (5) |
| `catalogReadableS` | 1000 | 1.44 / 1.6 / 1.63 (5) | 6.71 / 7.04 / 7.17 (5) |
| `rssIdleMb` | 1000 | 89.2 / 89.4 / 90.1 (5) | 410.9 / 450.7 / 453.5 (5) |
| `rssLoadedMb` | 1000 | 118.2 / 125.7 / 127.1 (5) | 543.7 / 546.3 / 613.8 (5) |
| `recoveredS` | 1000 | 0.77 / 0.78 / 0.78 (5) | 2.33 / 2.39 / 2.51 (5) |
| `getP95Ms` | 1000 | 0.23 / 0.23 / 0.24 (5) | 6.6 / 6.95 / 7.36 (5) |
| `getRps` | 1000 | 144079.7 / 146179.6 / 152189.4 (5) | 3224.3 / 3493.8 / 3572.9 (5) |
| `listP95Ms` | 1000 | 0.99 / 1.03 / 1.1 (5) | 51.92 / 53.19 / 58.24 (5) |
| `listRps` | 1000 | 23028.8 / 24799.8 / 25395.5 (5) | 411.0 / 438.4 / 552.6 (5) |
| `readyS` | 10000 | 0.89 / 0.92 / 1.02 (5) | 3.32 / 3.45 / 3.49 (5) |
| `catalogReadableS` | 10000 | 2.13 / 2.18 / 2.43 (5) | 26.96 / 27.2 / 28.11 (5) |
| `rssIdleMb` | 10000 | 88.4 / 89.7 / 89.8 (5) | 450.2 / 452.9 / 453.4 (5) |
| `rssLoadedMb` | 10000 | 164.6 / 175.4 / 179.3 (5) | 803.9 / 829.2 / 896.3 (5) |
| `recoveredS` | 10000 | 1.13 / 1.13 / 1.13 (5) | 2.3 / 2.31 / 2.47 (5) |
| `getP95Ms` | 10000 | 0.45 / 0.45 / 0.53 (5) | 6.36 / 6.47 / 6.8 (5) |
| `getRps` | 10000 | 53246.2 / 58576.3 / 60014.1 (5) | 3534.0 / 3687.4 / 3793.1 (5) |
| `listP95Ms` | 10000 | 5.89 / 6.01 / 6.55 (5) | 17.27 / 17.68 / 18.94 (5) |
| `listRps` | 10000 | 3487.6 / 3899.5 / 3960.3 (5) | 1251.4 / 1279.0 / 1332.5 (5) |

At 10k, Toolbox's filtered lists went from 198 to 3,900 req/s, and p95 from 118 to 6.0 ms. The 1k list went from 2,003 to 24,800 req/s. No other median
moved by more than run-to-run spread.

## Static facts and single-run probes

These do not vary between runs, so they were run once per round. Round 3's values, with what
changed from rounds 1 and 2:

| Fact | Toolbox | Backstage | Source |
|---|---|---|---|
| Unauthenticated catalog read | 401 | 401 | surface.py |
| Forged identity header | 401 | 401 (guest sign-in 403) | surface.py |
| Unauthenticated MCP | 401 | 401 | surface.py |
| The read client deletes an entity | 405, entity still there | 403, entity still there (rounds 1 and 2, unrestricted token: 204, entity gone) | surface.py |
| MCP tools (read-only) | 14 (13) | 13 (9) | surface.py |
| `tools/list` bytes | 6,473 | 20,500 | surface.py |
| One entity over MCP, bytes | 253 | 580 | surface.py |
| Install on disk | 121.1 MB | 709.9 MB | footprint.py |
| Adopter files / lines for a first portal | 2 / 230 | 44 / 1,357 | footprint.py |
| Third-party packages | 605: 388 crates, 142 Go modules in OpenFGA, 75 npm packages in the UI (rounds 1 and 2: crates only) | 2,791 | footprint.py |
| Known vulnerabilities (high) | 5 (0) (rounds 1 and 2: 2 (0)) | 34 (16) | footprint.py, osv-scanner 2.2.4 |
| Release artifacts with provenance | 100% | 0% | footprint.py |
| Adopter lines changed per release | 0 (v0.1.0..HEAD) | 47.3 (v1.51.0..v1.57.1) | upgrade.py |
| Manual rollback steps per release | 0 (from v0.1.1) | 1.5 (9 Knex migrations over 6 releases) | upgrade.py |

Toolbox's five known vulnerabilities:

- RUSTSEC-2026-0543 and RUSTSEC-2026-0555: `chrono` and `chrono-tz` are unmaintained, through
  `cel`'s default `chrono` feature. Neither is a code vulnerability. Removing them drops CEL's
  `timestamp()` and `duration()`, which rules use (for example `duration('180d')` in
  [meta-catalog.md](../../docs/design/meta-catalog.md)), so they stay until `cel` moves off `chrono`.
- GO-2026-6859 (Go standard library 1.27.3), GO-2026-6860 and GO-2026-6866 (`golang.org/x/net`
  0.60.0), in the OpenFGA 1.22.1 binary the image carries; none is rated. OpenFGA 1.23.0
  (2026-11-20) is the upgrade path, not yet taken or checked.

The rule is still ahead on value (5 against 34) and fails its own `== 0` bar. The UI's npm packages
have none.

The facts in [facts.yaml](./facts.yaml) (loopback, owner-only fields, extension isolation, offline
validate, change preview, repo scope, federation, extension language and rebuild, the twelve
capability scenarios, contributors) each carry their evidence. Round 3 judges each product as it
ships to production: Toolbox's image, Backstage's create-app Dockerfile configuration. One fact is
scored in Backstage's favour: a permission rule counts as a capability although its policy is
TypeScript code, not configuration. Three were scored in Toolbox's favour in rounds 1 and 2
(loopback, credential isolation, scorecards); all three are now corrected, credential isolation
after the round 3 adjudication.

### Credential isolation, round 3

Two Toolbox changes made before the round 3 freeze, each for any deployment that runs connectors
beside the server (`71d8655`, `26b77f3`):

- The server makes itself non-dumpable at start, so a process of the same user can no longer read
  `/proc/<pid>/environ` or its memory (`crates/toolbox-cli/tests/serve.rs`; the previous binary
  leaked a probe variable to the same test).
- OpenFGA starts with a cleared environment plus an allowlist. Its datastore URI is no longer on its
  command line, which every local user can read; it is in a `0600` file in a `0700` directory,
  removed once OpenFGA is healthy. Every call needs a preshared key minted per start, so a connector
  that finds OpenFGA's port gets 401 (`openfga_gets_no_server_credentials`).

**What still leaks (ADJUDICATION.md R3.3).** An in-image connector runs as the same user as
OpenFGA, and Toolbox cannot make OpenFGA non-dumpable. The measurement VM has no Yama, so nothing
stops a same-user process from reading another's memory. The adjudicator read `/proc/<openfga>/mem`
and found the datastore password and the per-start key in OpenFGA's config text, and OpenFGA
accepted the key (200). Hosts with Yama default to `ptrace_scope=1`, which blocks this, but the image
cannot choose its host's kernel, and container hosts are unverified. So the fact is `false`, and the
rule is at parity, both failing.

Two more exposures:

- **`--database-url` on the command line.** `/proc/<pid>/cmdline` is readable by every local user,
  whatever the UID, and non-dumpable does not cover it. The harness passes the URL this way
  (`measure.py`), and `toolbox serve --database-url` is documented. The image passes it in the
  environment, so it is unaffected.
- **Files.** A connector can write any file the server's user can; whether `/portal` is writable in
  the image is unverified.

A remote connector holds only its own token. Backstage runs plugins in one process with one
environment and does not claim to isolate them (its threat model). To earn the pass, a connector
must lose access to its siblings' memory (its own UID, or a Landlock domain or PID namespace), and
the canary must be rerun on the image.

## Deviations from the pre-registration

- **Bars written after the smoke rounds.** The README gives no bar ("—") for adopter files and
  lines, upgrade lines and third-party packages. The scorecards set ≤ 10 files, ≤ 500 lines,
  ≤ 10 upgrade lines, ≤ 1,000 packages and `knownVulns == 0`, and a community bar of ≥ 10
  contributors. These bars decide pass or fail only. Both products have a value on each of those
  rules, so the values decide the standing, and no standing depends on them.
- **The scorecards, `snapshot.py` and `facts.yaml` were not frozen in rounds 1 and 2.** They were
  written during round 1. Round 3 froze all three (`e66526c`, `581be50`) before it ran. `facts.yaml`
  decides 24 of the 48 rules.
- **Extensibility is its own scorecard.** The README counts extension cost inside capabilities; the
  scorecards split it out. Capabilities is behind either way, and the verdict counts eight factors.
- **A rule falls back to outcomes when a value is missing**, as D01M857ZB468ZM2YS6GNSEG6YB9 says;
  the README said a missing value is unknown. No rule in this run lacks a value on either side.
- **Two harness YAML files were reformatted after rounds 1 and 2** (`yamark format`, D97). Both
  parse to the same data as at `dfeeea4`. After round 3, the same was done to the frozen standard
  and the two snapshots that `snapshot.py` wrote. All three parse to the same data as at `581be50`.
- **Round 2 follows a product change made after round 1's results**, and round 3 follows three
  more: the two credential isolation changes (`71d8655`, `26b77f3`), and `5968a55`, which narrows a
  filter by field index only when `kind` names exactly one kind and never on `id`. The protocol
  allows these when a change names a use outside the harness and touches no harness file (P5).
- **`facts.yaml` changed after round 3**, to score `extensionCredentialIsolated` as the round 3
  adjudication found it (`false`). The records were regenerated from the same raw results with
  `snapshot.py`; no other fact changed. The change goes against Toolbox.
- **Round 3 changed the harness after seeing results**: the equivalence gate, spread reads, the list
  filter, the production config, equal log levels, the read-only token, wider supply chain and
  upgrade window. Each answers a finding of the adjudication; each moves a measure toward Backstage
  or toward like-for-like.

## Not measured yet

- F2: wall time from a clean VM to a first portal, and bytes downloaded for Backstage (no clean
  image pinned).
- F5: commit-to-visible freshness through each product's pull mode. `catalogReadableS` covers only
  the first ingest.
- F11: the reference extension's lines of code.
- F13: calls and bytes to answer the 20 scripted questions.
- F12 federation: neither product federates; Toolbox's is designed, not built (D68).
- Multi-replica behaviour: one VM, one replica each.
