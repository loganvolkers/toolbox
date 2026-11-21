# Campaigns and tracks

*Status: decided 2026-11-21 (D01M7Y11G0YZT23W26Z9D3XW68Y). Companion to
[scorecards-for-scorecards.md](./scorecards-for-scorecards.md) (continuous standards) and
[report-ux.md](./report-ux.md) (how progress is shown).*

---

## 1. Conclusion

**Scorecards are the always-on bar. Campaigns are time-bound focus. Tracks group both into programs.**

A campaign does not invent new checks. It names a fleet goal over one or more existing scorecards
(`includes`), optional milestones, and dates. The pipeline scores scorecards first, then derives
campaign progress. Agents and humans read the same numbers from report, CLI, API, MCP and UI.

| Concept | Lifetime | What it holds | Progress |
|---|---|---|---|
| **Scorecard** | Continuous | Rules and levels for one kind | Per-entity level |
| **Track** | Long-running program | Title, owner, description; optional home for campaigns and scorecards | None (grouping) |
| **Campaign** | Start → end (or open-ended) | `includes` of scorecard goals, `targetPercent`, milestones | `%` of in-scope entities meeting every include |

## 2. Peer systems reviewed

Primary sources: Spotify Soundcheck / Backstage plugin docs and READMEs, Cortex Initiatives docs,
OpsLevel Campaigns docs, Port Initiatives docs (vendor marketing hosts are often blocked in cloud
sessions; GitHub and docs mirrors were preferred — see [research-log.md](../research-log.md)).

| System | Ongoing | Time-bound | Goal shape | Steal | Skip (v1) |
|---|---|---|---|---|---|
| **Soundcheck** | **Tracks** (check ladders) | **Campaigns** (checks + filters + dates + milestones + target %) | Pass all selected checks | Naming; milestones; target %; entity + per-check views | Historical Campaign Insights curves |
| **Cortex** | Scorecards | **Initiatives** on one published scorecard | Level **or** rule subset + deadline + owners | `atLeast` and/or `rules` on an include; owner + end date | Notifications, workflows |
| **OpsLevel** | Rubric (org) vs scorecards (team) | **Campaigns** with DRAFT→ENDED | Scoped checks; promote to rubric later | Lifecycle (`draft` / active / `ended`); keep campaigns off the continuous rollup | Reminders; promote-to-rubric automation |
| **Port** | Scorecards | **Initiatives** dashboards | Rule or whole-card progress | Separate dashboard mode in the hosted report | In-product widgets beyond the static report |

**Cross-cutting:** config-as-code (no admin UI for campaign definitions); reuse scorecard outcomes;
show fleet progress before drill-down.

## 3. Manifest shape

```yaml
tracks:
  - name: maintainer
    title: Maintainer quality
    owner: group:maintainers
    description: Long-term standards for building Toolbox itself.

scorecards:
  - name: maintainer-skill
    track: maintainer   # optional; groups the card under the track without a campaign
    # … levels, rules …

campaigns:
  - name: skill-silver
    title: Skills to silver
    track: maintainer
    owner: group:maintainers
    description: Every shipped skill reaches silver.
    start: "2026-11-01"
    end: "2026-12-31"
    targetPercent: 100
    draft: false
    # status: active   # optional override for snapshots / tests
    milestones:
      - name: half
        targetPercent: 50
    includes:
      - scorecard: maintainer-skill
        atLeast: silver
        # rules: [valid-frontmatter]   # optional; AND with atLeast when both set
```

### 3.1 Includes

Each include names one scorecard and at least one of `atLeast` or `rules` (non-empty).

| Field | Meaning |
|---|---|
| `scorecard` | Existing scorecard name |
| `atLeast` | Entity's level rank ≥ this level (scorecard levels order; `none` is below the first level) |
| `rules` | Named rules must have outcome `pass` |

An entity **meets** the campaign only if it satisfies **every** include (AND).

### 3.2 Scope and progress

1. **In scope:** entities that are **not** `skipped` on **every** scorecard named in `includes`
   (intersection). Per-scorecard `appliesTo` still decides skip via normal scoring.
2. **Meeting:** in-scope entities that satisfy every include.
3. **Percent:** `round(100 * meeting / inScope)` when `inScope > 0`, else `0` (to the nearest
   percent, for display).
4. **On target:** `100 * meeting >= targetPercent * inScope` (default `targetPercent` is `100`):
   exact counts, so 199 of 200 is shown as 100 but is not on a 100% target.
