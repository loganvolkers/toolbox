# Round 3: the bar fails against vanilla again, on tool calls

Round 3 ran each arm three times against Toolbox at `428ef81`. Every run delivered: C1 and P1–P10
held in all twelve, as four independent adjudicators observed them
([ADJUDICATION.md](ADJUDICATION.md), records in [adjudication/](adjudication/)). Toolbox now beats
GitHub Actions on both measures and Backstage on both, and beats a vanilla app on lines. It still
loses to vanilla on tool calls by more than W, so the bar fails.

| Arm | Lines to maintain (median, range) | Tool calls (median, range) | Wall time (s, median) |
|---|---|---|---|
| Toolbox | 166 (154–169) | 27 (25–32) | 185 |
| GitHub Actions | 914 (863–941) | 41 (34–45) | 642 |
| Backstage | 1484 (1357–1544) | 62 (48–70) | 963 |
| Vanilla | 575 (493–705) | 13 (10–19) | 224 |

| Rival | H2 lines | H3 tool calls | Bar |
|---|---|---|---|
| GitHub Actions | holds: 166 vs 914, W 78 | holds: 27 vs 41, W 11 | met |
| Backstage | holds: 166 vs 1484, W 187 | holds: 27 vs 62, W 22 | met |
| Vanilla | holds: 166 vs 575, W 212 | **lost**: 27 vs 13, W 9 | not met |

Round 2's fixes held: no Toolbox run wrote glue code, and every run kept only three short
repository manifests, an org manifest and, in two runs, an environment file. Toolbox's calls fell
from 29 to 27. Vanilla's fell further, from 18 to 13, because its runs each wrote one program in one
or two edits and tested every refusal on scratch copies in one shell call.

## Why, and what changed

From the Toolbox transcripts (calls per run):

| Calls | Spent on | Fix (all after round 3's freeze) |
|---|---|---|
| about 10 | Rehearsing invalid inputs, self-approval, carol's approval and a rejected push against the live org portal and the real origins, with a `pre-receive` hook added to a real origin and removed again. Vanilla ran the same checks on scratch copies in one call | `toolbox test` runs action cases from the manifest's `tests`: a requester, approvers, an origin that rejects the push, and the expected state, refusals, error and files. Each case runs the real lifecycle in a scratch clone whose origin holds the base with the working tree on top, so nothing lands, nothing is recorded, and it works before the first commit (`0219541`, `e24b19d`) |
| 5–6 | Orientation: the `run` and `actions` skills, `--help` for `run` and `serve`, the example README and its manifests, and the `serve` skill, to piece together one loop. Vanilla spent 3 | An `add-action` skill, planned in [agent-plugins.md](../../../../docs/design/agent-plugins.md), puts the loop in one place: both manifests, testing, serving the org portal and pinning its key, and the request, approve and verify commands (`2f03a5a`) |
| 5 (toolbox-2) | A repeated request whose content `main` already had: the change committed nothing, so the run was reported unverified, and the agent added its own guard against no-op runs | A change its base already has succeeds without a commit, once the refetched base is seen to be the commit it ran on; the preview warns that nothing will change (`f1f79f2`) |

Not fixed, and why:

- **Handoff writing.** Toolbox runs spent 3–5 calls writing and revising `HANDOFF.md`, vanilla
  1–2. The handoff is the benchmark's own artifact, so shaping Toolbox to write it would break P5.
- **The rest of round 2's list** stands: dev mode without OpenFGA, commits authored by Toolbox, and
  GitHub-side settings not exercised.

The adjudicators found no defects in the Toolbox runs outside the probes.

Round 4 is frozen at `4825782`, with every fix above. The fixes are recorded in
[D01M86J769VGE6K5FJJQ76EP4AA](../../../../docs/decisions/D01M86J769VGE6K5FJJQ76EP4AA.md) §14–15,
and the design they change is [federated-actions.md](../../../../docs/design/federated-actions.md)
§6.3 and §6.5.

## Measures

Raw measures per run: `<run>.check.json`, `<run>.lines.json` (before strikes), `<run>.count.json`,
`<run>.HANDOFF.md`. Lines after strikes are in each adjudication record; agents in
[runs.tsv](runs.tsv) and [adjudication/agents.tsv](adjudication/agents.tsv).
