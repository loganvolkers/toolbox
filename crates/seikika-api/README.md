# seikika-api

Serves a configured portal over HTTP and the Model Context Protocol (MCP). It publishes
catalog snapshots and serves reads, action requests, and the portal UI.

This Rust crate is part of the Seikika framework. Use it when maintaining Seikika's
internals.

## Where to start

Start with the HTTP API guide for client requests. In the Rust API, see the routes, state,
and reads modules.

- [HTTP API guide](https://seikika.com/users/api/)
- [Rust API source](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-api/src/lib.rs)
- [Build and check Seikika](https://seikika.com/start/run/)
