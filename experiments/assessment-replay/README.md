# Historical assessment replay

This maintainer SDLC experiment tests the box with pinned Seikika PRs and reports each before/after or negative-control result as JSON. The [assessment audit](../../docs/audits/agentic-assessments-2026-11-26.md) interprets the evidence. [Results](results.json)

## Run

Use a full checkout with the commits referenced in the runner, Python 3.12 or later, the repository's Rust and Node toolchains, and pnpm. From the repository root:

```sh
cargo build --locked -p seikika-cli -p seikika-connector-workspace -p seikika-connector-runner
pnpm install --frozen-lockfile
pnpm --filter @seikika/astro-report exec playwright install chromium
python3 experiments/assessment-replay/replay.py
```

Set `PLAYWRIGHT_CHROMIUM_EXECUTABLE` to use a system Chromium. The browser case serves only the pinned historical HTML and fixtures on loopback. The SDK case uses the installed JS dependencies and needs the Cargo dependencies for the historical lockfiles in the local cache. The coverage case runs against the current checkout, so reproducing the recorded counts requires `ab60efa`'s portal inputs and a CLI built from that revision. `--cli PATH` selects the local binary. Run individual cases with repeatable `--case` flags.

| Case | Method |
|---|---|
| `assessment` | Runs the current assessment CLI with the repository's integration-test fixture. Computes the archived #108 predicate on those same observed rows for the old exit code. |
| `coverage` | Runs default and strict live self-portal checks with CI's connector exclusions. Counts rubric/subject pairs, including assessment reviews. |
| `browser` | Renders the historical JavaScript at `b15820e` and `08f071a` with the same fixtures from `08f071a`. Measures the CSS background and visible empty-state text. |
| `quickstart` | Executes the #126 quickstart Bash blocks in a disposable repo using the real CLI and a local mirror of the pinned starter files. The historical installation failure comes from the original audit's release observation. |
| `acceptance` | Executes the historical `m2-acceptance.sh` before and after #135. Stubs Postgres and CLI readiness, forces a fixture push failure, and compares exit, cleanup, and log visibility. |
| `sdk` | Applies the #139 coverage tool to its historical audit baseline and repaired source in separate temporary directories. |
| `rounds` | Recomputes recorded usability and audit counts. The two-extra-runs policy uses the observed round 14 plateau and models starts only. |

The replays cover known cases and repaired revisions. The recorded comparison contains zero model runs and measures call counts only where the historical audit gives enough information. Use separately held-out examples to compare general review accuracy or token spend.
