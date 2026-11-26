# Round 6: the bar fails against vanilla, on tool calls

Round 6 ran each arm three times against Seikika at `f172337`. Every run delivered: C1 and P1–P10
held in all twelve, as four independent adjudicators observed them
([ADJUDICATION.md](ADJUDICATION.md), records in [adjudication/](adjudication/)). Seikika beats
GitHub Actions and Backstage on both measures and vanilla on lines, but loses to vanilla on tool
calls, so the bar fails.

| Arm | Lines to maintain (median, range) | Tool calls (median, range) | Wall time (s, median) |
|---|---|---|---|
| Seikika | 139 (136–140) | 32 (28–33) | 252 |
| GitHub Actions | 918 (879–1187) | 51 (38–53) | 718 |
| Backstage | 2047 (1605–2311) | 63 (56–84) | 1009 |
| Vanilla | 784 (552–853) | 18 (12–21) | 321 |

| Rival | H2 lines | H3 tool calls | Bar |
|---|---|---|---|
| GitHub Actions | holds: 139 vs 918, W 308 | holds: 32 vs 51, W 15 | met |
| Backstage | holds: 139 vs 2047, W 706 | holds: 32 vs 63, W 28 | met |
| Vanilla | holds: 139 vs 784, W 301 | **lost**: 32 vs 18, W 9 | not met |

Round 5's standard files worked on lines: Seikika went from 350 to 139, because the action and
test files are now byte-identical in every repository and counted once. Calls barely moved
(31 to 32).

## Why, and what changed

Where the Seikika calls went (28 to 33), from the three transcripts:

| Spent on | Calls per run | Fix (all after round 6's freeze) |
|---|---|---|
| Reading the skills, `--help` and the example before writing anything | 5–7 | `seikika run --help` names parent-checkout approvals and the `add-action` skill; the `run` skill points to `add-action` alone (`d20a43a`) |
| Starting the org server (and, in seikika-2, patching its signing key into the action), then stopping it | 1–3 | A parent portal holds approvals in its checkout: `seikika run --as <person>` requests, `--approve <run>` decides, and the last approval signs the grant with a key kept in the state directory. No server to start or stop (`9b78410`, `53782e3`, D01M86WAVQQXYF9DFHCP3G035MA) |
| Copying the files to the other repositories, then committing and pushing | 1–3 | The `add-action` skill gives one fail-fast loop over the repositories (`d2fd0b6`) |

The vanilla runs were the fastest of any round on calls (12 at the low end). A vanilla agent
writes one script and runs it; it reads nothing first.
