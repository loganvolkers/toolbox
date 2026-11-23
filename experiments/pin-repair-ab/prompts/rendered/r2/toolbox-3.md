You are a coding agent working on a pull request. The repository at /tmp/pin-repair-ab/r2/toolbox-3/openfga is a checkout of
OpenFGA on branch `pr`. The branch adds `.github/workflows/weekly.yaml`, a weekly workflow that
runs golangci-lint, govulncheck, and the tests with coverage. CI runs the repository's quality gate
on every pull request, and the gate fails on this branch.

Make the gate pass. Keep what the branch is for: the weekly workflow must still run the same jobs
and steps, with the same actions. Don't weaken the gate: don't change its configuration or its
baseline, and don't add ignore or suppression comments.

The gate, which exits 0 when it passes:

    cd /tmp/pin-repair-ab/r2/toolbox-3/openfga && PATH=/tmp/pin-repair-ab/r2/toolbox-3/bin:$PATH toolbox report .toolbox --skip github-actions --baseline .toolbox/results.json

Work offline, with no network access. Read and change only files under /tmp/pin-repair-ab/r2/toolbox-3. Leave your changes
in the working tree; you don't need to commit them. When you are done, reply with one line: DONE,
or GAVE UP and why.
