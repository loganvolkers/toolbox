# seikika-model

Defines and validates the portal manifest, which is the configuration that tells Seikika
what to collect, map, and check. It also defines shared protocol and report types.

This Rust crate is part of the Seikika framework. Use it when maintaining Seikika's
internals.

## Where to start

For validation in Rust, start with `validate_portal`. It loads included files, validates
the schema and cross-references, and checks fixture messages.

- [Manifest reference](https://seikika.com/operators/manifest/)
- [Rust API source](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-model/src/lib.rs)
- [Build and check Seikika](https://seikika.com/start/run/)
