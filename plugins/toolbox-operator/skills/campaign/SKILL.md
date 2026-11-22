---
name: campaign
description: >-
  Prints campaign progress over existing scorecards: how many entities meet the
  goal, and the paydown since each campaign's recorded start, in entities and in
  its measure. Use when checking a time-bound focus area, in a repo portal or
  against a server. With no name, it prints every campaign in one run.
---

# toolbox campaign

**In a repository (no server):** one run prints every campaign. Pass `--baseline` for the paydown
since each campaign's start, which `toolbox report --update` records:

```
toolbox campaign --portal .toolbox --baseline .toolbox/results.json
```

Each campaign prints `N of M meet it`, then `entities short` and, when its rules have a measure,
`measure`, each as `start → now since <date> (paid P, X% of the way)`. A negative paid count means
the campaign went backwards. This runs the portal's connectors like `toolbox report`, so `--skip`
the ones no campaign reads. Name one campaign to print only it. `-o json` gives the same as data.
In this mode it shows totals, not each entity: for the entities still short, run `toolbox report`
and read the campaign's scorecards.

**Against a server:** `toolbox campaign <name> -o json`; with no name it lists the campaigns.

1. Work the entities that are not meeting the goal; each gap names a scorecard include (`atLeast`
   or a rule).
2. Fix the underlying scorecard outcomes; do not edit a campaign's recorded start by hand.
3. A start moves only on purpose. `toolbox report --update --accept <scorecard>` restarts the
   campaigns over that scorecard, keeping the old start under `restartedFrom`. Add
   `--keep-campaigns` when the accepted findings are new to the campaign (a tool upgrade, a newly
   added tool) and its start should stay.

Campaigns are declared in the manifest (`tracks` / `campaigns`). They compose scorecards; they do
not define new checks. If it fails, the campaign name is wrong, or (with a server) the server is
down.
