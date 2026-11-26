# CLI Skills: One Skill for Every Seikika Command

*Status: catalog and scorecard built, 2026-11-20 (D94). Each `seikika` command has an agent skill
that names its invocation, and `cli-skill` fails a command that has none. Related:
[skill-standards.md](./skill-standards.md), [seikika-for-seikika.md](./seikika-for-seikika.md).*

---

## 1. Conclusion

**An agent that cannot find the command will not run it.** Help text is necessary and not
sufficient: it is not loaded until someone already knows the command. A skill is loaded when the
task matches its description.

| Question | What answers it |
|---|---|
| Does this command have a procedure an agent can be routed to? | `cli-skill` bronze: a `SKILL.md` mentions `seikika <command>` |
| Is that skill loadable and specific? | `maintainer-skill` (D72), unchanged |
| Does the skill raise task success? | Not gated. The same paired-run measurement D72 leaves out |

The match is the invocation, not a filename convention. The workspace connector already captures
`seikika <command>` from markdown, including `SKILL.md`, and links that doc to the command. A
command passes when one of those paths ends in `SKILL.md`. Nested commands are the leaves
`seikika describe` emits (`assess brief`, `connector test`, `describe portal`), so the skill must
name the leaf, not only the parent.

A new command must reach bronze (`ratchet.newEntitiesAtLeast`). The skill file itself must reach
bronze on `maintainer-skill`. Whether a harness loads the skill is not a gate.

## 2. The scorecard

`cli-skill`, kind `Command`, one level.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `has-skill` | `documentedIn` contains a path ending in `SKILL.md` |

The evidence is the invocation that is missing. The fix is to add
`plugins/seikika-operator/skills/<slug>/SKILL.md` with that invocation, the flags, and what not to
do.

Operator and user commands share the card. `operator-cli` and `user-cli` still score help text,
documentation, arguments and tests. Those do not say how to run the command.

## 3. This repository

One skill per leaf command lives under `plugins/seikika-operator/skills/<slug>/`. The slug is the
command with spaces replaced by hyphens, which is also the skill's `name`. The existing
`assess-entity` skill remains the judgement procedure; `assess-brief`, `assess-check` and
`assess-submit` are the commands that produce and check the file.

`seikika test portal` pins the rule both ways: the fixture skill for `validate` passes, and
`push`, which no skill names, fails with the missing invocation in the evidence.

## Sources

- [skill-standards.md](./skill-standards.md), D72. The skill file is scored there; this card only
  asks that the command has one.
- `crates/seikika-cli/src/main.rs`, the command list `seikika describe` walks.
- The workspace connector's `commands` capture, which links a doc that mentions an invocation to
  the command entity (`portal/seikika.yaml`).
