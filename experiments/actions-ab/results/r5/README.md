# Round 5: the bar fails against vanilla, on tool calls

Round 5 ran each arm three times against Seikika at `8900f7b`. Every run delivered: C1 and P1–P10
held in all twelve, as four independent adjudicators observed them
([ADJUDICATION.md](ADJUDICATION.md), records in [adjudication/](adjudication/)). Seikika beats
Backstage on both measures, GitHub Actions on lines, and vanilla on lines, but loses to vanilla on
tool calls, so the bar fails.

| Arm | Lines to maintain (median, range) | Tool calls (median, range) | Wall time (s, median) |
|---|---|---|---|
| Seikika | 350 (330–367) | 31 (24–31) | 259 |
| GitHub Actions | 915 (737–1014) | 33 (32–46) | 585 |
| Backstage | 1894 (1847–2196) | 68 (62–84) | 1076 |
| Vanilla | 759 (582–797) | 16 (15–17) | 250 |

| Rival | H2 lines | H3 tool calls | Bar |
|---|---|---|---|
| GitHub Actions | holds: 350 vs 915, W 277 | inconclusive: 31 vs 33, W 14 | met |
| Backstage | holds: 350 vs 1894, W 349 | holds: 31 vs 68, W 22 | met |
| Vanilla | holds: 350 vs 759, W 215 | **lost**: 31 vs 16, W 7 | not met |

Round 4's `defaults` fix worked on lines: against vanilla, H2 went from inconclusive to a win. But
the vanilla runs were unusually tight on calls (15 to 17, W 2), and Seikika's 24 to 31 left W at
7, so a gap of 15 calls became a loss.

## Why, and what changed

Where the Seikika lines went (seikika-1; the others are within a few lines):

| Lines | File | Note |
|---|---|---|
| 3 × 74 | `<repo>/.seikika/tests/mark-deprecated.test.yaml` | Five or six cases, now with `defaults` |
| 3 × 45 | `<repo>/.seikika/seikika.yaml` | The action, plus the portal's identity |
| 10 | `org/seikika.yaml` | The org portal |

The three copies in each pair still differ: `portal.name`, a precondition naming the repository
(`inputs.successor != 'acme/alpha'`), and test values that name it. So `score.py`, which counts
byte-identical copies once, counted each of them.

Where the Seikika calls went (24 to 31; 20 to 26 of them shell):

| Spent on | Calls per run | Fix (all after round 5's freeze) |
|---|---|---|
| Reading the skills, an example, the schema and `--help` before acting | 5–7 | The `add-action` skill is one self-contained recipe, with a numbered procedure, the fleet layout, a complete action, its tests, and the serve, run and handoff commands (`1d83ab2`) |
| Writing and fixing three near-identical action and test files | 4–6 | One standard action file and one standard test file, byte-identical in every repository: a precondition names the portal it runs in through `entity.name`, a test names it through `{portal}`, and the repository's `seikika.yaml` is three lines that include the standard file (`673303e`, `c8bcb21`; D01M86J769VGE6K5FJJQ76EP4AA §18) |
| Rehearsing a rejected push and refusals against a live server, after `seikika test` had shown them | 2–4 | The skill says the passing cases are the evidence and gives the handoff's commands for them |

A defect the standard file exposed, fixed in `c8bcb21`: the workspace connector read a child's
`federation` (runner and approval keys) only from its root `seikika.yaml`, not from an included
file, so an org portal saw no runner for a child that declared it in a standard file.

Not fixed, and why:

- **Approvers are people, not a group.** Every Seikika run named `user:ann` and `user:bob` in each
  repository's approval rule, so adding a maintainer edits every repository. Seikika's approval
  rules already accept groups; the runs didn't use them because the task names two people. Left
  as a design note: it doesn't change either measure.
- **The approval key lives in the server's state.** Each repository pins the org server's public
  key, and the server generates it into its state directory on first start; a server started with
  another state directory can't approve. The handoffs say so. Left for the deployment track.

The adjudicators found no defects in the Seikika runs outside the probes. In the gha runs, gha-1
retries a push once after a `pre-receive` rejection, and gha-3's handoff says code-owner review
would be advisory on GitHub and an interrupted approval leaves a run stuck as running.
backstage-1 rounds some timestamps to the second, so `outcome.at` can read earlier than
`observed.at`; backstage-3 records the observation inside its final event and checks the file's
content on `main` rather than its own commit.

Round 6 is frozen at `f172337`, whose product is that of `8aa85df`, with every fix above.

## Measures

Raw measures per run: `<run>.check.json`, `<run>.lines.json` (before strikes), `<run>.count.json`,
`<run>.HANDOFF.md`. Lines after strikes are in each adjudication record; agents in
[runs.tsv](runs.tsv) and [adjudication/agents.tsv](adjudication/agents.tsv).
