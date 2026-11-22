//! Optional OTLP meter mirrors of the Prometheus catalog (same SemVer names).

use crate::catalog::names;
use opentelemetry::metrics::{Counter, Gauge, Meter};
use opentelemetry::{KeyValue, global};
use std::sync::{Arc, OnceLock};

struct Meters {
    ingest_records: Counter<u64>,
    ingest_batches: Counter<u64>,
    snapshot_lag: Gauge<f64>,
}

static METERS: OnceLock<Option<Arc<Meters>>> = OnceLock::new();

fn meter() -> Meter {
    global::meter("toolbox")
}

/// Install catalog-aligned OTLP instruments after the global MeterProvider is set.
pub(crate) fn install() {
    let m = meter();
    let ingest_records = m
        .u64_counter(names::INGEST_RECORDS_TOTAL)
        .with_description("Connector ingest records accepted")
        .build();
    let ingest_batches = m
        .u64_counter(names::INGEST_BATCHES_TOTAL)
        .with_description("Connector ingest batches accepted")
        .build();
    let snapshot_lag = m
        .f64_gauge(names::SNAPSHOT_LAG_SECONDS)
        .with_description("Age of the published snapshot in seconds")
        .build();
    let _ = METERS.set(Some(Arc::new(Meters {
        ingest_records,
        ingest_batches,
        snapshot_lag,
    })));
}

pub(crate) fn record_ingest(records: u64) {
    if let Some(Some(m)) = METERS.get() {
        m.ingest_records.add(records, &[]);
        m.ingest_batches.add(1, &[]);
    }
}

pub(crate) fn set_snapshot_lag(seconds: f64) {
    if let Some(Some(m)) = METERS.get() {
        m.snapshot_lag.record(seconds, &[]);
    }
}

/// Touch zero-valued instruments so exporters see the names even before traffic.
pub(crate) fn touch_required() {
    if let Some(Some(m)) = METERS.get() {
        m.ingest_records.add(0, &[]);
        m.ingest_batches.add(0, &[]);
        m.snapshot_lag
            .record(0.0, &[KeyValue::new("toolbox.touch", "true")]);
    }
}
