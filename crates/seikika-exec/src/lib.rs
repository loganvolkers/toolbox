//! Action runs (docs/design/action-runs.md, docs/design/federated-actions.md): the Seikika layer of
//! a preview ([`check`]), executors ([`executor`], with the CI drivers in [`ci`] and the forge
//! drivers in [`forge`]) and the run lifecycle with confirm tokens, approvals and verification
//! ([`run`]), and approval grants between portals ([`grant`]). Both a repo portal's `seikika run`
//! and the server drive runs through this crate; authorization and storage are theirs.

pub mod check;
pub mod ci;
pub mod executor;
pub mod federation;
pub mod forge;
pub mod grant;
mod http;
pub mod prepare;
pub mod run;
