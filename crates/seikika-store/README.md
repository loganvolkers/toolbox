# seikika-store

Stores raw connector records and values that Seikika manages. It gives the derivation
pipeline a consistent input snapshot.

This Rust crate is part of the Seikika framework. Use it when maintaining Seikika's
internals.

## Where to start

Use `MemStore` for in-memory fixtures and `PgStore` for Postgres-backed portals. The crate
also loads newline-delimited JSON fixtures.

- [Deploy a shared portal](https://seikika.com/operators/deploy/)
- [Rust API source](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-store/src/lib.rs)
- [Build and check Seikika](https://seikika.com/start/run/)
