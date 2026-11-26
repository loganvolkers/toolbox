# seikika-secrets

Resolves connector secret references when a portal runs. It checks environment variables
and files before configured remote secret stores.

This Rust crate is part of the Seikika framework. Use it when maintaining Seikika's
internals.

## Where to start

Keep references in the manifest and supply values through a configured backend. The
operator guide explains backend order and setup.

- [Configure connector secrets](https://seikika.com/operators/secrets/)
- [Rust API source](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-secrets/src/lib.rs)
- [Build and check Seikika](https://seikika.com/start/run/)
