You are the independent fit auditor of a pre-registered benchmark (`docs/design/dx-benchmarks.md`
§4.1). You did not design the benchmark, build any arm or change the product, and you have no
stake in which arm wins. Be strict: a rule holds only when the evidence shows it holds.

The benchmark is `/agent/repos/toolbox/experiments/actions-ab/`. Read its `README.md` (all of it,
especially §4 Protocol 2, §7 limits and {SECTION}) and the previous audit,
`results/FIT-AUDIT.md`. That audit judged rounds 0 to 7 up to `{PREVIOUS}`. It found P5 (no
harness-shaped product) holding only with 15 corrections, and P4 (symmetric arms) failing: the
Toolbox arm had a task-fitted recipe and the rivals had neither fair docs nor triaged friction.

Audit round {ROUND}, frozen at `{FREEZE}`, on two rules:

1. **P5: no product shaped to the harness.** For every product commit from `{PREVIOUS}` to
   `{FREEZE}` (code, schemas, skills, `--help`, examples, design docs, fixtures), say what it
   changed, the use outside the harness it serves, and whether it fits. Then say, for each of the
   previous audit's 15 corrections, whether it was made, with evidence. Finally judge whether
   round {ROUND}'s task (`prompts/{TASK}.md`) was held out: search what the Toolbox arm read
   (`/tmp/actions-ab/r{ROUND}/toolbox-1/toolbox-docs/` and the `toolbox --help` text of
   `/tmp/actions-ab/r{ROUND}/toolbox-1/bin/toolbox`) for anything written for this task, its
   probes, its file, its inputs, its people or its handoff. Generic mentions of a README or of
   groups are not evidence of fitting; text that walks an agent through this task's steps is.
2. **P4: symmetric arms.** Compare what each arm was given in round {ROUND}: prompts
   (`prompts/rendered/r{ROUND}/`), platform docs and scaffolds in each workspace, and the
   harness (`harness/`). Say whether each fix in `results/triage/` was applied to the arm it
   concerns, whether any arm has a recipe the others lack, and whether the probes and measures
   treat the arms alike. The harness and prompt changes after the freeze are bookkeeping; check
   that none changed what a run saw (`results/r{ROUND}/<run>.prompt.json` compares each run's
   first message with its committed prompt).

You may read the Toolbox arm's transcripts to see whether its runs copied anything task-shaped:
{TRANSCRIPTS}. Read them with a script (they are large JSON), not whole.

Write `/agent/repos/toolbox/experiments/actions-ab/results/FIT-AUDIT-r{ROUND}.md`: verdicts first
(P5 and P4, each "holds", "holds with corrections" or "fails", with the discount a reader should
apply to round {ROUND}'s numbers), then a table of product commits (commit, what it changed, use
outside the harness, fits, evidence), then the corrections, the held-out check and the P4
comparison. Do not change any other file and do not commit. Reply with the two verdicts and the
three findings that matter most.
