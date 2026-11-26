# Round 1: the bar fails against vanilla

Round 1 ran each arm three times against Seikika at `0ada74c`. Every run in every arm delivered:
C1 and P1–P10 held in all twelve, as four independent adjudicators observed them
([ADJUDICATION.md](ADJUDICATION.md), records in [adjudication/](adjudication/)). Seikika beat GitHub
Actions and Backstage on lines to maintain. Against a vanilla app it won neither measure by more
than W, so the bar, which needs a win against every rival, fails. Seikika lost no measure.

| Arm | Lines to maintain (median, range) | Tool calls (median, range) | Wall time (s, median) |
|---|---|---|---|
| Seikika | 350 (304–745) | 53 (51–91) | 425 |
| GitHub Actions | 855 (855–909) | 34 (28–40) | 566 |
| Backstage | 1749 (1597–1804) | 56 (53–67) | 849 |
| Vanilla | 603 (506–675) | 16 (11–21) | 244 |

| Rival | H2 lines | H3 tool calls | Bar |
|---|---|---|---|
| GitHub Actions | holds: 350 vs 855, W 441 | inconclusive: 53 vs 34, W 40 | met |
| Backstage | holds: 350 vs 1749, W 441 | inconclusive: 53 vs 56, W 40 | met |
| Vanilla | inconclusive: 350 vs 603, W 441 | inconclusive: 53 vs 16, W 40 | not met |

W is the wider of the two arms' ranges, and Seikika's is set by seikika-3: 745 lines and 91 calls,
against 304–350 and 51–53 for the other two.

## Why, and what changed

Seikika's runs spent their calls learning the box, then distrusting it. The friction came from
the transcripts and the adjudicators' notes; each fix is general, not shaped to this task (P5).

| Runs | Friction | Fix |
|---|---|---|
| all three | Each agent read the schema and skills to find how a repo declares an action and an org portal lists its children, and seikika-1 and seikika-2 hand-made an Ed25519 key | Fixed before the dry run: the serving portal keeps its own signing key and prints it; the `actions` and `run` skills give the whole loop ([r2dry](../r2dry/README.md)) |
| seikika-3 (calls 24–45) | It built a connector to re-read `DEPRECATED.md` from the origin, believing `succeeded` did not | Fixed after the dry run: `verification.files` records each file's blob and whether the base has it, and a change that drifted from its preview fails |
| seikika-3 (about 15 calls) | It wrote `org/bin/actions`, a shell wrapper repeating `--url` and `--as` on every `seikika run` | `seikika run` reads `SEIKIKA_URL` and `SEIKIKA_AS`, like the read commands (`a56cfd2`, after round 2's freeze) |
| seikika adjudicator | The fetch that checks a landed push ignored its own error, so a failed fetch read back the run's own push | A base that cannot be fetched fails the run (`83081db`, after round 2's freeze) |
| seikika-2 | Its handoff said the requester authors the commit, but every checkout's git user was ann's | Fixed in round 2's freeze: the org portal's clones commit as Seikika, with `Requested-By` and `Approved-By` trailers |

Not fixed, and why:

- **No OpenFGA in dev mode.** All three Seikika runs served without OpenFGA, so
  `can_run_actions` was not enforced; approval rules were. The rival arms enforce what they wrote
  themselves. Dev sign-in on a loopback address is a deliberate default (v1-spec §10.2); a portal
  that serves others runs OpenFGA.
- **Definitions read from checkouts.** seikika-2 and seikika-3 listed working checkouts as their
  org portal's children, not git URLs, so an uncommitted edit could be offered. Git children are
  synced clones; round 1's skills did not mention them, and since the dry run's fixes the `run`
  and `serve` skills give `federation.children: [{git: <url>}]` as the way to list them.
- **seikika-1's startup race.** Its first listing, right after its start script, was empty; its
  script did not wait for the server to answer.

Round 2 is frozen at `ad36c06`. It has the fixes for the key, the skills, verification and commit
identity, but not the wrapper or fetch fixes.

## Measures

Raw measures per run: `<run>.check.json`, `<run>.lines.json` (before strikes), `<run>.count.json`,
`<run>.HANDOFF.md`. Lines after strikes are in each adjudication record.
