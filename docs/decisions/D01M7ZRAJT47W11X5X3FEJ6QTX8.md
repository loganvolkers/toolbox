---
id: D01M7ZRAJT47W11X5X3FEJ6QTX8
decided: 2026-11-21T17:00:00Z
---

# Seikika is a box for tools, not a wrapper or a factory

**Seikika's main job is to be a box for tools: a home for them, a standard way to carry, pin, run
and read them together, and the memory of what they found. It is not coupled to any tool. A
better tool replaces a worse one without changing the box, and any tool can be taken out and used
directly, with the same command, configuration and findings.**

1. **Tools stay independent.** A tool works without Seikika. The box doesn't add flags, ignores or
   filters the tool doesn't know about. A tool's configuration lives in the tool's own file.
2. **Standard slots before fittings.** Findings come in through standard shapes (SARIF, NDJSON
   records, JUnit). A per-tool fitting (parser) is a stopgap until the tool has standard output.
3. **Seikika builds tools only where none exists.** Prefer an existing tool, then a plugin to an
   existing framework (an ESLint, semgrep, ast-grep or clippy rule), then a standalone tool with
   standard output that is usable without Seikika. A jig made from Seikika's own primitives
   (workspace `counts`, CEL) is for a proof of concept, and graduates to one of those when reused.
   Seikika doesn't compete with the compiled linters, formatters and analysers that ecosystems
   converge on.
4. **Every design doc and package README says which side it is on:** the box (carrying, pinning,
   running, remembering, reporting, distributing), or a tool (an existing tool, a fitting, a jig,
   or a standalone tool built with Seikika's parts).
5. **Seikika for Seikika is reviewed in this light.** Its home-made extractors are replaced by
   existing tools where those exist, extracted as standalone tools where they would be used
   outside Seikika, and kept in the box only when they describe the box itself.

**Status:** ✅ (the vision and the writing rule); the follow-up work is ranked in the design doc

**Where:** [docs/design/seikika-and-tools.md](../design/seikika-and-tools.md)
