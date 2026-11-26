---
id: D01M880QMWD7Z7FTGKJT99AHWBX
decided: 2026-11-25T00:30:00Z
---

# The report UI is TypeScript, bundled by Vite, on the same toolchain as the minimal UI

**`report-ui/src` is TypeScript, checked by `tsc` under `strict` and bundled by Vite in library
mode (IIFE) into the same embedded `crates/seikika-cli/src/seikika-report.js`. Vite and TypeScript
are pinned to the versions `ui/` uses.** This replaces the bundler half of point 1 of
D01M87PSM9YQGGW3YQ5X4XCJP63 (esbuild); the rest of that decision stands, including point 2: the
report stays framework-free.

1. **One toolchain.** `ui/` already builds with Vite 8.3.5 and TypeScript 7.0.2, and Storybook's
   builder for the report's stories is Vite. The report's bundle now uses the same Vite, so the
   repository has one bundler for its two UIs instead of two (D45). esbuild leaves `report-ui`'s
   direct dependencies; it remains a transitive one of Storybook.
2. **The data contract is a type.** `src/types.ts` declares `ReportData`, the JSON `seikika
   report` writes (`report_json` in `crates/seikika-cli/src/report.rs`), and every view and helper
   that takes the report is typed with it. A field renamed on one side and not the other now fails
   `tsc` on the UI side, where before it rendered `undefined`.
3. **Strict, with one exception.** `strict` is on, including strict null checks. `noImplicitAny`
   is off: with it on, `tsc` reports 389 unannotated parameters and 160 index expressions on
   untyped maps, and they are typed module by module. Turning it on is a later, separate change, not a precondition.
4. **Gates.** `npm run check` runs `tsc` and then fails when the committed bundle differs from a
   fresh Vite build; `npm run build` type-checks before it writes. CI's `report-ui` job runs both
   through `npm run check`. Measured here, `tsc` takes 0.25 s and the Vite bundle 0.2 s.
5. **Proved equivalent.** The port changed no output. Every story's shadow DOM was snapshotted
   before and after and compared (identical except a "days left" count that crossed midnight
   between runs), the probes passed 212 of 212, and the interactions no snapshot covers (filters,
   folds, jump links) were clicked against the old and the new bundle with the same result.

## Consequences

- Vite's output differs in formatting from esbuild's (tabs, double-quoted strings), so the
  committed bundle was rewritten once; tests that look for strings in it must not depend on
  quoting.
- A view can import types from `src/types.ts` and from the modules it uses; the element declares
  the views it installs on its prototype through an interface, so cross-view calls are checked.
- Bumping Vite or TypeScript is done in `ui/` and `report-ui/` together.
