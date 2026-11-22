//! OpenTelemetry / Prometheus bootstrap for Toolbox (docs/v1-spec.md §10.5, plan §6.3).
//!
//! This crate is the **sole** place that installs exporters, builds the resource
//! (`service.name`), owns the SemVer metric catalog, and bridges `tracing` to OTLP
//! logs. Other crates emit via [`tracing`] and [`catalog`] helpers only.
//!
//! Semantic conventions pin: OpenTelemetry SemConv as shipped with the `opentelemetry`
//! 0.33.x stack; MCP gen-ai attrs are Development — see `METRIC_CATALOG.md`.
//!
//! Maintainer SDLC: this crate is part of Toolbox itself (not operator portal config).

#![forbid(unsafe_code)]

pub mod analytics;
pub mod catalog;
mod init;
mod metrics_prom;
mod resource;
mod scrub;
mod trace;

pub use analytics::{AnalyticsSink, NullAnalytics};
pub use init::{TelemetryError, TelemetryGuard, TelemetryMode, TelemetryOptions, init};
pub use metrics_prom::metrics_text;
pub use resource::service_name;
pub use scrub::scrub_value;
