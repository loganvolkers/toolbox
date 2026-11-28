---
title: SDK documentation comments
description: Document the public Rust and npm APIs where consumers read them, then verify the comments and examples.
sidebar:
  order: 3
---

Write a short summary wherever a consumer can import a symbol, configure an option, or implement a trait. Follow it with the details they need to use the API correctly. Publish that guidance in rustdoc, editor hovers, and the package README. The general [documentation style guide](/maintainers/writing/) applies to all three.

## Document what callers can rely on

- Lead with one sentence in present tense. Use a verb for an operation, such as “Queues a connector command,” and describe what a type represents. Summaries appear alone in search results and hovers.
- Explain meaningful input constraints and output semantics. Include units, defaults, ordering, empty results, ownership, and side effects when they affect the caller. Let the signature show the types.
- Put prerequisites and failure conditions where the reader makes the decision. For async code, say when work starts, what completes it, and which runtime or cancellation behavior matters. For settings that expose data or change persistent state, state the effect before the example.
- When deprecating an API, use the language's deprecation annotation and point to the replacement or migration procedure.
- Cover public modules, types, constructors, functions, trait methods, fields, variants, and options. Keep one canonical description on the declaration. Add a neighboring implementation comment when a maintainer needs the reason for a tricky choice.
- Use a small, runnable example for the primary workflow and for behavior that is easy to misuse. Import from the public package path, show setup and a recognizable result, and use fixture data. Link related operations by name.

## Rust

Use `//!` for crate and module overviews and `///` for item comments. The crate overview should say who uses the crate, link to the main entry points, and show or link to a first task. Use intra-doc links such as `` [`Out::not_found`] ``. Link protocol details to the applicable schema. [The rustdoc book](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html), [Intra-doc links](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html)

Use `# Examples` on entry points and when a call sequence or edge case needs code. Use `# Errors` for fallible behavior and `# Panics` for caller-relevant panic conditions. A `# Safety` section on an unsafe API must state the invariants its caller is responsible for. Use these headings where the section applies. Explain requirements shared by a type's methods on the type itself.

Run examples as doctests. An example that waits for a portal or needs credentials can use `no_run` so the compiler still checks it. Say what the reader needs to run it. Keep imports and essential setup visible. Use `?` or check the returned value when showing recoverable failure. [Rustdoc doctests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)

From the repository root, run the connector SDK checks:

```bash
cargo test --locked -p seikika-connect --doc
cargo doc --locked --no-deps -p seikika-connect --open
```

Open the crate overview and follow the example and API links. Publish the generated reference on docs.rs and link to the crate from the connector guide. Use a versioned docs.rs URL when documenting a specific release. If a crate exposes optional features, configure the supported docs.rs feature set in Cargo and check the rendered availability notes. [Docs.rs build metadata](https://docs.rs/about/metadata)

## JavaScript and TypeScript

Put `/** … */` on the declaration that consumers see. For a JavaScript entry point with a hand-written declaration, document the public signature in the shipped `.d.ts` or `.d.mts`. Preserve comments when emitting declarations. Vue components use a named `Props` interface with a summary and comments on each public prop. Astro components use `Props` for typed props. Include slots, layout restrictions, and CSS variables in the package reference so they are discoverable before import. [TypeScript declaration publishing](https://www.typescriptlang.org/docs/handbook/declaration-files/publishing.html), [Vue props and TypeScript](https://vuejs.org/guide/typescript/composition-api.html#typing-component-props)

Use `@param name - …` for semantics that affect how a parameter is supplied. Add `@returns` for details such as ordering or a sentinel value. Use `@throws` for an exception or rejection the caller can handle, and say when it occurs. Use `{@link Name}` for a related declaration and `@example` when the snippet helps at the call site. Write consequential defaults and restrictions in prose so readers can see them in tools that render only a summary. [TSDoc tag reference](https://tsdoc.org/pages/tags/param/), [TSDoc examples](https://tsdoc.org/pages/tags/example/), [TSDoc exceptions](https://tsdoc.org/pages/tags/throws/)

Ship a README with a first task and an API reference under the package's public import paths. Use one `###` heading with the import path in backticks per entry point. List a component's slots in that section, using `default` for its unnamed slot. For published packages, check the exports in `package.json`, run `pnpm check`, and inspect `pnpm pack` output for the README and types. Try a real import in a consumer project. Keep longer, interactive examples in [Storybook](/storybook/).

## Review the change

| Check | How to verify it |
|---|---|
| A consumer can find the API | Open rustdoc, inspect the packed npm import path, or inspect the consumer's editor hover. |
| A consumer can choose and call it | Read the summary in isolation. Check documented inputs, output, defaults, and failure conditions against implementation and tests. |
| A consumer can follow the example | Run the doctest or the package example. Confirm the observable result in the stated environment. |
| Coverage stays at the new baseline | Run `node scripts/sdk-docs.mjs` for the raw evidence and the maintainer portal report for `maintainer-sdk-docs`. |

The scorecard checks public entry points, comments, and reference coverage. Review meaning and example quality by reading the rendered docs and running the examples. See the [SDK documentation audit](https://github.com/seikika/seikika/blob/main/docs/audits/sdk-documentation-2026-11-26.md) for the research and initial findings.

When adding another Rust SDK, include its library target in the SDK checker and link its generated reference from the author guides. Published npm workspaces enter the check through their package manifests.
