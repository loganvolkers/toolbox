# AB2 arm A: write S1 PostHog

You are a platform engineer turning a team's written-but-unenforced coding standard into an enforced one, in a checkout of PostHog at /tmp/claude-0/ab2/A/posthog. This is a measured experiment, so follow these rules exactly:

- Work ONLY inside /tmp/claude-0/ab2/A/posthog. Don't read or modify anything outside it, and don't look for other tools or repositories on this machine.
- No network. You may commit locally with git (never push).
- At most 50 tool calls, then stop.
- Use whatever you like: Python, the shell, git. Put your artifacts under `.standards/` (plus a workflow under `.github/workflows/`).

## The standard (S1: no direct ORM queries on person tables)

From the repository's AGENTS.md: "Never add new direct ORM queries like `Person.objects.filter(...)` or `PersonDistinctId.objects.filter(...)` — use the existing routed helpers or create new ones following the established pattern."

Precisely: a **violation** is each occurrence of `Person.objects`, `PersonDistinctId.objects` or `CohortPeople.objects` in a Python file. **Exempt:** test files (any file under a `test/` or `tests/` directory, `test_*.py`, `*_test.py`, `conftest.py`) and the routed helpers that are allowed to query these tables: `posthog/models/person/util.py` and `posthog/models/group_type_mapping.py`.

There are many existing violations. The team wants a ratchet plus a campaign: existing violations are recorded and may only go down, new ones fail CI, and progress toward zero is visible.

## Deliverables

1. **Report:** one command that prints every violation by file as JSON: `[{"file": "<path from repo root>", "count": <n>}, ...]`.
2. **Gate:** one command that exits non-zero when the working tree has a violation the recorded baseline doesn't allow (a new file with violations, or more in a file than recorded), and zero otherwise. It should also say when violations went down, so the baseline can be tightened. Add a GitHub Actions workflow that runs it on pull requests.
3. **Campaign:** one command that prints progress toward zero: the baseline count, the current count, and the percentage done.
4. **`.standards/STANDARD.md`:** what the standard is, the exact commands for Report, Gate and Campaign (copy-pasteable, run from the repo root), how to update the baseline after fixing violations, and how to change the standard (add a model, add an exemption).

Verify each command works, then commit your work locally.

## Also keep a friction log

Every time you were unsure what to do, couldn't find information, hit an error, or had to work around something, note: what happened, roughly how many tool calls it cost, and what would have avoided it.

When done, reply with: (1) the files you created or changed, (2) the exact Report, Gate and Campaign commands, (3) the counts your Report gives (files and total), (4) your friction log, and (5) how many tool calls you used.
