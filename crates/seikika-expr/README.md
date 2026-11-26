# seikika-expr

Checks and evaluates Common Expression Language (CEL) expressions for Seikika manifests
and catalog queries. It defines available variables and functions and exchanges values as
JSON.

This Rust crate is part of the Seikika framework. Use it when maintaining Seikika's
internals.

## Where to start

Validation checks syntax and available names. It does not perform full CEL type checking.

- [Manifest reference](https://seikika.com/operators/manifest/)
- [Rust API source](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-expr/src/lib.rs)
- [Build and check Seikika](https://seikika.com/start/run/)
