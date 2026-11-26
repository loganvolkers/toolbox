# Round 7: the tally meets the bar, but it is not a pass

Round 7 ran each arm three times against Seikika at `554e83f`. Every run delivered: C1 and P1–P10
held in all twelve, as four independent adjudicators observed them
([ADJUDICATION.md](ADJUDICATION.md), records in [adjudication/](adjudication/)). Seikika beats
every rival on lines. It beats GitHub Actions and Backstage on calls, and against vanilla calls
are inconclusive, not lost. So the tally meets the bar for the first time.

**It is not a pass.** The independent [fit audit](../FIT-AUDIT.md) found two faults:

- **P5:** the Seikika `add-action` skill and `--help` had been fitted to this task;
- **P4:** the rival arms lacked help that Seikika had.

So this round's Seikika results stand only with that label: *with a Seikika-only task-fitted
recipe*. The corrections are in the product (`7193247`, `fb5dc3f`) and the harness (the
[triage](../triage/)). Round 8 runs them on a held-out task.

| Arm | Lines to maintain (median, range) | Tool calls (median, range) | Wall time (s, median) |
|---|---|---|---|
| Seikika | 111 (83–124) | 24 (23–26) | 240 |
| GitHub Actions | 986 (958–1210) | 37 (30–42) | 656 |
| Backstage | 1714 (1583–1780) | 54 (51–77) | 823 |
| Vanilla | 617 (524–807) | 20 (15–20) | 257 |

| Rival | H2 lines | H3 tool calls | Bar |
|---|---|---|---|
| GitHub Actions | holds: 111 vs 986, W 252 | holds: 24 vs 37, W 12 | met |
| Backstage | holds: 111 vs 1714, W 197 | holds: 24 vs 54, W 26 | met |
| Vanilla | holds: 111 vs 617, W 283 | inconclusive: 24 vs 20, W 5 | met |

## What the triage found in the rival arms

Three triagers, who built no arm, walked each rival's six transcripts (rounds 6 and 7) call by
call ([gha.md](../triage/gha.md), [backstage.md](../triage/backstage.md),
[vanilla.md](../triage/vanilla.md)).

- **GitHub Actions:** most of this arm's cost came from a prompt contradiction, not from the
  platform. The GitHub Actions prompt credits GitHub-only features configured as code, but the
  shared task says "your system must enforce" the approval rules. All six runs rebuilt GitHub's
  approval gate and run history in a local CLI, which is 466–789 kept lines and 5–18 calls per
  run. With that removed, round 7's H3 against GitHub Actions would be inconclusive (25–30 calls
  against 24, W 5); H2 would still hold. The arm also had no GitHub or act docs in its workspace.
- **Backstage:** the arm had no Backstage docs, while Seikika had its own. With the docs
  provided, H3 against Backstage would probably be inconclusive; H2 is safe. Of the kept lines,
  the triager counted 258 and 298 in backstage-1 and backstage-2 as code-host glue: a catalog
  provider and file reads for the sandbox's bare local origins, which Backstage's GitHub
  integration would replace (README §7, limit 7).
- **Vanilla:** every fix it found lowers vanilla's numbers: host git signing and fsmonitor in the
  checkouts, plus a clearer rejected-push sentence in the task.

So the calls wins over GitHub Actions and Backstage were also obtained against arms that lacked
fair help. Round 8 applies every accepted fix to every arm before its freeze.
