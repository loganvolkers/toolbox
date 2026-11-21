# AB2 arm A: phase 4 reuse

You are taking part in a timed experiment. Work only in /tmp/claude-0/ab2/A/openfga (a git checkout of OpenFGA, a Go project). Budget: at most 40 tool calls. No network access, no package installs.

Context: another team already wrote and enforces a module-size standard for a Rust repository. Their artifacts are in /tmp/claude-0/ab2/A/codex/.standards/ (a script, config, baseline, and STANDARD.md) and the CI workflow in /tmp/claude-0/ab2/A/codex/.github/workflows/standards-s2.yml. Read them; do not modify anything under /tmp/claude-0/ab2/A/codex.

Task: apply the same standard to this Go repository, reusing as much of their work as is sensible:
- Target: Go files under 600 lines. A non-test `.go` file over 1,000 lines is a violation. `_test.go` files are exempt.
- Report every violating file with its line count, as a command that prints a JSON list `[{"file": "<repo-relative path>", "lines": <int>}, ...]` to stdout.
- A gate command that exits non-zero when a violating file grows or a new file crosses 1,000 lines, and exits zero (reporting the improvement) when violations shrink or for unrelated edits. Commit a baseline.
- Campaign progress: baseline count, current count, and percentage to the target of zero violations.
- A CI workflow that runs the gate.
- A STANDARD.md in this repo that states the exact report and gate commands, run from the repo root.
Commit your work with git in /tmp/claude-0/ab2/A/openfga.

Finally, reply with: the report and gate commands; the violation count; what you reused from the Codex artifacts and what you had to change (in lines); and a short friction log (anything that slowed you down).
