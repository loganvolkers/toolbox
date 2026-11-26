# seikika-connect

Defines how Seikika and connectors exchange records and commands. It provides the
connector software development kit (SDK), the server-side command hub, and a protocol
conformance kit.

Implement the SDK's `Source` trait to write a connector. The Seikika server uses the command
hub to queue requests and track their results.

## Where to start

If you are writing a connector, start with the connector author guide. Use the conformance
kit to check the capabilities your connector advertises.

- [Write a connector](https://seikika.com/authors/connectors/)
- [Rust API reference on docs.rs](https://docs.rs/seikika-connect)
- [Build and check Seikika](https://seikika.com/start/run/)
