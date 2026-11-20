# Plugin Standards: Cataloguing Agent Plugin Packages

*Status: deterministic catalog and scorecard built, 2026-11-20 (D80). Toolbox for Toolbox catalogues
each `plugin.json` and scores it with `maintainer-plugin`. Loading the package in a harness is not
a gate. Related: [agent-plugins.md](./agent-plugins.md),
[agents-md-standards.md](./agents-md-standards.md),
[toolbox-for-toolbox.md](./toolbox-for-toolbox.md).*

---

## 1. Conclusion

**A plugin package is a directory a plugin manager can load, not a promise that every harness behaves the same.**

The maintainer SDLC ships one package, `plugins/toolbox-operator`. Agent Plugins 1.0.0 says what
that directory must contain. Skills inside it stay on `maintainer-skill`. The checks that do not
need a harness:

| Question | What answers it | In this repo |
|---|---|---|
| Does the manifest match 1.0.0, and do skills sit in the one legal place? | `$schema`, the name grammar, a description, `skills/<name>/SKILL.md` | bronze |
| Is the MCP server declared in `mcp.json`, and is its stdio command one token? | The 1.0.0 mcp schema, a command with no shell | silver |
| Does the package keep secrets and paths inside itself? | Header values, `..` segments, secret shapes | gold |

A `license` field is not required (Q1). Hooks from the 1.1.0 proposal are not required. Two
harnesses loading the same package is not a gate.

## 2. What the field says

| Source | What it decides | Adopted |
|---|---|---|
| **Agent Plugins 1.0.0** | `plugin.json` and `mcp.json` schemas, a name of 1–64 characters, skills only at `skills/<name>/SKILL.md` | Those are the bronze and silver rules. The schema URLs are exact |
| **This repository's package** | `plugins/toolbox-operator` names `./bin/toolbox-mcp` and keeps skills under `skills/` | The live package is the gold example |
| **Q1** | The project licence is undecided | A missing `license` field does not fail |

## 3. The catalog

Glob: `plugins/*/plugin.json`. Id `plugin:<directory>`. `mcp.json` is read from the same directory
when it exists. The facts are `schemaOk`, `nameOk`, `descriptionChars`, `misplacedSkills`,
`hasMcp`, `mcpSchemaOk`, `commandToken`, `headerSecrets`, `pathEscapes` and `secretHits`.

| Level | Rule | Passes when |
|---|---|---|
| bronze | `manifest-valid` | The 1.0.0 plugin schema, a legal name, a description |
| bronze | `portable-components` | No `SKILL.md` outside `skills/<name>/` |
| silver | `mcp-declared` | `mcp.json` is present and uses the 1.0.0 mcp schema |
| silver | `stdio-command` | Every stdio `command` is one token, optionally under `./`, `${PLUGIN_ROOT}/` or `${PLUGIN_DATA}/` |
| gold | `no-secret-headers` | No header value matches a secret shape |
| gold | `paths-stay-inside` | No `..` segment in a command or argument |
| gold | `no-hardcoded-secrets` | No secret shape in the two JSON files |

## 4. Where this repository lands

| Package | Level | Notes |
|---|---|---|
| `plugins/toolbox-operator` | gold | Schema, name and description match 1.0.0. `mcp.json` names `./bin/toolbox-mcp`. Skills sit under `skills/` |

Fixtures: `plugin:plugins/demo/focused` at gold, `plugin:plugins/demo/partial` at bronze (no
`mcp.json`), `plugin:plugins/demo/unsafe` at none (a bad name, a shell command, a path escape, a
secret). Every rule passes on one fixture and fails on another (D71).

## Sources

- Agent Plugins 1.0.0 `plugin.schema.json` and `mcp.schema.json` at `agent-plugins.org`, the schema
  URLs this package already declares.
- This repository's `plugins/toolbox-operator/plugin.json` and `mcp.json`.
