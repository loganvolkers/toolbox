---
id: D01M83FFM2K6XS4EAVHVP92TBDZ
decided: 2026-11-23T06:00:00Z
---

# Close connector authoring learning tax

**A package-author SDLC fix (not a benchmark-bar change).** The connector A/B
([experiments/connector-ab](../../experiments/connector-ab/RESULTS.md),
D01M833YAH8MT4VY2W4K1PYAEHB) failed C1 and C2. Phase 1 on Seikika cost 53 tool calls
versus 29–41 on rivals. Agent feedback (friction 2–5): inventing a portal for
`connector test`, hand-building stdio JSON to dump records, `Source::version()`
reporting the connect crate version, and env/manifest config drift. Learning, not
protocol correctness, was the loss.

**Decision:** Attack the learning tax in product, not by softening the benchmark bar.

1. Ship `examples/connector-scaffold/` (portal stub + Rust `Source` + toolchain pin).
2. Add `seikika connector drive` for one-shot resync dumps without hand-built JSON.
3. Require `Source::version()` on every implementor (no SDK-crate default).
4. Add the `author-connector` and `connector-drive` skills, and a public Astro page
   `authors/connectors.md`.
5. Point the connector-ab Seikika scaffold at that scaffold/portal pattern so the
   next run measures product DX, not missing stubs.

**Consequences:** Package-author `build-connector` still lacks a published crates.io
SDK and non-Rust SDKs; those gaps stay listed. The A/B bar (C1–C3) stays as
pre-registered. Reruns use the same truth and call budgets.
