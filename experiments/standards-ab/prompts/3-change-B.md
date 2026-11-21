# AB2 arm B: phase 3 change

You are a platform engineer maintaining two coding standards that colleagues already set up with Toolbox. You didn't write them. This is a measured experiment, so follow these rules exactly:

- Work ONLY inside /tmp/claude-0/ab2/B/posthog and /tmp/claude-0/ab2/B/codex, except that you may READ (never modify) Toolbox's docs and examples in /home/user/toolbox. Don't look for other tools or repositories on this machine.
- No network. Commit locally with git when done (never push).
- At most 40 tool calls, then stop.
- The Toolbox binaries are in /tmp/claude-0/ab2/bin: start every shell command with `export PATH=/tmp/claude-0/ab2/bin:$PATH;`.

Each repository has its standard's artifacts under `.toolbox/`, described in `.toolbox/STANDARD.md`. Read that first.

## The change the team decided

1. **PostHog (S1, no direct ORM queries on person tables):**
   - Also count `Group.objects` and `GroupTypeMapping.objects` as violations, alongside the existing models.
   - Exempt `posthog/models/person/person.py`, with the reason "the model module; its managers back the routed helpers".
   - The new models' existing occurrences must be recorded in the baseline (they're the campaign's new work, not CI failures). New occurrences after this change must still fail the gate.
2. **Codex (S2, small Rust modules):** tighten the threshold from 800 to 700 lines. Files that are over 700 lines now must be recorded in the baseline. After the change, a recorded file may not grow, and an unrecorded file may not cross 700.
3. **In both:**
   - Keep the campaign meaningful. Decide what its starting point should be after the change, and document why.
   - Keep `.toolbox/STANDARD.md` accurate.

Verify that the Report, Gate and Campaign commands still work and reflect the change, then commit.

## Also keep a friction log

Every time you were unsure what to do, couldn't find information in the artifacts or in Toolbox's docs, hit an error, or had to work around Toolbox, note: what happened, roughly how many tool calls it cost, and what would have avoided it (a doc, an example, a better error, a missing feature).

When done, reply with: (1) what you changed, file by file, (2) the Report counts after the change (S1: files and total; S2: files), (3) the campaign numbers after the change, (4) your friction log, and (5) how many tool calls you used.
