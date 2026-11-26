# Full recompute experiment

This standalone Rust experiment measures how the same Common Expression
Language (CEL) rules behave as the number of catalog entities grows. It also
measures group membership closure and ownership counts over a synthetic graph.

With Rust, Cargo, and network access to fetch the pinned CEL dependency, run
this smoke check from `experiments/full-recompute`:

```sh
cargo run --release -- 100 2
```

Look for `entities=100 rules=2 evals=200` in the thread measurements and a
`closure users=10000` summary. The arguments set the entity and rule counts.

See the [experiments index](../README.md) for the benchmark scale and the
[adversarial critique](../../docs/adversarial-critique.md) for the recorded
findings. The maintained pipeline lives in [seikika-derive](../../crates/seikika-derive/README.md).
