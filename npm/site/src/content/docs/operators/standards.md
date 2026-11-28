---
title: Track progress and change a standard
description: Use campaigns to track a scorecard goal and record reviewed changes to standards.
sidebar:
  order: 11
---

Keep a **scorecard** for an ongoing standard. Use a **campaign** to track a specific goal over the scorecard's results, such as moving every repository to silver. Use a **track** if you want to group related scorecards and campaigns in the report.

## Track a goal with a campaign

Add this example to the manifest from the [repo portal quickstart](/start/quickstart/). It uses the starter's existing `repo-hygiene` scorecard. Add a goal under `includes` with a level in `atLeast`, named `rules`, or both. When you add several includes, each entity must meet all of them for the campaign to count it as meeting the goal.

```yaml
campaigns:
  - name: repository-basics
    title: Repository basics to silver
    owner: group:maintainers
    targetPercent: 100
    includes:
      - scorecard: repo-hygiene
        atLeast: silver
```

From the checkout root, use the [0.2 source CLI and workspace connector](/start/install/#build-from-source). Keep your report connector settings consistent with CI.

```bash
seikika validate .seikika
seikika report .seikika --baseline .seikika/results.json --update
seikika campaign repository-basics --portal .seikika --baseline .seikika/results.json
```

Look for the number of repositories that meet the goal and how many were short at the recorded start. Commit the manifest and the reviewed baseline. The local command shows totals. In a running portal, use `seikika campaign NAME --url PORTAL_URL` with your credentials to see gaps for the entities you can view. You can also browse the **Campaigns** tab in a [static report](/operators/publish/).

## Review an intentional change

If a change makes a rule stricter, compare the old and new manifests with [Change a portal](/operators/portal/). Review the new findings with the scorecard owner. To accept expected regressions for one scorecard into the baseline, use the following from the checkout root. Replace `repo-hygiene` with the scorecard you changed.

```bash
seikika report .seikika --baseline .seikika/results.json --update --accept repo-hygiene
```

This accepts regressions for that scorecard. Regressions for other scorecards still fail the command. By default, campaigns that include the accepted scorecard record a new start. If a tool upgrade adds findings and you want those findings to count against the existing campaign start, add `--keep-campaigns`. Review and commit the baseline change, then rerun with `--ci`.

## Record an exception for one finding

Use a **waiver** after your team accepts a specific rule failure on one entity. Add an entry under `waivers` with the `entity` ID, `scorecard`, `rule`, `decision`, and `reason`. Use the decision ID as `decision`. Seikika checks for `../docs/decisions/DECISION_ID.md` relative to the manifest directory. For an example, see [Seikika's waivers](https://github.com/seikika/seikika/blob/main/.seikika/waivers.yaml).

Run `seikika validate` and review the report. The rule counts as waived when it fails, and its evidence stays visible. Once the underlying finding is fixed, remove the waiver and check the result with the CI gate.

See [scorecard outcomes](/concepts/scorecards/) for how failures, unknown results, and waivers affect the report.
