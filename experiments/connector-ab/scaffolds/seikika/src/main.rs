//! Scaffold: implement a Forge connector with `seikika_connect::sdk::{Source, Out, run_stdio}`.
//! Config via `SEIKIKA_CONNECTOR_CONFIG` JSON: `{"baseUrl","token"}`.
//! Kinds: `project`, `issue` (phase 1); add `component` from phase 3.
//!
//! Override `Source::version` with `env!("CARGO_PKG_VERSION")` from *this* crate.
//! Prefer `seikika connector drive` (and the portal stub under `portal/`) over hand-built
//! stdio JSON. See `examples/connector-scaffold/` and the `author-connector` skill.

fn main() {
    eprintln!(
        "scaffold: replace this binary with a Source that calls run_stdio; see crates/seikika-connect/src/sdk.rs and examples/connector-scaffold/"
    );
    std::process::exit(2);
}
