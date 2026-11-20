//! Connector control (docs/design/connector-control.md): the server-side [`hub`], the
//! [`sdk`] connectors use, the [`conformance`] kit, and the shared [`assessment`] record.

pub mod assessment;
pub mod conformance;
pub mod files;
pub mod hub;
pub mod sdk;

pub use hub::{Command, Hub, Request, Status};
