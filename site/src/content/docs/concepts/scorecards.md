---
title: Scorecards
description: A level is the highest bar whose rules all pass, with evidence and a fix on each rule.
---

A scorecard is a ladder of rules on one kind of entity. The levels in this repository are `none`, `bronze`, `silver` and `gold`. An entity's level is the highest one where every rule at that level and below passes.

Each rule carries:

- a **check**, a short expression over the entity's facts
- **evidence**, the items that failed
- a **fix**, what to change

An agent reads the evidence and the fix, edits the file, and refreshes that entity. The score is computed again from the facts. A missing tool or a skipped job is `unknown`. Unknown is not a pass, and the ratchet does not treat it as a regression.

## The ratchet

CI compares the current report with a committed baseline (for example `.toolbox/results.json` or this repository’s `portal/results.json`). The first run with `--update` records today’s failures. Later runs fail when a rule newly fails or a failing rule’s evidence grows; improvements update the file so the bar only moves one way. A new entity must reach the floor declared for its scorecard (bronze for the standards in this repository). An existing entity may not drop a level. Missing tools yield `unknown`, which is neither a pass nor a regression.

That committed-results shape is what we learned from [Betterer](/toolbox/compare/betterer/). Walk through recording a baseline in [Quickstart](/toolbox/start/quickstart/).

## This site

`maintainer-site` scores every page:

| Level | What must be true |
|---|---|
| Bronze | The page has a title and a description, and its links resolve |
| Silver | It is not a draft, and the sidebar includes it |
| Gold | It contains no secret-shaped string |

A page that is not linked from the sidebar stops at bronze. That is the catalog saying the page exists and is not yet part of the site a reader can find.

The repository's own level is the lowest of its counted scorecards, not an average.
