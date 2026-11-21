# AB2 arm B′ (rerun): write S2 Codex

You are a platform engineer turning a team's written-but-unenforced coding standard into an enforced one, in a checkout of OpenAI Codex at /tmp/claude-0/ab3/B/codex. This is a measured experiment, so follow these rules exactly:

- Work ONLY inside /tmp/claude-0/ab3/B/codex, except that you may READ (never modify) only these parts of /home/user/toolbox: README.md, docs/v1-spec.md, docs/design/, docs/reference/, examples/, packages/, crates/, schema/ and plugins/. Don't read anything else there, and don't look for other tools or repositories on this machine.
- No network. You may commit locally with git (never push).
- At most 50 tool calls, then stop.
- Use **Toolbox** for this: a repo portal (a `toolbox.yaml`) scored by `toolbox report`, its ratchet (`--baseline`), and a campaign. The binaries are in /tmp/claude-0/ab3/bin: start every shell command with `export PATH=/tmp/claude-0/ab3/bin:$PATH;`. Put the portal under `.toolbox/` (plus a workflow under `.github/workflows/`). Where Toolbox's docs are a good start: /home/user/toolbox/docs/design/repo-portals.md, /home/user/toolbox/docs/design/campaigns.md, /home/user/toolbox/examples/repo-portal-starter/, /home/user/toolbox/packages/, and `toolbox --help` / `toolbox report --help`.

## The standard (S2: small Rust modules)

From the repository's AGENTS.md: "Target Rust modules under 500 LoC, excluding tests. If a file exceeds roughly 800 LoC, add new functionality in a new module instead of extending the existing file."

Precisely: a **violation** is a `.rs` file under `codex-rs/` with more than **800 lines** (count every line of the file). 500 is the target, not a violation. **Exempt:** test files: any file in a `tests/` directory, files named `tests.rs`, and files ending in `_tests.rs`.

There are many existing violations. The team wants a ratchet plus a campaign: a file already over 800 lines may not grow, a file at or under 800 may not cross 800, and progress toward zero violating files is visible.

## Deliverables

1. **Report:** one command that prints every violating file as JSON: `[{"file": "<path from repo root>", "lines": <n>}, ...]`.
2. **Gate:** one command that exits non-zero when the working tree breaks the ratchet (a violating file grew, or a file newly crossed 800 lines), and zero otherwise. It should also say when a file shrank or dropped below 800, so the baseline can be tightened. Add a GitHub Actions workflow that runs it on pull requests.
3. **Campaign:** one command that prints progress toward zero violating files: the baseline number of files, the current number, and the percentage done.
4. **`.toolbox/STANDARD.md`:** what the standard is, the exact commands for Report, Gate and Campaign (copy-pasteable, run from the repo root), how to update the baseline, and how to change the standard (change the threshold, exempt a file).

Verify each command works, then commit your work locally.

## Also keep a friction log

Every time you were unsure what to do, couldn't find information in Toolbox's docs, hit an error, or had to work around Toolbox, note: what happened, roughly how many tool calls it cost, and what would have avoided it (a doc, an example, a starter, a better error, a missing feature).

When done, reply with: (1) the files you created or changed, (2) the exact Report, Gate and Campaign commands, (3) the counts your Report gives (number of files), (4) your friction log, and (5) how many tool calls you used.
