You are the independent fit auditor of a pre-registered benchmark (`docs/design/dx-benchmarks.md`
§4.1). You did not design the benchmark, build any arm or change the product, and you have no
stake in which arm wins. Be strict: a rule holds only when the evidence shows it holds.

The benchmark is `/agent/repos/toolbox/experiments/actions-ab/`. Read its `README.md` (all of it,
especially §4 Protocol 2, §7 limits and §§8) and the previous audit,
`results/FIT-AUDIT.md`. That audit judged rounds 0 to 7 up to `ce8210f`. It found P5 (no
harness-shaped product) holding only with 15 corrections, and P4 (symmetric arms) failing: the
Toolbox arm had a task-fitted recipe and the rivals had neither fair docs nor triaged friction.

Audit round 8, frozen at `8519cd5`, on two rules:

1. **P5: no product shaped to the harness.** For every product commit from `ce8210f` to
   `8519cd5` (code, schemas, skills, `--help`, examples, design docs, fixtures), say what it
   changed, the use outside the harness it serves, and whether it fits. Then say, for each of the
   previous audit's 15 corrections, whether it was made, with evidence. Finally judge whether
   round 8's task (`prompts/task-notice.md`) was held out: search what the Toolbox arm read
   (`/tmp/actions-ab/r8/toolbox-1/toolbox-docs/` and the `toolbox --help` text of
   `/tmp/actions-ab/r8/toolbox-1/bin/toolbox`) for anything written for this task, its
   probes, its file, its inputs, its people or its handoff. Generic mentions of a README or of
   groups are not evidence of fitting; text that walks an agent through this task's steps is.
2. **P4: symmetric arms.** Compare what each arm was given in round 8: prompts
   (`prompts/rendered/r8/`), platform docs and scaffolds in each workspace, and the
   harness (`harness/`). Say whether each fix in `results/triage/` was applied to the arm it
   concerns, whether any arm has a recipe the others lack, and whether the probes and measures
   treat the arms alike. The harness and prompt changes after the freeze are bookkeeping; check
   that none changed what a run saw (`results/r8/<run>.prompt.json` compares each run's
   first message with its committed prompt).

You may read the Toolbox arm's transcripts to see whether its runs copied anything task-shaped:
`/tmp/cursor/cloud-agent-transcripts/2026-11-24T15-41-21Z-ce71/bc-e3c36ef3-2252-54a5-a219-28924a2ed903/transcript.json` (toolbox-1), `/tmp/cursor/cloud-agent-transcripts/2026-11-24T15-46-54Z-9617/bc-5af509bf-888c-5a8b-a4e0-93dbb0d24c16/transcript.json` (toolbox-2), `/tmp/cursor/cloud-agent-transcripts/2026-11-24T15-53-56Z-95e9/bc-b1ec99a3-c721-5cae-850f-879385bf99e6/transcript.json` (toolbox-3). Read them with a script (they are large JSON), not whole.

Write `/agent/repos/toolbox/experiments/actions-ab/results/FIT-AUDIT-r8.md`: verdicts first
(P5 and P4, each "holds", "holds with corrections" or "fails", with the discount a reader should
apply to round 8's numbers), then a table of product commits (commit, what it changed, use
outside the harness, fits, evidence), then the corrections, the held-out check and the P4
comparison. Do not change any other file and do not commit. Reply with the two verdicts and the
three findings that matter most.
