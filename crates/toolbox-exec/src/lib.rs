//! Action runs (docs/design/action-runs.md, docs/design/federated-actions.md): the Toolbox layer of
//! a preview ([`check`]), executors ([`executor`], [`github`]) and the run lifecycle with confirm
//! tokens, approvals and verification ([`run`]), and approval grants between portals ([`grant`]). Both a repo portal's `toolbox run` and the
//! server drive runs through this crate; authorization and storage are theirs.

pub mod check;
pub mod executor;
pub mod federation;
pub mod github;
pub mod grant;
pub mod prepare;
pub mod run;
