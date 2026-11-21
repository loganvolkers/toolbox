# AB2 arm A: phase 3 change

You are a platform engineer maintaining two coding standards that colleagues already set up. You didn't write them. This is a measured experiment, so follow these rules exactly:

- Work ONLY inside /tmp/claude-0/ab2/A/posthog and /tmp/claude-0/ab2/A/codex. Don't read or modify anything else, and don't look for other tools or repositories on this machine.
- No network. Commit locally with git when done (never push).
- At most 40 tool calls, then stop.

Each repository has its standard's artifacts under `.standards/`, described in `.standards/STANDARD.md`. Read that first.

## The change the team decided

1. **PostHog (S1, no direct ORM queries on person tables):**
   - Also count `Group.objects` and `GroupTypeMapping.objects` as violations, alongside the existing models.
   - Exempt `posthog/models/person/person.py`, with the reason "the model module; its managers back the routed helpers".
   - The new models' existing occurrences must be recorded in the baseline (they're the campaign's new work, not CI failures). New occurrences after this change must still fail the gate.
2. **Codex (S2, small Rust modules):** tighten the threshold from 800 to 700 lines. Files that are over 700 lines now must be recorded in the baseline. After the change, a recorded file may not grow, and an unrecorded file may not cross 700.
3. **In both:**
   - Keep the campaign meaningful. Decide what its starting point should be after the change, and document why.
   - Keep `.standards/STANDARD.md` accurate.

Verify that the Report, Gate and Campaign commands still work and reflect the change, then commit.

## Also keep a friction log

Every time you were unsure what to do, couldn't find information, hit an error, or had to work around something, note: what happened, roughly how many tool calls it cost, and what would have avoided it.

When done, reply with: (1) what you changed, file by file, (2) the Report counts after the change (S1: files and total; S2: files), (3) the campaign numbers after the change, (4) your friction log, and (5) how many tool calls you used.
