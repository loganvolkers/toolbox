# `install-approved-app` UI bundle (specification)

A custom action UI (tier 3) built with the default Astro and Vue kit and mounted through the framework-agnostic
UI extension contract (D40, [docs/v1-spec.md](../../../../docs/v1-spec.md) §7.6).

- **Build:** `pnpm build` in this directory emits `dist/index.js` (an ES module). `toolbox build`
  copies it into the image; the manifest declares it under `ui.bundles` and mounts it in the
  `action:install-approved-app` slot.
- **Exports:** `mount(element, ctx) → { unmount() }`.
- **Uses from `ctx`:** `ctx.entity` (the repository), `ctx.action.options("app")` (the per-repo
  eligible apps with labels and disabled reasons), `ctx.action.preview(inputs)` (live preview with
  effects, warnings, blockers and the confirmation token), `ctx.action.run(token)` and
  `ctx.runs.watch(runId)` (the run timeline including "checking the result").
- **Must show:** eligible apps as a rich combobox; the live preview with permissions granted; the
  approval path; a confirm button bound to the current token; the run timeline ending at the
  verified state, or the `unverified`/`timed_out` warning.
- **Stories (default kit):** one per persona and state: repo admin, non-admin (disabled with reason),
  approval required (tier 0/1), preview stale, executor failure, verification timeout.
