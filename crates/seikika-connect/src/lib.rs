//! Write and test connectors that supply records to a Seikika portal.
//!
//! Start with [`sdk::Source`] and choose [`sdk::run_stdio`] for an in-image connector or
//! [`sdk::run_remote`] for a long-running process that polls the portal. A webhook receiver can
//! send unsolicited batches with [`sdk::push`].
//!
//! See the [`sdk`] module for a complete connector and [`conformance`] for the checks run by
//! `seikika connector test`. Portal hosts use [`hub`] to queue commands and track completion.
//! The [connector protocol] defines the JSON exchanged over both transports.
//!
//! [connector protocol]: https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/connector-messages.schema.json

pub mod assessment;
pub mod conformance;
pub mod files;
pub mod hub;
pub mod sdk;

pub use hub::{Command, Hub, Request, Status};
