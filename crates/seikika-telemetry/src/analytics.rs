//! Product analytics sinks (plan §6.3). Operational OTel is separate (D6).

/// Where product-analytics events go. Operational traces/metrics/logs never use this.
pub trait AnalyticsSink: Send + Sync {
    /// Record one versioned analytics event. Implementations must scrub PII first.
    fn capture(&self, event: &str, properties: &[(&str, &str)]);
}

/// Default sink: discard everything (`SEIKIKA_ANALYTICS=off` equivalent).
#[derive(Debug, Default, Clone, Copy)]
pub struct NullAnalytics;

impl AnalyticsSink for NullAnalytics {
    fn capture(&self, _event: &str, _properties: &[(&str, &str)]) {}
}
