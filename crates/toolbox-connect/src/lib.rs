//! Connector control (docs/design/connector-control.md): the server-side [`hub`], the
//! [`sdk`] connectors use, and the [`conformance`] kit.

pub mod conformance;
pub mod hub;
pub mod sdk;

pub use hub::{Command, Hub, Request, Status};
