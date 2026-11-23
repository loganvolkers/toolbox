---
id: D01M83NX8BXH45GQA7WMN4ACEH9
decided: 2026-11-23T07:50:00Z
---

# connector-repair-ab: measure the connector repair loop

**A package-author DX Benchmark (tool fitting), not a bar change to connector-ab.**

[connector-ab](../../experiments/connector-ab/RESULTS-v3.md) measured author → change →
reuse → read. It did **not** measure the feedback loop when a near-correct connector fails
truth or conformance. In v3, every arm first failed phase 3 on composite component ids;
recovery was an external C3 rematch, not a scored phase. Agent feedback also stressed cold
builds, shim installs, and dual schemas — none were first-class hypotheses.

**Decision:** Add [experiments/connector-repair-ab](../../experiments/connector-repair-ab/README.md):

1. Each arm inherits a **seeded-broken** Forge connector (wrong composite ids, missing kind
   registration, extra catalog fields).
2. Phases: **diagnose** (from tool output / docs only) → **repair** to truth+shape green →
   **reuse** tenant B without rewriting sync → **read**.
3. Pre-registered claims **D1–D3** (cost to green, diagnosis recall, correctness). Same Forge
   mock and open-source pins as connector-ab; bar set before the first arm starts.

**What this adds:** error-feedback quality and repair cost — the DevEx "feedback loops"
dimension connector-ab left mostly qualitative.

**Consequences:** Catalog entry `connector-repair-ab`. connector-ab stays the authoring A/B;
this is a sibling, not a replacement.
