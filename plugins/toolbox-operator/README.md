# toolbox-operator agent plugin

An [Agent Plugins 1.0.0](https://github.com/agentplugins/agent-plugins-spec) package for Toolbox
operators and their coding agents. It contains:
- [Agent Skills](https://github.com/agentskills/agentskills) for authoring, testing, planning and
  operating a portal
- a local stdio MCP server (`toolbox mcp --dev`), started through `bin/toolbox-mcp`

**Status:** pre-release skeleton. The `toolbox` binary does not exist yet, so the MCP launcher exits
with an explanatory error, and skills describe the planned CLI. The plugin is MIT-licensed, like
the project (D88).

Design: [docs/design/agent-plugins.md](../../docs/design/agent-plugins.md).

**Validation** (done before each commit):
- `plugin.json` and `mcp.json` validate against the official 1.0.0 JSON schemas.
- Every skill passes `skills-ref validate`.
