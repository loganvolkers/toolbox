//! Prometheus registry owned by this crate; `/metrics` scrapes [`metrics_text`].

use crate::catalog::{names, prom_aliases_enabled};
use prometheus::{
    Encoder, Gauge, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
    Opts, Registry, TextEncoder,
};
use std::sync::{Arc, OnceLock, RwLock};

static STATE: OnceLock<RwLock<Option<Arc<Catalog>>>> = OnceLock::new();

fn slot() -> &'static RwLock<Option<Arc<Catalog>>> {
    STATE.get_or_init(|| RwLock::new(None))
}

/// Install the catalog (called from `init`). Replaces any previous catalog (tests).
pub(crate) fn install(catalog: Arc<Catalog>) {
    *slot().write().expect("catalog lock") = Some(catalog);
}

pub(crate) fn global() -> Option<Arc<Catalog>> {
    slot().read().expect("catalog lock").clone()
}

#[cfg(test)]
pub(crate) fn clear_for_tests() {
    *slot().write().expect("catalog lock") = None;
}

/// Instruments + the registry they belong to.
pub struct Catalog {
    pub registry: Registry,
    pub ingest_records: IntCounter,
    pub ingest_batches: IntCounter,
    pub recomputes: IntCounter,
    pub recompute_seconds: Gauge,
    pub snapshot_lag_seconds: Gauge,
    pub snapshot_age_seconds: Option<Gauge>,
    pub command_duration_seconds: HistogramVec,
    pub run_transitions_total: IntCounterVec,
    pub authz_checks_total: IntCounterVec,
    pub mcp_tool_calls_total: IntCounterVec,
    pub connector_lag_seconds: IntGaugeVec,
    pub scoring_backlog: IntGauge,
    pub action_runs_total: IntCounterVec,
    pub index_freshness_seconds: IntGaugeVec,
}

impl Catalog {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        let ingest_records = IntCounter::with_opts(Opts::new(
            names::INGEST_RECORDS_TOTAL,
            "Connector ingest records accepted",
        ))?;
        let ingest_batches = IntCounter::with_opts(Opts::new(
            names::INGEST_BATCHES_TOTAL,
            "Connector ingest batches accepted",
        ))?;
        let recomputes = IntCounter::with_opts(Opts::new(
            names::RECOMPUTES_TOTAL,
            "Derived-state recomputes completed",
        ))?;
        let recompute_seconds = Gauge::with_opts(Opts::new(
            names::RECOMPUTE_SECONDS,
            "Duration of the last recompute in seconds",
        ))?;
        let snapshot_lag_seconds = Gauge::with_opts(Opts::new(
            names::SNAPSHOT_LAG_SECONDS,
            "Age of the published snapshot in seconds",
        ))?;
        let snapshot_age_seconds = if prom_aliases_enabled() {
            Some(Gauge::with_opts(Opts::new(
                names::SNAPSHOT_AGE_SECONDS,
                "Deprecated alias for toolbox_snapshot_lag_seconds",
            ))?)
        } else {
            None
        };
        let command_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                names::COMMAND_DURATION_SECONDS,
                "CLI / admin command duration in seconds",
            ),
            &["status"],
        )?;
        let run_transitions_total = IntCounterVec::new(
            Opts::new(
                names::RUN_TRANSITIONS_TOTAL,
                "Action-run state transitions (stub until M5)",
            ),
            &["state"],
        )?;
        let authz_checks_total = IntCounterVec::new(
            Opts::new(names::AUTHZ_CHECKS_TOTAL, "Authorization check outcomes"),
            &["decision"],
        )?;
        let mcp_tool_calls_total = IntCounterVec::new(
            Opts::new(names::MCP_TOOL_CALLS_TOTAL, "MCP tool invocations"),
            &["tool", "outcome"],
        )?;
        let connector_lag_seconds = IntGaugeVec::new(
            Opts::new(
                names::CONNECTOR_LAG_SECONDS,
                "Connector freshness lag in seconds",
            ),
            &["connector", "kind"],
        )?;
        let scoring_backlog = IntGauge::with_opts(Opts::new(
            names::SCORING_BACKLOG,
            "Entities waiting for score recomputation",
        ))?;
        let action_runs_total = IntCounterVec::new(
            Opts::new(
                names::ACTION_RUNS_TOTAL,
                "Action runs by action and state (stub until M5)",
            ),
            &["action", "state"],
        )?;
        let index_freshness_seconds = IntGaugeVec::new(
            Opts::new(
                names::INDEX_FRESHNESS_SECONDS,
                "Search/index freshness in seconds",
            ),
            &["index"],
        )?;

        registry.register(Box::new(ingest_records.clone()))?;
        registry.register(Box::new(ingest_batches.clone()))?;
        registry.register(Box::new(recomputes.clone()))?;
        registry.register(Box::new(recompute_seconds.clone()))?;
        registry.register(Box::new(snapshot_lag_seconds.clone()))?;
        if let Some(age) = &snapshot_age_seconds {
            registry.register(Box::new(age.clone()))?;
        }
        registry.register(Box::new(command_duration_seconds.clone()))?;
        registry.register(Box::new(run_transitions_total.clone()))?;
        registry.register(Box::new(authz_checks_total.clone()))?;
        registry.register(Box::new(mcp_tool_calls_total.clone()))?;
        registry.register(Box::new(connector_lag_seconds.clone()))?;
        registry.register(Box::new(scoring_backlog.clone()))?;
        registry.register(Box::new(action_runs_total.clone()))?;
        registry.register(Box::new(index_freshness_seconds.clone()))?;

        // Touch labelled stubs so scrape lists the metric families even at zero.
        let _ = run_transitions_total.with_label_values(&["pending"]);
        let _ = authz_checks_total.with_label_values(&["allow"]);
        let _ = mcp_tool_calls_total.with_label_values(&["get", "ok"]);
        let _ = command_duration_seconds.with_label_values(&["ok"]);
        let _ = connector_lag_seconds.with_label_values(&["_", "_"]);
        let _ = action_runs_total.with_label_values(&["_", "pending"]);
        let _ = index_freshness_seconds.with_label_values(&["default"]);

        Ok(Self {
            registry,
            ingest_records,
            ingest_batches,
            recomputes,
            recompute_seconds,
            snapshot_lag_seconds,
            snapshot_age_seconds,
            command_duration_seconds,
            run_transitions_total,
            authz_checks_total,
            mcp_tool_calls_total,
            connector_lag_seconds,
            scoring_backlog,
            action_runs_total,
            index_freshness_seconds,
        })
    }
}

/// Prometheus text exposition for `GET /metrics`.
pub fn metrics_text() -> String {
    let Some(cat) = global() else {
        return String::new();
    };
    let encoder = TextEncoder::new();
    let families = cat.registry.gather();
    let mut buf = Vec::new();
    if encoder.encode(&families, &mut buf).is_err() {
        return String::new();
    }
    String::from_utf8(buf).unwrap_or_default()
}
