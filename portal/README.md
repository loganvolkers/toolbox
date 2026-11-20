# Toolbox for Toolbox

*The portal Toolbox maintainers run on this repository. Design and findings:
[docs/design/toolbox-for-toolbox.md](../docs/design/toolbox-for-toolbox.md).*

Toolbox catalogues its own crates, docs, scripts and tests, the CLI commands and MCP tools it promises,
and scores each against one [SDLC layer's](../docs/sdlc.md) standard. It runs on a laptop or in CI:
no Postgres, no OpenFGA, no network.

| Scorecard | Layer | Kind | What it raises |
|---|---|---|---|
| `maintainer-crate` | 1. Maintainer | Crate | Crate docs, tests that pass, clippy, public-API docs, function size and complexity, unwraps |
| `maintainer-docs` | 1. Maintainer | Doc | Links resolve, the SDLC layer is named, design docs have a status banner and sources |
| `operator-cli` | 2. Operator | Command | Operator commands have help, are documented and are run by a script or test |
| `user-cli` | 3. User | Command | Read commands have help, structured output (`-o json`) and are exercised |
| `user-mcp` | 3. User | McpTool | Tools are described and annotated, inputs described, documented, and contract-tested over MCP |

## Run it

```bash
scripts/self-portal.sh report --static   # ~1 s: static facts + surfaces, every scorecard
scripts/self-portal.sh report            # ~2 min: also runs cargo test and clippy per crate
scripts/self-portal.sh serve             # live portal on :8090 (API, MCP, UI); connectors re-read on refresh
toolbox test portal                      # the standards' own tests, over the curated fixtures
```

With `serve` running, any agent can use it:

```bash
toolbox score crate:toolbox-derive --url http://127.0.0.1:8090 --as maintainer
toolbox query --kind McpTool --filter "entity.exercisedBy.size() == 0" --url http://127.0.0.1:8090 --as maintainer
toolbox refresh crate:toolbox-derive --url http://127.0.0.1:8090 --as maintainer   # reruns that crate's tests + clippy
toolbox mcp --url http://127.0.0.1:8090 --as maintainer                             # stdio MCP for an agent harness
```

To register it in an MCP client (for example Claude Code's `.mcp.json`), use the stdio bridge:
`{"mcpServers": {"toolbox-self": {"command": "target/debug/toolbox", "args": ["mcp", "--url", "http://127.0.0.1:8090", "--as", "maintainer"]}}}`.

## How it works

| Connector | Records | How | Refresh |
|---|---|---|---|
| `workspace` ([crate](../crates/toolbox-connector-workspace)) | `crate`, `doc`, `file` | `cargo metadata`, a `syn` parse of each crate, markdown and file scans with configurable regex captures and counts | Re-reads one crate, doc or file |
| `runner` ([crate](../crates/toolbox-connector-runner)) | `test`, `clippy`, `command`, `mcp-tool` | Runs `cargo test -p`, `cargo clippy -p` and `toolbox describe` | Reruns one crate's jobs |

Connectors report raw facts (every function with its length and complexity, every undocumented
public item); the manifest's scorecards set the thresholds. `fixtures/` is a trimmed snapshot of
this repo with hand-made runner results, and `tests/standards.test.yaml` pins what each rule means.
