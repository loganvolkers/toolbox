//! OpenTelemetry / Prometheus bootstrap for Seikika (docs/v1-spec.md §10.5, plan §6.3).
//!
//! This crate is the **sole** place that installs exporters, builds the resource
//! (`service.name`), owns the SemVer metric catalog, and bridges `tracing` to OTLP
//! logs. Other crates emit via [`tracing`] and [`catalog`] helpers only.
//!
//! Semantic conventions pin: OpenTelemetry SemConv as shipped with the `opentelemetry`
//! 0.33.x stack; MCP gen-ai attrs are Development — see `METRIC_CATALOG.md`.
//!
//! Maintainer SDLC: this crate is part of Seikika itself (not operator portal config).

#![forbid(unsafe_code)]

pub mod analytics;
pub mod catalog;
mod init;
mod metrics_prom;
mod otlp_meters;
mod resource;
mod scrub;
mod trace;

pub use analytics::{AnalyticsSink, NullAnalytics};
pub use init::{TelemetryError, TelemetryGuard, TelemetryMode, TelemetryOptions, init};
pub use metrics_prom::metrics_text;
pub use resource::service_name;
pub use scrub::scrub_value;

/// Hex `trace_id` for the current tracing span when an OTel context is attached.
///
/// Used so JSON stderr lines (and tests) can assert correlation without depending on
/// collector-side log export alone.
pub fn current_trace_id() -> Option<String> {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let cx = tracing::Span::current().context();
    let span = cx.span();
    let sc = span.span_context();
    if sc.is_valid() {
        Some(format!("{:032x}", sc.trace_id()))
    } else {
        None
    }
}
