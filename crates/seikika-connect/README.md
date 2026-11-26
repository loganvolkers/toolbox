# seikika-connect

Defines how Seikika and connectors exchange records and commands. It provides the
connector software development kit (SDK), the server-side command hub, and a protocol
conformance kit.

This Rust crate is part of the Seikika framework. Use it when maintaining Seikika's
internals.

## Where to start

If you are writing a connector, start with the connector author guide. Use the conformance
kit to check the capabilities your connector advertises.

- [Write a connector](https://seikika.com/authors/connectors/)
- [Rust API source](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-connect/src/lib.rs)
- [Build and check Seikika](https://seikika.com/start/run/)
