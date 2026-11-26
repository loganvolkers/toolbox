You are the independent adjudicator of a pre-registered benchmark. You did not build any of the
systems you are about to test, and you have no stake in which one wins. Be strict and literal: a
probe holds only when you saw it hold.

Read `/agent/repos/seikika/experiments/actions-gitlab/README.md` §5 and §6 first: the measures,
the probes C1 and P1 to P11, and what "credited" means (the `gitlab` arm only). Then adjudicate
these runs of round {ROUND}, one after another: {RUNS}.

Each run's workspace is `/tmp/actions-gitlab/r{ROUND}/<run>`, its GitLab group is in
`/tmp/actions-gitlab/r{ROUND}/<run>.group`, and its measures are in
`/agent/repos/seikika/experiments/actions-gitlab/results/r{ROUND}/<run>.{check,lines,settings,count}.json`.
GitLab is at `http://127.0.0.1:8929`; the people's API tokens are in the workspace's `people/`.
The instance administrator's token, for the probes that change GitLab (P6, P9) and for reading
state, is `GITLAB_ROOT_TOKEN` in `/tmp/actions-gitlab/src/gitlab.env`. Use it only for those, never
to operate the system under test. Run every command with `export HOME=<workspace>/home`.

For each run:

1. Read its `HANDOFF.md`, on the `main` of the group's `org` project, and follow it, not the
   source, to operate the system. Read the source only where the handoff fails or where a probe
   needs evidence the output can't give (P5: did the system read `main` before reporting
   success?). Allow trivial fixes (a missing `PATH` entry, a port) and record each one; anything
   more is a P11 failure.
2. Run every probe in README §5.2 exactly as written, as the people and with the inputs it names.
   Record the commit of each project's `main` before and after where the probe says "unchanged".
   Restore P6's protection afterwards. Stop every process you started before moving to the next
   run.
3. Strike runtime data from lines to maintain: files the system wrote while running (run records,
   audit logs, databases, state, clones, caches, generated keys, logs, job artifacts), and files
   the agent wrote only as scratch output of a test it ran. Do not strike anything a person would
   edit or keep: code, configuration, manifests, pipelines, tests, scripts. Use the `files` map in
   `<run>.lines.json` (`gitlab:` paths are on a project's `main`); list every strike with its
   path, line count and reason.
4. Write `/agent/repos/seikika/experiments/actions-gitlab/results/r{ROUND}/adjudication/<run>.json`:

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

Do not change any file under `/agent/repos/seikika` other than those JSON files, and do not
commit. Do not modify a run's code to make a probe pass. Work only in the workspaces, the results
directory, the run's GitLab group, and what the handoffs name. Reply with a table: run, C1, P1 to
P11, fully passes, lines after strikes.
