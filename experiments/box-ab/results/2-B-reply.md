# Phase 2, arm B: the agent's reply (DX feedback in §3)

1. The sarif kit had already started `semgrep-zero` in phase 1: start 209 (all suppressed; 104 + 105), short 63, recorded 2026-11-22 in .toolbox/results.json. New .toolbox/accept.py as `accept`: runs --update; if something regressed, reads --verdict and accepts only the scorecards that regressed; then restores each campaign's recorded start from the previous baseline, so accepted findings show as the campaign going backwards (tested: 209 → 210). TOOLS.md: progress names semgrep-zero and actionlint-zero with --skip so each runs only its tool; a Campaigns table; ruff needs no campaign. Workflow: header comment only. Gate exit 0; accept on the clean state leaves results.json byte-identical.
2. Couldn't: drop ruff-zero (the vendored kit always emits a campaign, no parameter); give the campaign a declared start (the kit can't); pin actions/report by SHA.
3. DX feedback:
   1. `--update --accept <scorecard>` restarts every campaign over that scorecard, with no way to keep history: accepting new findings and changing the standard look the same. Worked around by restoring campaign entries in results.json (accept.py). Wants --keep-campaigns, or a separate flag to restart.
   2. Kit campaigns can't be turned off or given a start: sarif and semgrep hard-code `campaigns: [<name>-zero]`; a tool at zero gets a pointless campaign; an include can't set start, end or draft.
   3. The campaign skill doesn't match the CLI: it says `toolbox campaign` prints "entities that still have gaps", but with --portal it prints only totals; the JSON shape isn't documented.
   4. `toolbox campaign --portal` reruns every connector (~40 s here because of semgrep); reading progress from the committed baseline would be instant. --skip helps, but you must know which connector feeds which campaign.
   5. Progress at zero reads oddly: "0 of 0 meet it (0%)" next to "100.0% of the way"; paydown only as a percentage, no absolute count.
   6. Empty heading for ruff: `## ruff  ()`.
   7. The actionlint campaign's scope moved silently: inScope 85 workflows at the baseline, 86 now, with no line saying so.
