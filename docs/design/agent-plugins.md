# Agent Plugins: One for Operators, One Generated for Users

*Status: design, 2026-11-19. Primary sources: the Agent Plugins specification repository
(`agentplugins/agent-plugins-spec`: 1.0.0 published, 1.1.0 working draft; JSON schemas) and the
Agent Skills specification (`agentskills/agentskills`, `docs/specification.mdx`). Both were read from
GitHub because agent-plugins.org and agentskills.io aren't reachable from this environment.*

## 1. The standard we target

**Agent Plugins** is a vendor-neutral package format for agent extensions.
- Core maintainers come from Amazon, Cursor, Microsoft, OpenAI and Vercel.
- The spec text is CC-BY-4.0 and the schemas are Apache-2.0.
- **1.0.0 is the published version and 1.1.0 is a working draft.** Seikika targets 1.0.0, and adds
  1.1.0 hooks only when that version is published.

| Rule (1.0.0) | Consequence for Seikika |
|---|---|
| A plugin is a directory with a **`plugin.json`** manifest whose schema is closed: `$schema`, `name`, `version`, `description`, `author`, `homepage`, `repository`, `license`, `keywords`, `extensions` | The workspace connector validates both `plugin.json` and `mcp.json` against the [published Agent Plugins 1.0.0 schemas](https://agent-plugins.org/schemas/1.0.0/plugin.schema.json), vendored under `crates/seikika-model/schema/agent-plugins-1.0.0/` with their Apache-2.0 license |
| `name`: 1–64 chars, `a-z0-9-.`, alphanumeric at both ends, no `--` or `..` | Portal names are normalised (`acme-portal`); `seikika build` fails on invalid names |
| Exactly two portable component types, at **fixed locations**: `skills/<name>/SKILL.md` and **`mcp.json`** | Everything else (client-specific marketplaces, extras) goes under reverse-domain extension directories (§8 of the spec) |
| Skills must conform to the **Agent Skills** spec: `name` (≤ 64 chars, lowercase/digits/hyphens, **must match its directory**), `description` (≤ 1024, says what *and when*); optional `license`, `compatibility` (≤ 500), `metadata`, `allowed-tools` (experimental). Name/description load at startup; the body loads on activation (< 5,000 tokens recommended) | Generated skills are small, keyword-rich and validated with `skills-ref`; detail lives in `references/`. Skills that exist in this repo are catalogued and scored by `maintainer-skill` ([skill-standards.md](./skill-standards.md), D72) |
| `mcp.json`: `$schema` + `mcpServers`. Variants: `stdio` (`command` is a single token, bare or `./…`; `${PLUGIN_ROOT}`/`${PLUGIN_DATA}` expansion) or `streamable-http` (`url`, HTTPS unless loopback; optional `headers`) | The operator plugin uses **stdio** for local authoring. The user plugin uses **streamable-http** pointing at production |
| **No secrets in `headers`; no OAuth fields.** Authorization is client-managed | The production MCP endpoint must support **MCP OAuth discovery** so clients can sign users in. A browser-only identity-aware proxy in front of `/mcp` won't work; use a separate MCP host or path with OAuth ([authorization.md](../authorization.md) §4.9) |
| Paths must stay within the plugin root; symlinks that escape are rejected | Bundled scripts and references are copied, never linked |
| 1.1.0 draft: portable **hooks** (`hooks.json`: `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `Stop`) | Planned for the operator plugin (§2.4) once 1.1.0 is published |

## 2. Plugin 1: `seikika-operator` (shipped in this repo)

**Audience:** Seikika operators and their coding agents, working *in a portal repo*: authoring kinds,
connectors, scorecards, actions, permissions and views, then testing, planning, upgrading and
operating.

**Location and lifecycle:**
- It lives at `plugins/seikika-operator/` in the Seikika repository and is **versioned and released with
  Seikika**: plugin `version` = Seikika version.
- Generated reference material (SDK API, CLI help, error codes, the manifest schema) is regenerated
  at release from the same sources as the docs, so skills never drift from the binary.

### 2.1 Layout

```
plugins/seikika-operator/
├── plugin.json
├── mcp.json                       # stdio: the local authoring MCP server
├── bin/
│   ├── seikika-mcp                # launcher (POSIX): finds or fetches the pinned `seikika`, verifies its signature, execs `seikika mcp --dev`
│   └── seikika-mcp.cmd            # launcher (Windows)
├── skills/
│   ├── seikika-authoring/         # the core loop: spec → tests → implement → test → simulate → plan → PR
│   ├── add-kind/
│   ├── extend-connector/          # record kinds, CEL mappings, act handlers, conformance kit
│   ├── add-scorecard/             # rules, levels, declassification, fixtures
│   ├── add-action/                # scopes, enumFrom inputs, executors, approvals, dry-run, MCP promotion
│   ├── add-permission/            # OpenFGA modules, .fga.yaml tests, derivation, CODEOWNERS authority
│   ├── write-cel/                 # typed CEL: predicates and object construction, common errors
│   ├── write-executor/            # in-image executors (WASM functions deferred, D41)
│   ├── add-view/                  # operator UI views (React/TSX on @seikika/ui), story first
│   ├── add-action-ui/             # generated → field widget → custom action UI; Storybook page stories + play tests
│   ├── review-plan/               # reading plan output and gates; when to acknowledge
│   ├── debug-simulation/          # replaying seeds, reading invariant failures
│   ├── upgrade-seikika/           # version bumps, codemods, manifest migrations
│   ├── operate-portal/            # admin ops, revocation/lag/backup runbooks
│   └── migrate-from-backstage/    # import catalog-info, map plugins to Seikika features
└── README.md                      # licence field added once the project licence is decided
```

### 2.2 MCP server (local authoring)

```json
{
  "$schema": "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json",
  "mcpServers": {
    "seikika-dev": { "type": "stdio", "command": "./bin/seikika-mcp", "args": ["--data", "${PLUGIN_DATA}"] }
  }
}
```

- **Tools:** `validate`, `test`, `simulate`, `plan` (remote, as the developer), `explain_error`,
  `docs_search`, `example`, `preview_url`, `scaffold` (new kind/action/scorecard from templates).
- **Workspace:** the spec runs stdio servers with the *plugin root* as the working directory, and MCP
  roots are deprecated. So every tool takes an explicit `workspace` argument, the portal repo root,
  and the `seikika-authoring` skill tells agents to pass it.
- **Version pinning:** the launcher reads `seikika.lock` in the workspace and runs exactly that Seikika
  version. Binaries are cached under `${PLUGIN_DATA}` and verified with Sigstore/cosign before
  execution. It never relies on whatever happens to be on `PATH` (the spec says plugins must not
  depend on that).

### 2.3 Skills: principles

- **Each skill is one task,** and its description names the trigger words agents see ("add a
  scorecard", "OpenFGA", "fga.yaml", "permission", "declassify"…).
- **Bodies stay short.** They give the procedure and the exact `seikika` commands, and link to
  `references/` (generated API and schema excerpts, worked examples like
  [approved-github-apps.md](../examples/approved-github-apps.md)).
- **Test-first by default:** every authoring skill starts by writing properties and failing tests
  ([verification.md](./verification.md) §7).
- **The plugin is evaluated.** The agent-authoring eval suite ([plan.md](../plan.md) §7) runs *with
  this plugin installed*. Changes to skills ship only if eval success rate, retries and tokens don't
  regress.

### 2.4 Hooks (when Agent Plugins 1.1.0 is published)

- `SessionStart`: print the portal status (Seikika version vs. lock, last test result, pending plan
  gates).
- `PostToolUse` (file modification under `modules/`, `authz/` or `src/`): run `seikika check
  --changed` and return diagnostics, so the agent sees type and CEL errors immediately.

## 3. Plugin 2: the user plugin (generated from the portal's config)

**Audience:** everyone who uses the portal from an agent (engineers in Claude Code, Codex, Cursor,
VS Code and others).
- It **points at the production MCP URL**.
- It is **generated by `seikika build`** from the manifest, so it always matches the deployed portal.

### 3.1 What gets generated

```
dist/agent-plugin/acme-portal/
├── plugin.json            # name from the portal (normalised), version = portal release, description, homepage = portal UI
├── mcp.json               # streamable-http → https://portal.acme.dev/mcp (OAuth handled by the client)
└── skills/
    ├── acme-portal/                 # overview: what's in this portal, how to use its MCP tools efficiently
    ├── find-owner/                  # ownership, on-call, escalation via get/graph/context
    ├── fix-scorecard-failures/      # explain → fix text → remediation actions (dry-run first)
    ├── build-dashboard/             # trend/rollup/history tools → Claude Artifacts / Cursor Canvas
    ├── team-standards/              # how to fetch live standards (Skill entities) for the current repo
    ├── install-approved-github-app/ # one per promoted action: when to use it, eligibility, dry-run, approvals
    └── <operator-authored skills>/  # copied verbatim from agent/skills/ in the portal repo, validated
```

**Generation rules:**
- **Config becomes baked skills; data stays live.** Kinds, scorecards, actions, MCP tool names and
  descriptions come from the manifest and are baked into skills. Catalog *data* is never baked in.
  That covers team standards (`Skill`/`Rule` entities), owners and scores, which the skills tell the
  agent to fetch live over MCP (resources, prompts, `context`). Plugin content therefore never goes
  stale and never leaks restricted data.
- **One skill per promoted action** (e.g. `install-approved-github-app`). Its body:
  1. when to use the action
  2. how to check eligibility
  3. **always call with `dry_run` first** and show the result to the user
  4. the approval path
  5. how to watch the run

  The description is derived from the action's agent-facing description, within the 1,024-character
  limit.
- **Audience filtering:** only actions and kinds visible to *portal members* are included. Operator
  and admin capabilities are excluded, and actions can opt out with `agentPlugin: { include: false }`.
  `plan` lists everything that would enter the plugin, so nothing is published by accident.
- **Names are validated** against both specs. Skill directory names equal their `name`.
- **Environment binding:** the MCP URL is an environment value
  (`seikika build --plugin-env prod`), consistent with "deploy time can only bind values". Staging
  and preview environments get their own plugin builds pointing at their own URLs.

```json
{
  "$schema": "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json",
  "mcpServers": {
    "acme-portal": { "type": "streamable-http", "url": "https://portal.acme.dev/mcp" }
  }
}
```

### 3.2 How it stays current

- **Part of the build:** `seikika build` emits the plugin next to the OCI image. The plugin's
  `version` is the portal release version, which clients use for update checks.
- **Part of the plan:** the "Agent surface" section of `seikika plan` now includes *plugin* changes
  (skills added, removed or changed, and the MCP URL) alongside MCP tool changes.
- **Published by CI after deploy and verify,** so the plugin never points at tools that aren't live:
  - a release artifact
  - the portal serves it for download (`/agent-plugin.tgz`, with `seikika plugin install
    https://portal.acme.dev` as a one-liner)
  - optional thin wrappers for client marketplaces, generated under reverse-domain extension
    directories, or as separate marketplace manifests where a client requires its own format
- **Consistency tests:**
  - every MCP tool referenced by a generated skill exists in the build's MCP surface snapshot
  - every skill passes `skills-ref` validation
  - the manifests validate against the official schemas
  - the user-agent eval tasks run against the preview environment with the generated plugin
    installed, e.g. "add Renovate to payments-api" must dry-run before running

## 4. How the two plugins relate

| | Operator plugin | User plugin |
|---|---|---|
| Source | Seikika repo (`plugins/seikika-operator/`) | Generated from each portal's manifest |
| Versioned with | Seikika releases | The portal's releases |
| MCP | stdio, local authoring server (`seikika mcp --dev`) | streamable-http, production `/mcp` |
| Skills | Authoring, testing, planning, operating Seikika | Using *this* portal: ownership, standards, actions, dashboards |
| Auth | Local; remote `plan` uses the developer's identity | Client-managed MCP OAuth against the portal |
| Evaluated by | Agent-authoring eval suite | User-agent eval suite on the preview environment |

**Operator-authored user skills.** Operators can add hand-written skills to the user plugin by
putting them in `agent/skills/<name>/SKILL.md` in the portal repo. They are validated, included in
`plan`, and owned like any other definition: a meta-catalog `Skill` entity with an owner.

## 5. Status in this repo

A validated skeleton is committed at [`plugins/seikika-operator/`](../../plugins/seikika-operator/):
- `plugin.json` and `mcp.json` pass the official 1.0.0 JSON schemas.
- Four skills (`seikika-authoring`, `add-scorecard`, `add-permission`, `review-plan`) pass
  `skills-ref validate`.
- The MCP launcher is a placeholder until the `seikika` binary exists.

## 6. Changes to the plan

- `seikika build` emits the user plugin. `seikika plugin build|validate|install` are new CLI commands.
- The plan's agent-surface section covers plugin changes.
- The release pipeline validates and publishes the operator plugin.
- The meta catalog gains `AgentPlugin` and `Skill` (plugin-sourced) entities with owners. A new
  `plugin-quality` default scorecard checks: valid manifests and skills; every referenced tool exists;
  eval pass rate.
- **Phasing:**
  - **Phase 1:** operator plugin v1 (authoring loop, `add-*` skills, local MCP); user plugin v1
    (overview, find-owner, fix-scorecard-failures, team-standards; streamable-http).
  - **Phase 2:** per-promoted-action skills; `build-dashboard`; publishing pipeline; plugin diff in
    `plan`.
  - **Phase 3+:** 1.1.0 hooks once that version is published.

## Sources

- Agent Plugins:
  - [agentplugins/agent-plugins-spec](https://github.com/agentplugins/agent-plugins-spec)
    (`spec/1.0.0.md`, `spec/1.1.0.md`, `schemas/1.0.0/*.schema.json`, `MAINTAINERS.md`, `LICENSE.md`)
  - [agent-plugins.org](https://agent-plugins.org/)
  - Secondary coverage: [Agent Plugins 1.0 for Codex](https://codex.danielvaughan.com/2026/08/08/agent-plugins-1-0-open-standard-codex-cli-portable-skills-mcp-packaging/)
- Agent Skills:
  - [agentskills/agentskills](https://github.com/agentskills/agentskills) (`docs/specification.mdx`,
    `skills-ref`)
  - [agentskills.io/specification](https://agentskills.io/specification)
