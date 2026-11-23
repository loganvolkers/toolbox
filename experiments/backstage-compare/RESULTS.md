# Results: Toolbox against Backstage, beyond DX

*Run 2026-11-23 on the harness frozen at `dfeeea4`, by the README's pre-registration. Round 1
measured Toolbox at `5ef6263` and lost on filtered lists at 10k components. Round 2 measured
`916f596`, which adds one product fix. Raw results for both rounds are in [results/](./results);
the product records the scorecards read are [portal/comparisons/](../../portal/comparisons).*

## Verdict

**After round 2, Toolbox is ahead of Backstage on eight of the nine factors and behind on one,
capabilities.** In round 1 it was behind on performance as well, for one reason: filtered lists
at 10k components. This is the Compare view of the Toolbox for Toolbox report, with values from
round 2:

| Factor (scorecard) | Standing | Rules ahead / parity / behind |
|---|---|---|
| Performance (`vs-backstage-performance`) | **ahead** (round 1: **behind**) | 7 / 0 / 0 (round 1: 6 / 0 / 1) |
| Footprint (`vs-backstage-footprint`) | **ahead** | 5 / 0 / 0 |
| Security (`vs-backstage-security`) | **ahead** | 5 / 3 / 0 |
| Supply chain (`vs-backstage-supply-chain`) | **ahead** | 4 / 0 / 0 |
| Change safety (`vs-backstage-change-safety`) | **ahead** | 4 / 0 / 0 |
| Org fit (`vs-backstage-org-fit`) | **ahead** | 1 / 1 / 0 (federation: neither) |
| Agent surface (`vs-backstage-agent-surface`) | **ahead** | 2 / 1 / 0 |
| Extensibility (`vs-backstage-extensibility`) | **ahead** | 2 / 0 / 0 |
| Capabilities (`vs-backstage-capabilities`) | **behind** | 1 / 5 / 7 |

The capabilities Toolbox lacks are docs search, docs, templates, API docs, Kubernetes,
notifications and community (4 contributors against 2,826). Templates are self-service actions,
which are paused (D64). Toolbox's own bars fail in three places: two known vulnerabilities, no
federation, and the capabilities above. Each is published as a loss, not waived.

## Performance and memory

Medians of 5 runs per arm, arms alternating, oha 1.10.0 at 16 connections for 10 s per read, on
one 4-vCPU VM. Toolbox checks every read in OpenFGA; Backstage runs its template's `allow-all`
policy.

### Round 1: Toolbox at `5ef6263`

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

**The loss.** A filtered list of one owner's components at 10k was 6× slower than Backstage, while
every other measure was ahead by 3× or more. The cause: the filter `entity.owner == 'group:g3'`
was evaluated in CEL against every component, 10,000 evaluations per page, where Backstage's
catalog answers the same query from an indexed table.

**The fix** (`916f596`, in `crates/toolbox-expr` and `crates/toolbox-api`). A filter's top-level
`entity.<field> == '<string>'` conjuncts now pick candidates from an index of that declared field's
values. The index is built once per snapshot, the first time a list filters on that field. The
full filter still runs on every candidate. A redacted field is an object, never a string, so a
filter still cannot probe a hidden facet (`a_filter_cannot_probe_a_hidden_facet` passes
unchanged). The use outside the benchmark is owner and team pages, the MCP `query` tool and
`toolbox query`, which all filter on an owner or a team. It touches no harness file.

### Round 2: Toolbox at `916f596`

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

At 10k, filtered lists went from 198 to 3,900 req/s, and p95 from 118 to 6.0 ms: 3× Backstage's
throughput at a third of its latency. The 1k list went from 2,003 to 24,800 req/s. No other median
moved by more than run-to-run spread.

## Static facts and single-run probes

These do not vary between runs, so they were run once per round. Both rounds gave the same values.

| Fact | Toolbox | Backstage | Source |
|---|---|---|---|
| Unauthenticated catalog read | 401 | 401 | surface.py |
| Forged identity header | 401 | 401 | surface.py |
| Unauthenticated MCP | 401 | 401 | surface.py |
| A client deletes an entity | 405, entity still there | 204, entity gone | surface.py |
| MCP tools (read-only) | 14 (13) | 13 (9) | surface.py |
| `tools/list` bytes | 6,473 | 20,500 | surface.py |
| One entity over MCP, bytes | 253 | 580 | surface.py |
| Install on disk | 121.1 MB | 709.9 MB | footprint.py |
| Adopter files / lines for a first portal | 2 / 230 | 44 / 1,357 | footprint.py |
| Third-party packages | 388 | 2,791 | footprint.py (Cargo.lock; yarn.lock) |
| Known vulnerabilities (high) | 2 (0) | 34 (16) | footprint.py, osv-scanner 2.2.4 |
| Release artifacts with provenance | 100% | 0% | footprint.py |
| Adopter lines changed per release | 0 (v0.1.1..HEAD) | 47.3 (v1.51.0..v1.57.1) | upgrade.py |
| Manual rollback steps per release | 0 | 1.5 (9 Knex migrations over 6 releases) | upgrade.py |

Toolbox's two known vulnerabilities are RUSTSEC-2026-0543 and RUSTSEC-2026-0555: `chrono` and
`chrono-tz` are unmaintained, through `cel`'s default `chrono` feature. Neither is a code
vulnerability. Removing them drops CEL's `timestamp()` and `duration()`, which rules use (for
example `duration('180d')` in [meta-catalog.md](../../docs/design/meta-catalog.md)), so they stay
until `cel` moves off `chrono`. The rule is still ahead on value (2 against 34), and fails its own
`== 0` bar.

The facts in [facts.yaml](./facts.yaml) (loopback, owner-only fields, extension isolation, offline
validate, change preview, repo scope, federation, extension language and rebuild, the twelve
capability scenarios, contributors) each carry their evidence. Where the definition was
ambiguous, they are scored in Backstage's favour. For example, a permission rule counts as a
capability for Backstage although its policy is TypeScript code, not configuration.

## Deviations from the pre-registration

- **Bars written after the smoke rounds.** The README gives no bar ("—") for adopter files and
  lines, upgrade lines and third-party packages. The scorecards set ≤ 10 files, ≤ 500 lines,
  ≤ 10 upgrade lines, ≤ 1,000 packages and `knownVulns == 0`, and a community bar of ≥ 10
  contributors. These bars decide pass or fail only. Both products have a value on each of those
  rules, so the values decide the standing, and no standing depends on them.
- **`snapshot.py` and `facts.yaml` were written during round 1**, not frozen with the harness.
  Neither measures anything: one takes medians and merges, and the other records evidence.
- **Two harness YAML files were reformatted after the runs** (`yamark format`, D97). Both parse to
  the same data as at `dfeeea4`.
- **Round 2 follows a product change made after round 1's results**, as the protocol allows when the
  change names a use outside the harness and touches no harness file (P5).

## Not measured yet

- F2: wall time from a clean VM to a first portal, and bytes downloaded for Backstage (no clean
  image pinned).
- F5: commit-to-visible freshness through each product's pull mode. `catalogReadableS` covers only
  the first ingest.
- F11: the reference extension's lines of code.
- F13: calls and bytes to answer the 20 scripted questions.
- F12 federation: neither product federates; Toolbox's is designed, not built (D68).
- Multi-replica behaviour: one VM, one replica each.
