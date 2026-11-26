# Round 4: the bar fails against vanilla, with both measures inconclusive

Round 4 ran each arm three times against Seikika at `4825782`. Every run delivered: C1 and P1–P10
held in all twelve, as four independent adjudicators observed them
([ADJUDICATION.md](ADJUDICATION.md), records in [adjudication/](adjudication/)). Seikika beats
Backstage on both measures and GitHub Actions on lines. Against a vanilla app neither measure is a
win, so the bar fails.

| Arm | Lines to maintain (median, range) | Tool calls (median, range) | Wall time (s, median) |
|---|---|---|---|
| Seikika | 368 (322–368) | 26 (21–34) | 210 |
| GitHub Actions | 967 (895–1014) | 33 (33–39) | 537 |
| Backstage | 1591 (1581–2067) | 71 (53–75) | 889 |
| Vanilla | 694 (474–803) | 19 (12–23) | 300 |

| Rival | H2 lines | H3 tool calls | Bar |
|---|---|---|---|
| GitHub Actions | holds: 368 vs 967, W 119 | inconclusive: 26 vs 33, W 13 | met |
| Backstage | holds: 368 vs 1591, W 486 | holds: 26 vs 71, W 22 | met |
| Vanilla | inconclusive: 368 vs 694, W 329 | inconclusive: 26 vs 19, W 13 | not met |

Round 3's fixes moved tool calls the right way against vanilla: the loss of 27 vs 13 became an
inconclusive 26 vs 19. No Seikika run rehearsed refusals or a rejected push against the live
origins; each wrote `seikika test` action cases instead. But those test files, 66 to 74 lines per
repository, more than doubled Seikika's lines (166 to 368), and the lines gap to vanilla (326)
fell just inside W (329).

## Why, and what changed

Where the Seikika lines went (seikika-1; the others are within a few lines):

| Lines | File | Note |
|---|---|---|
| 3 × 72 | `<repo>/.seikika/tests/mark-deprecated.test.yaml` | Five or six cases, each repeating the action, requester, inputs and approvers |
| 3 × 44 | `<repo>/.seikika/seikika.yaml` | The action |
| 20 | `org/seikika.yaml`, `org/env.sh` | The org portal and its environment |

Every arm keeps one definition per repository (R1). `score.py` counts byte-identical copies once,
so vanilla's and GitHub Actions' per-repository files counted once. Seikika's three copies differ
in `portal.name` and in a precondition that names the repository
(`!inputs.successor.endsWith('/alpha')`), so each counted.

From the Seikika transcripts:

| Spent on | Fix (all after round 4's freeze) |
|---|---|
| Test files repeating every field in every case (lines) | A test file's `defaults` holds what every case shares; a case's `inputs` merge into them (`3c81d57`) |
| Retries after an unquoted comma inside `{…}` cut a value, reported as `which is better. is not an input` | The error says the key looks like the rest of a value cut at a comma (`72d0fa0`) |
| Schema lookups to find how to write `preconditions` and which fields a run's record has; a live run repeating what a passing case had shown | The `add-action` skill shows a precondition, lists the record's fields, and says to cite a passing case instead of rehearsing it live (`20ef2e4`) |

Not fixed, and why:

- **Per-repository differences.** `portal.name` is required in each repository's manifest, and
  nothing in a precondition names the portal it runs in, so a standard action can't be copied
  unchanged. Left for a later round if round 5 needs it: it changes the manifest contract.
- **Handoff writing** and **the rest of round 3's list** stand, for the reasons given there.

The adjudicators found no defects in the Seikika runs outside the probes. seikika-1 and seikika-3
answer refused approvals with 403 without appending them to the run's events, which the handoffs
state and R7 as probed does not require. Elsewhere: backstage-3 ran one approved run's scaffolder
task twice (both pushed the same commit) and returns stack traces in errors; gha-1 and gha-3 name
an unpushed local commit as `outcome.commit` after a rejected push; gha-3 cannot reach the GitHub
API from this machine, as its handoff says, so its GitHub-side settings were not exercised.

Round 5 is frozen at `8900f7b`, with every fix above. The fixes are recorded in
[D01M86J769VGE6K5FJJQ76EP4AA](../../../../docs/decisions/D01M86J769VGE6K5FJJQ76EP4AA.md) §16–17,
and the design they change is [federated-actions.md](../../../../docs/design/federated-actions.md)
§6.5.

## Measures

Raw measures per run: `<run>.check.json`, `<run>.lines.json` (before strikes), `<run>.count.json`,
`<run>.HANDOFF.md`. Lines after strikes are in each adjudication record; agents in
[runs.tsv](runs.tsv) and [adjudication/agents.tsv](adjudication/agents.tsv).
