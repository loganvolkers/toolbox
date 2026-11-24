# Dry run before round 2 (not counted)

One fresh agent ran the Toolbox arm's round 1 prompt against Toolbox at `aa68987`, the fixes from
round 1's friction included, to find what was left before round 2 is frozen. It is not one of the
benchmark's runs and no rival ran beside it.

| Measure | Value |
|---|---|
| C1 (each origin's `main` has the exact `DEPRECATED.md`) | pass |
| Tool calls | 41 (round 1's Toolbox runs so far: 53, 51) |
| Wall time | 299 s |
| Authored lines (run records aside) | about 230: three 44-line repo manifests, `org/toolbox.yaml`, a 16-line `org/bin/serve`, and a 61-line `org/bin/check-deprecated` |

Raw measures: [toolbox-1.check.json](toolbox-1.check.json),
[toolbox-1.lines.json](toolbox-1.lines.json), [toolbox-1.count.json](toolbox-1.count.json),
[toolbox-1.HANDOFF.md](toolbox-1.HANDOFF.md).

## Friction left, and what changed

| Calls | Friction | Fix |
|---|---|---|
| 11–13, 23–24, 31, 33–36 | The agent believed `succeeded` did not re-read the file ("checking the text would need a custom connector") and wrote `check-deprecated` to do it. Toolbox already compared every changed file with the fetched base; the record just didn't say so | The record's `verification.files` lists each file's blob and whether the base has it, with a `checked` sentence; the `run` skill says no extra check is needed. A `change` whose diffs differ from the confirmed preview now fails before it commits |
| 25 | The first commit was authored by the host's global git identity, so the agent restarted the server with `GIT_AUTHOR_*` set | The org portal's clones commit as Toolbox; the commit also carries `Approved-By` |
| 9–10 | `not` is outside the form subset; the agent moved "not its own successor" to a precondition | None: the precondition is the right place, and the error named the keyword |

Decision: [D01M86J769VGE6K5FJJQ76EP4AA](../../../../docs/decisions/D01M86J769VGE6K5FJJQ76EP4AA.md) §9.
