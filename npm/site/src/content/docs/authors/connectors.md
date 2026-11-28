---
title: Write a connector
description: Build a Rust connector, test its protocol, and inspect the records it emits.
sidebar:
  order: 2
---

A connector is a program that collects data and sends records to Seikika. This guide builds a sample connector that emits one record for the portal to map to a catalog entity.

## Build the scaffold

You need the [Seikika source checkout and CLI](/start/install/#build-from-source) and Rust. From the checkout root, run:

```bash
cd examples/connector-scaffold
cargo build
```

The scaffold pins its Rust toolchain and uses the SDK from the checkout. The build produces `target/debug/example-connector`.

## Test the protocol

From the scaffold directory, run:

```bash
seikika connector test --name example --portal .seikika -- target/debug/example-connector
```

Seikika reads the named connector's configuration from `.seikika/seikika.yaml`. Look for passing checks for startup, capabilities, and resync. A resync asks the connector to send its current records.

## Inspect a record

```bash
seikika connector drive --kinds example-item --format catalog \
  --config '{"seed":"alpha"}' -- target/debug/example-connector
```

The driver starts the connector and requests records for `example-item`. Look for an output line with `"kind":"example-item"` and `"id":"example:alpha"`. The default `records` format shows protocol fields such as `externalId`.

## Adapt the connector

1. Update the scaffold's SDK path if you move it outside this repository. Choose a published version and open its API reference. [SDK reference](/reference/sdks/), [Release notes](/start/versions/)
2. Implement the SDK's `Source` trait with a name, the version from `CARGO_PKG_VERSION`, capabilities, and a handler. Start it with `run_stdio`.
3. Read connector configuration from the `SEIKIKA_CONNECTOR_CONFIG` JSON. Use [logical secret names](/authors/secrets/) for credentials.
4. Update the example manifest's `records` and mappings. Run both commands again and check that the expected kind, ID, and fields arrive.

For message formats and remote connectors, see the [connector protocol](https://github.com/seikika/seikika/blob/main/docs/design/connector-control.md) and its [JSON Schema](https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/connector-messages.schema.json). For examples of how operators configure shipped connectors, see the [connector directory](/connectors/).
