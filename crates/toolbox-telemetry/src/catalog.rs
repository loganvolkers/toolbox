//! SemVer metric catalog (v1-spec §10.5). Names, units and required labels are stable.
//!
//! Prometheus exposition uses the exact names in [`names`]. Counters already include `_total`.

use crate::metrics_prom::{Catalog, global};
use prometheus::{HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec};
use std::sync::Arc;

/// Catalog metric names (Prometheus exposition / OTel instrument alignment).
pub mod names {
    pub const INGEST_RECORDS_TOTAL: &str = "toolbox_ingest_records_total";
    pub const INGEST_BATCHES_TOTAL: &str = "toolbox_ingest_batches_total";
    pub const RECOMPUTES_TOTAL: &str = "toolbox_recomputes_total";
    /// Last recompute duration (gauge). Histogram migration is a later SemVer minor.
    pub const RECOMPUTE_SECONDS: &str = "toolbox_recompute_seconds";
    /// Canonical snapshot lag (v1-spec §10.5).
    pub const SNAPSHOT_LAG_SECONDS: &str = "toolbox_snapshot_lag_seconds";
    /// Deprecated alias for [`SNAPSHOT_LAG_SECONDS`]; dual-exported while `TELEMETRY_PROM_ALIASES` is on.
    pub const SNAPSHOT_AGE_SECONDS: &str = "toolbox_snapshot_age_seconds";
    pub const COMMAND_DURATION_SECONDS: &str = "toolbox_command_duration_seconds";
    pub const RUN_TRANSITIONS_TOTAL: &str = "toolbox_run_transitions_total";
    pub const AUTHZ_CHECKS_TOTAL: &str = "toolbox_authz_checks_total";
    pub const MCP_TOOL_CALLS_TOTAL: &str = "toolbox_mcp_tool_calls_total";
    pub const CONNECTOR_LAG_SECONDS: &str = "toolbox_connector_lag_seconds";
    pub const SCORING_BACKLOG: &str = "toolbox_scoring_backlog";
    pub const ACTION_RUNS_TOTAL: &str = "toolbox_action_runs_total";
    pub const INDEX_FRESHNESS_SECONDS: &str = "toolbox_index_freshness_seconds";

    /// Every v1 required name that must appear in a scrape (stubs included).
    pub const REQUIRED: &[&str] = &[
        INGEST_RECORDS_TOTAL,
        INGEST_BATCHES_TOTAL,
        RECOMPUTES_TOTAL,
        RECOMPUTE_SECONDS,
        SNAPSHOT_LAG_SECONDS,
        COMMAND_DURATION_SECONDS,
        RUN_TRANSITIONS_TOTAL,
        AUTHZ_CHECKS_TOTAL,
        MCP_TOOL_CALLS_TOTAL,
        CONNECTOR_LAG_SECONDS,
        SCORING_BACKLOG,
        ACTION_RUNS_TOTAL,
        INDEX_FRESHNESS_SECONDS,
    ];
}

fn cat() -> Arc<Catalog> {
    global().expect("toolbox-telemetry::init must run before catalog metrics")
}

/// Whether the deprecated `toolbox_snapshot_age_seconds` alias is emitted (default on).
pub fn prom_aliases_enabled() -> bool {
    match std::env::var("TELEMETRY_PROM_ALIASES") {
        Ok(v) => v != "0" && !v.eq_ignore_ascii_case("false"),
        Err(_) => true,
    }
}

pub fn ingest_records() -> IntCounter {
    cat().ingest_records.clone()
}

pub fn ingest_batches() -> IntCounter {
    cat().ingest_batches.clone()
}

pub fn recomputes() -> IntCounter {
    cat().recomputes.clone()
}

pub fn recompute_seconds() -> prometheus::Gauge {
    cat().recompute_seconds.clone()
}

pub fn snapshot_lag_seconds() -> prometheus::Gauge {
    cat().snapshot_lag_seconds.clone()
}

pub fn snapshot_age_seconds() -> Option<prometheus::Gauge> {
    cat().snapshot_age_seconds.clone()
}

pub fn command_duration_seconds() -> HistogramVec {
    cat().command_duration_seconds.clone()
}

pub fn run_transitions_total() -> IntCounterVec {
    cat().run_transitions_total.clone()
}

pub fn authz_checks_total() -> IntCounterVec {
    cat().authz_checks_total.clone()
}

pub fn mcp_tool_calls_total() -> IntCounterVec {
    cat().mcp_tool_calls_total.clone()
}

pub fn connector_lag_seconds() -> IntGaugeVec {
    cat().connector_lag_seconds.clone()
}

pub fn scoring_backlog() -> IntGauge {
    cat().scoring_backlog.clone()
}

pub fn action_runs_total() -> IntCounterVec {
    cat().action_runs_total.clone()
}

pub fn index_freshness_seconds() -> IntGaugeVec {
    cat().index_freshness_seconds.clone()
}

/// Record ingest acceptance (records + one batch).
pub fn record_ingest(records: u64) {
    if let Some(c) = global() {
        c.ingest_records.inc_by(records);
        c.ingest_batches.inc();
    }
}

/// Record a finished recompute: count + last duration seconds.
pub fn record_recompute(duration_secs: f64) {
    if let Some(c) = global() {
        c.recomputes.inc();
        c.recompute_seconds.set(duration_secs);
    }
}

/// Set snapshot lag (and the deprecated age alias when enabled).
pub fn set_snapshot_lag(seconds: f64) {
    if let Some(c) = global() {
        c.snapshot_lag_seconds.set(seconds);
        if let Some(age) = &c.snapshot_age_seconds {
            age.set(seconds);
        }
    }
}

/// Count an authz decision (`allow` | `deny`).
pub fn record_authz(decision: &str) {
    if let Some(c) = global() {
        c.authz_checks_total.with_label_values(&[decision]).inc();
    }
}

/// Count an MCP tool invocation.
pub fn record_mcp_tool(tool: &str, outcome: &str) {
    if let Some(c) = global() {
        c.mcp_tool_calls_total
            .with_label_values(&[tool, outcome])
            .inc();
    }
}

/// Observe a CLI/admin command duration.
pub fn record_command(status: &str, seconds: f64) {
    if let Some(c) = global() {
        c.command_duration_seconds
            .with_label_values(&[status])
            .observe(seconds);
    }
}