5. **Milestone met:** the same comparison with `milestone.targetPercent`.
6. **Measure:** when the included rules declare a `measure` (lines over a limit, matches of a
   pattern), its sum over in-scope entities. A campaign over counted standards pays this down.

Raw `inScope` and `meeting` counts are always emitted (deterministic for tests and ratchets).

### 3.4 Progress from a recorded start

A percentage of entities meeting the goal hides paydown inside entities that still fail: 120
violations in 40 files going to 114 in 39 files is 2.5% fewer files and 5.0% fewer violations.
So `toolbox report --baseline` records each campaign's starting point in the baseline file the
first time the campaign is active (`campaigns.<name>: {start, recorded, inScope, short,
measure?}`) and keeps it while the campaign's `start` is unchanged. The summary and `-o json`
then show `sinceStart`: entities short of the goal, and the measure, from the recorded start to
now, with the percentage of the way to zero (negative when it went backwards).

| To | Do |
|---|---|
| Restart a campaign (a new scope, a new standard) | Change its `start`, or run `--update --accept <scorecard>` for an intended change to a scorecard it includes, then commit the baseline |
| Keep its history across a change | Leave `start` alone and don't `--accept` its scorecards; the paydown then counts from the original start |

### 3.3 Status

Computed after scores (wall-clock `asOf` = UTC date `YYYY-MM-DD`):

| Status | When |
|---|---|
| `draft` | `draft: true` |
| `scheduled` | not draft, and `start` is after `asOf` |
| `ended` | not draft, and `end` is before `asOf` |
| `active` | otherwise |

A manifest `status` string overrides the computed lifecycle value (for fixtures and frozen reports).
Progress fields are independent of status: an `ended` campaign still reports its final percent.

## 4. Derived result

`Derived.campaigns`: name →

```json
{
  "status": "active",
  "track": "maintainer",
  "targetPercent": 100,
  "inScope": 32,
  "meeting": 28,
  "percent": 88,
  "onTarget": false,
  "measure": 120,
  "milestones": [{"name": "half", "targetPercent": 50, "met": true}],
  "entities": {
    "skill:plugins/…/scorecard": {"meeting": true},
    "skill:…": {"meeting": false, "gaps": ["maintainer-skill:atLeast silver (bronze)"]}
  }
}
```

`gaps` is present only when not meeting: one string per failed include.

## 5. Surfaces

| Surface | Behaviour |
|---|---|
| `toolbox validate` | Unique track/campaign names; track refs; scorecard/level/rule refs; dates `YYYY-MM-DD`; each include has `atLeast` and/or `rules` |
| `toolbox derive` / serve snapshot | Fills `Derived.campaigns` after scorecards |
| `toolbox report` JSON/HTML/web | `tracks`, `campaigns` (defs + progress, and `sinceStart` with `--baseline`, §3.4), without a server; Campaigns mode in `<toolbox-report>`; Overview lists active campaigns |
| `toolbox report --baseline` summary | One line per campaign with a recorded start: entities short and measure, from the start to now |
| `toolbox campaign [name]` | List or one campaign's progress and non-meeting entities (MCP `campaign_report`) |
| `GET /v1/campaigns` · `/v1/campaigns/{name}` | Same data, facet-visible entities only |
| `GET /v1/views/campaign/{name}` | Composite for the minimal UI |
| Minimal UI | `/cgn` list via home; `/cgn/{name}` detail |

## 6. What v1 defers

| Deferred | Why |
|---|---|
| Owner notifications / reminders | Lives under [notifications.md](./notifications.md) |
| Historical progress time series | Needs archived snapshots (Phase 2 facts/history) |
| Promote campaign checks into a "rubric" / org rollup | OpsLevel pattern; Toolbox keeps campaigns off `repoRollup` |
| Campaign-specific CEL `appliesTo` | Scope comes from included scorecards' `appliesTo` |
| Admin UI / DB-held campaigns | Charter: config as code (D2, D33) |

## 7. Sources

- Spotify Soundcheck / Backstage plugin documentation (tracks, campaigns, Campaign Insights)
- Cortex Initiatives documentation (scorecard initiatives, level vs rule goals)
- OpsLevel Campaigns and Maturity Report documentation (lifecycle, rubric vs campaign)
- Port Initiatives / scorecard dashboard documentation
- [report-ux.md](./report-ux.md), [plan.md](../plan.md) §4.3 initiative sketch (now campaigns)
