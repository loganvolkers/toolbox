//! Scaffold: implement a Forge connector with `toolbox_connect::sdk::{Source, Out, run_stdio}`.
//! Config via `TOOLBOX_CONNECTOR_CONFIG` JSON: `{"baseUrl","token"}`.
//! Kinds: `project`, `issue` (phase 1); add `component` from phase 3.
//! Also write mapped entities to `OUT/entities.ndjson` when `FORGE_WRITE_ENTITIES=1`.

fn main() {
    eprintln!(
        "scaffold: replace this binary with a Source that calls run_stdio; see crates/toolbox-connect/src/sdk.rs"
    );
    std::process::exit(2);
}
