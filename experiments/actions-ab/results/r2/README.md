# Round 2: the bar fails against vanilla, on tool calls

Round 2 ran each arm three times against Seikika at `ad36c06`. Every run delivered: C1 and P1–P10
held in all twelve, as four independent adjudicators observed them
([ADJUDICATION.md](ADJUDICATION.md), records in [adjudication/](adjudication/)). Seikika beat
GitHub Actions and Backstage on lines to maintain, and lost no measure to either. Against a vanilla
app it won on lines but lost on tool calls by more than W, so the bar, which needs a win against
every rival and no loss, fails.

| Arm | Lines to maintain (median, range) | Tool calls (median, range) | Wall time (s, median) |
|---|---|---|---|
| Seikika | 146 (146–161) | 29 (24–30) | 222 |
| GitHub Actions | 807 (747–871) | 29 (27–29) | 539 |
| Backstage | 1716 (1633–1724) | 66 (51–73) | 1188 |
| Vanilla | 644 (527–737) | 18 (17–21) | 283 |

| Rival | H2 lines | H3 tool calls | Bar |
|---|---|---|---|
| GitHub Actions | holds: 146 vs 807, W 124 | inconclusive: 29 vs 29, W 6 | met |
| Backstage | holds: 146 vs 1716, W 91 | holds: 29 vs 66, W 22 | met |
| Vanilla | holds: 146 vs 644, W 210 | **lost**: 29 vs 18, W 6 | not met |

Round 1's fixes worked where they were aimed. Seikika's lines fell from a median of 350 to 146, and
its range from 441 to 15: every run wrote three short `.seikika/seikika.yaml` files, an org manifest
and a start script, and no glue code. Its tool calls fell from 53 to 29, which now matches GitHub
Actions. Vanilla still needs 11 fewer.

## Why, and what changed

A vanilla run is mostly edits: it writes one program and its definitions, runs the three requests
and writes the handoff. A Seikika run writes less but spends its calls around the box. From the
transcripts (counts per run):

| Calls | Spent on | Fix (all after round 2's freeze) |
|---|---|---|
| 5–7 | Orientation: the skills, the schema and an example, to find how a repo declares an action and an org lists its children | Nothing new; the `run` skill already gives the loop. The field table in `examples/federated-actions` now lists every field, so the schema need not be read (`da71c5d`) |
| 3–4 | Starting the server in the background, waiting for it, and reading its approval key from the log | `seikika serve --detach` returns once the portal is ready and prints its URL, pid, log, approval key and run-records directory; `--stop` stops it (`b681579`, `29ca002`) |
| about 5 | Testing refusals and a rejected push on scratch copies, and checking that `succeeded` meant observed | `seikika run --verify` exits 4 for an unverified run (seikika-2 shipped `run-status.sh` because it exited 0, `d88e2f7`); a base that can't be fetched fails the run (`83081db`) |
| 2–3 | Reading run records, whose `approved_by` was `user:ann\|user:bob=user:bob` | Run records list approvers as plain identities, and `approval.satisfied` says which entry each met (`64ec5c8`) |
| — | Repeating `--url` and `--as` on every `seikika run` | `seikika run` reads `SEIKIKA_URL` and `SEIKIKA_AS` (`a56cfd2`) |

A dry run of the Seikika arm at `070d7fc` ([r3dry](../r3dry/)), not counted, found two more:

- **A literal `{successor}` in a commit message.** The `change` executor's `message` took no
  placeholders, so the agent's template was committed as written. A message now takes `{action}`,
  `{run}`, `{entity}` and `{inputs.<name>}`, and `seikika validate` refuses any other placeholder
  in a message or branch (`66edec9`).
- **A readiness race.** A preview right after `serve --detach` got `404 not-offered`: the portal
  answered `/readyz` before it had read its federation children. A portal with children is now
  ready once it has read them, and a request before then waits for that read (`18309d0`).

Not fixed, and why:

- **No OpenFGA in dev mode.** As in round 1, every Seikika run served without OpenFGA, so anyone,
  carol included, could request a run. Approval rules were enforced. Dev sign-in on a loopback
  address is a deliberate default (v1-spec §10.2).
- **Commits authored by Seikika.** The landed commit's author is Seikika, with `Seikika-Run`,
  `Requested-By` and `Approved-By` trailers. Authoring as the requester would mean inventing email
  addresses for identities that have none.
- **GitHub-side settings, not exercised.** No GitHub Actions run had a probe credited: each
  enforced approval in its own local glue, and configured GitHub environments, rulesets and
  CODEOWNERS as code that act cannot run. The adjudicators judged what ran.

### The harness

backstage-2 first started its backend in a tmux session named `backstage-backend`, which
backstage-1 had created in its own workspace on the same host. By backstage-2's account the
command never ran there and it did not use the session again; backstage-1 had already finished.
From round 3, each prompt tells the run to prefix tmux sessions and other named resources with its
run name (`070d7fc`). The prompts are otherwise unchanged.

Round 3 is frozen at `428ef81`, with every fix above. The fixes are recorded in
[D01M86J769VGE6K5FJJQ76EP4AA](../../../../docs/decisions/D01M86J769VGE6K5FJJQ76EP4AA.md), and the
design they change is [federated-actions.md](../../../../docs/design/federated-actions.md).

## Measures

Raw measures per run: `<run>.check.json`, `<run>.lines.json` (before strikes), `<run>.count.json`,
`<run>.HANDOFF.md`. Lines after strikes are in each adjudication record; agents in
[runs.tsv](runs.tsv) and [adjudication/agents.tsv](adjudication/agents.tsv).
