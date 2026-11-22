# Phase 2, arm A: the agent's reply (DX feedback in §3)

1. Campaign `semgrep-to-zero` started at 209 (all suppressed), recorded in .github/quality-gate/baseline.json under `campaigns` with start 209, started 2026-11-22 and a per-rule split (104 + 105); `accept` keeps campaigns. quality_gate.py: `campaign` records per-rule starts; `progress` prints one line per campaign plus per rule. ci-quality-gate.yml: a "Campaign progress" step writes progress to the job summary. TOOLS.md: a Campaigns section (how to pay a finding down) and the ruff decision: no campaign, the ratchet at 0 already fails on the first new finding. Gate exit 0 (~22 s); a planted new nosemgrep line failed it (exit 1) and was restored.
2. Couldn't: tell suppressed from active findings in `progress`: semgrep's JSON with --disable-nosem doesn't flag them.
3. DX feedback:
   1. A Bash run (gate plus progress) hit the 120 s limit, moved to the background, left an empty output file and no process, and hadn't restored the edited file: restored by hand and reran with a timeout.
   2. semgrep rule ids depend on the --config path (`semgrep.rules.<id>`), so `campaign --rule` needs the full form; a bare id silently matches nothing.
   3. Each gate or progress run is a full semgrep scan (~20 s+), slower after an edit; CI now runs semgrep twice (gate, then progress) because progress doesn't reuse the gate's findings.
