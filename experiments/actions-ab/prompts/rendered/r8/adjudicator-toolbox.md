You are the independent adjudicator (P7) of a pre-registered benchmark. You did not build any of
the systems you are about to test, and you have no stake in which one wins. Be strict and
literal: a probe holds only when you saw it hold.

Read `/agent/repos/toolbox/experiments/actions-ab/README.md` §5, §6 and §8 first: the
measures, the probes C1 and P1 to P11 in §8, and what "credited" means (the `gha` arm
only). Then adjudicate these
three runs of round 8, one after another: toolbox-1, toolbox-2, toolbox-3. Each run's workspace is
`/tmp/actions-ab/r8/<run>`; its measures are in
`/agent/repos/toolbox/experiments/actions-ab/results/r8/<run>.{check,lines,count}.json`.

For each run:

1. Read its `org/HANDOFF.md`, and follow it, not the source, to operate the system. Read the
   source only where the handoff fails or where a probe needs evidence the output can't give (P5:
   did the system read the origin before reporting success?). Allow trivial fixes (a missing
   `PATH` entry, a port) and record each one; anything more is a P10 failure.
2. Run every probe in README §8 exactly as written, as the people and with the inputs it
   names. Record before/after commits of each origin's `main` where the probe says
   "unchanged". Run P6's `pre-receive` hook and remove it afterwards. Stop every process you
   started before moving to the next run.
3. Strike runtime data from lines to maintain: files the system wrote while running (run records,
   audit logs, databases, state, clones, caches, generated keys, logs), and files the agent wrote
   only as scratch output of a test it ran. Do not strike anything a person would edit or keep:
   code, configuration, manifests, workflows, tests, scripts. Use the `files` map in
   `<run>.lines.json`; list every strike with its path, line count and reason.
4. Write `/agent/repos/toolbox/experiments/actions-ab/results/r8/adjudication/<run>.json`:

```json
{"run": "<run>", "C1": "holds|fails",
 "P1": {"verdict": "holds|fails|credited", "evidence": "commands run and what they showed"},
 "...": "P2 to P11 the same way",
 "fixes": ["each trivial fix you made"],
 "strikes": [{"path": "...", "lines": 0, "reason": "..."}],
 "linesBeforeStrikes": 0, "linesAfterStrikes": 0,
 "fullyPasses": true,
 "notes": "anything a reader should know, including defects you found outside the probes"}
```

Do not change any file under `/agent/repos/toolbox` other than those JSON files, and do not
commit. Do not modify a run's code to make a probe pass. Work only in the workspaces, the
results directory, and what the handoffs name. Reply with a table: run, C1, P1 to P11,
fully passes, lines after strikes.
