# seikika-operator agent plugin

An [Agent Plugins 1.0.0](https://github.com/agentplugins/agent-plugins-spec) package for Seikika
operators and their coding agents. It contains:
- [Agent Skills](https://github.com/agentskills/agentskills) for authoring, testing, planning and
  operating a portal, including one skill per `seikika` command (D94)
- a local stdio MCP server (`seikika mcp --dev`), started through `bin/seikika-mcp`

**Status:** pre-release skeleton. The plugin is MIT-licensed, like the project (D88). Each `seikika`
command has a skill under `skills/<slug>/` that names the invocation (D94).

Design: [docs/design/agent-plugins.md](../../docs/design/agent-plugins.md).

**Validation** (done before each commit):
- `plugin.json` and `mcp.json` validate against the official 1.0.0 JSON schemas.
- Every skill passes `skills-ref validate`.
