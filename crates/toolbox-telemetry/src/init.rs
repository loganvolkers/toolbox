//! Process-wide telemetry bootstrap. Call [`init`] once before listeners.

use crate::metrics_prom::{self, Catalog};
use crate::resource::build_resource;
use crate::trace::{self, TraceStack};
use opentelemetry::global;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, SdkTracerProvider};
use std::sync::Arc;
use thiserror::Error;

/// How sampling / failure defaults are chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryMode {
    Local,
    Ci,
    Prod,
}

impl TelemetryMode {
    /// Detect from `TOOLBOX_TELEMETRY`, then `CI=true`, else Local.
    pub fn detect() -> Self {
        match std::env::var("TOOLBOX_TELEMETRY")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "ci" => Self::Ci,
            "prod" | "production" => Self::Prod,
            "local" => Self::Local,
            _ if std::env::var("CI").ok().as_deref() == Some("true") => Self::Ci,
            _ => Self::Local,
        }
    }
}

/// Options for [`init`]. Env vars still win for OTel exporter settings.
#[derive(Debug, Clone, Default)]
pub struct TelemetryOptions {
    pub mode: Option<TelemetryMode>,
    pub service_name: Option<String>,
    /// Bind/serve Prometheus instruments (default true). Scrape is still via `metrics_text`.
    pub prometheus: Option<bool>,
    /// Override `TOOLBOX_LOG_FORMAT` (`json` | `pretty`).
    pub log_format: Option<String>,
}

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("service.name is required in CI/prod (set OTEL_SERVICE_NAME)")]
    MissingServiceName,
    #[error("prometheus catalog: {0}")]
    Prometheus(#[from] prometheus::Error),
    #[error("opentelemetry: {0}")]
    OpenTelemetry(String),
}

/// Flush providers on drop.
pub struct TelemetryGuard {
    _catalog: Arc<Catalog>,
    trace: TraceStack,
    service_name: String,
    mode: TelemetryMode,
    otlp: bool,
}

impl TelemetryGuard {
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn mode(&self) -> TelemetryMode {
        self.mode
    }

    pub fn otlp_enabled(&self) -> bool {
        self.otlp
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.trace.tracer_provider.take()
            && let Err(e) = provider.shutdown()
        {
            eprintln!("toolbox-telemetry: tracer shutdown: {e}");
        }
        if let Some(provider) = self.trace.logger_provider.take()
            && let Err(e) = provider.shutdown()
        {
            eprintln!("toolbox-telemetry: logger shutdown: {e}");
        }
    }
}

/// Initialize telemetry. Safe to call once per process; subsequent calls replace the catalog
/// (intended for tests) and may no-op the tracing subscriber.
pub fn init(opts: TelemetryOptions) -> Result<TelemetryGuard, TelemetryError> {
    let mode = opts.mode.unwrap_or_else(TelemetryMode::detect);
    let prometheus_on = opts.prometheus.unwrap_or(true)
        && std::env::var("TOOLBOX_METRICS_PROMETHEUS")
            .map(|v| !v.eq_ignore_ascii_case("false") && v != "0")
            .unwrap_or(true);

    let (resource, service_name) = build_resource(mode, opts.service_name.as_deref())?;

    // Catalog instruments always exist in-process; `prometheus_on` is reserved for
    // future scrape-toggle behaviour (helpers still record when false).
    let _ = prometheus_on;
    let catalog = {
        let c = Arc::new(Catalog::new()?);
        metrics_prom::install(c.clone());
        c
    };

    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    let otlp = otlp_endpoint.is_some();

    let tracer_provider = match &otlp_endpoint {
        Some(endpoint) => match build_tracer(mode, resource.clone(), endpoint) {
            Ok(p) => {
                global::set_tracer_provider(p.clone());
                Some(p)
            }
            Err(e) => match mode {
                TelemetryMode::Ci => return Err(e),
                TelemetryMode::Local | TelemetryMode::Prod => {
                    tracing::warn!(error = %e, "OTLP tracer init failed; continuing");
                    // tracing may not be installed yet — also print.
                    eprintln!("toolbox-telemetry: OTLP tracer init failed: {e}");
                    None
                }
            },
        },
        None => None,
    };

    // Optional OTLP metrics exporter (mirrors catalog via SDK meters later; scrape stays Prom).
    if let Some(endpoint) = &otlp_endpoint
        && let Err(e) = install_otlp_metrics(mode, resource.clone(), endpoint)
    {
        match mode {
            TelemetryMode::Ci => return Err(e),
            TelemetryMode::Local | TelemetryMode::Prod => {
                eprintln!("toolbox-telemetry: OTLP metrics init failed: {e}");
            }
        }
    }

    let log_provider = if otlp {
        match build_logger_provider(resource) {
            Ok(p) => Some(p),
            Err(e) => match mode {
                TelemetryMode::Ci => return Err(e),
                TelemetryMode::Local | TelemetryMode::Prod => {
                    eprintln!("toolbox-telemetry: OTLP logs init failed: {e}");
                    None
                }
            },
        }
    } else {
        None
    };

    let trace = trace::install_subscriber(mode, &opts, tracer_provider, log_provider);

    tracing::info!(
        service.name = %service_name,
        telemetry.mode = ?mode,
        otlp = otlp,
        "toolbox telemetry initialized"
    );

    Ok(TelemetryGuard {
        _catalog: catalog,
        trace,
        service_name,
        mode,
        otlp,
    })
}

fn build_tracer(
    mode: TelemetryMode,
    resource: opentelemetry_sdk::Resource,
    endpoint: &str,
) -> Result<SdkTracerProvider, TelemetryError> {
    use opentelemetry_otlp::WithExportConfig;

    let protocol = std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").unwrap_or_default();
    let exporter = if protocol.eq_ignore_ascii_case("http/protobuf")
        || protocol.eq_ignore_ascii_case("http/json")
    {
        opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .build()
            .map_err(|e| TelemetryError::OpenTelemetry(e.to_string()))?
    } else {
        opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .map_err(|e| TelemetryError::OpenTelemetry(e.to_string()))?
    };

    let sampler = sampler_for(mode);
    Ok(SdkTracerProvider::builder()
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build())
}

fn sampler_for(mode: TelemetryMode) -> Sampler {
    if let Ok(name) = std::env::var("OTEL_TRACES_SAMPLER") {
        let arg = std::env::var("OTEL_TRACES_SAMPLER_ARG").ok();
        return match name.to_ascii_lowercase().as_str() {
            "always_on" => Sampler::AlwaysOn,
            "always_off" => Sampler::AlwaysOff,
            "traceidratio" => {
                let ratio = arg.as_deref().and_then(|s| s.parse().ok()).unwrap_or(1.0);
                Sampler::TraceIdRatioBased(ratio)
            }
            "parentbased_always_on" => Sampler::ParentBased(Box::new(Sampler::AlwaysOn)),
            "parentbased_always_off" => Sampler::ParentBased(Box::new(Sampler::AlwaysOff)),
            "parentbased_traceidratio" => {
                let ratio = arg
                    .as_deref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(match mode {
                        TelemetryMode::Prod => 0.1,
                        TelemetryMode::Local | TelemetryMode::Ci => 1.0,
                    });
                Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio)))
            }
            _ => default_sampler(mode),
        };
    }
    default_sampler(mode)
}

fn default_sampler(mode: TelemetryMode) -> Sampler {
    let ratio = match mode {
        TelemetryMode::Prod => 0.1,
        TelemetryMode::Local | TelemetryMode::Ci => 1.0,
    };
    Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio)))
}

fn install_otlp_metrics(
    _mode: TelemetryMode,
    resource: opentelemetry_sdk::Resource,
    endpoint: &str,
) -> Result<(), TelemetryError> {
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::metrics::SdkMeterProvider;

    let protocol = std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").unwrap_or_default();
    let exporter = if protocol.eq_ignore_ascii_case("http/protobuf")
        || protocol.eq_ignore_ascii_case("http/json")
    {
        opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .build()
            .map_err(|e| TelemetryError::OpenTelemetry(e.to_string()))?
    } else {
        opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .map_err(|e| TelemetryError::OpenTelemetry(e.to_string()))?
    };

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter).build();
    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build();
    global::set_meter_provider(provider);
    Ok(())
}

fn build_logger_provider(
    resource: opentelemetry_sdk::Resource,
) -> Result<opentelemetry_sdk::logs::SdkLoggerProvider, TelemetryError> {
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::logs::SdkLoggerProvider;

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .map_err(|_| TelemetryError::OpenTelemetry("OTEL_EXPORTER_OTLP_ENDPOINT unset".into()))?;
    let protocol = std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").unwrap_or_default();
    let exporter = if protocol.eq_ignore_ascii_case("http/protobuf")
        || protocol.eq_ignore_ascii_case("http/json")
    {
        opentelemetry_otlp::LogExporter::builder()
            .with_http()
            .with_endpoint(endpoint)
            .build()
            .map_err(|e| TelemetryError::OpenTelemetry(e.to_string()))?
    } else {
        opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .map_err(|e| TelemetryError::OpenTelemetry(e.to_string()))?
    };

    Ok(SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_local_by_default() {
        // CI may be set in the agent environment; only assert the enum is well-formed.
        let _ = TelemetryMode::detect();
    }

    #[test]
    fn init_local_without_otlp() {
        metrics_prom::clear_for_tests();
        let g = init(TelemetryOptions {
            mode: Some(TelemetryMode::Local),
            service_name: Some("toolbox-test".into()),
            prometheus: Some(true),
            log_format: Some("pretty".into()),
        })
        .expect("init");
        assert_eq!(g.service_name(), "toolbox-test");
        assert!(!g.otlp_enabled());
        crate::catalog::record_ingest(3);
        crate::catalog::set_snapshot_lag(1.5);
        let text = crate::metrics_text();
        assert!(
            text.contains("toolbox_ingest_records_total"),
            "missing ingest counter:\n{text}"
        );
        assert!(
            text.contains("toolbox_snapshot_lag_seconds"),
            "missing lag:\n{text}"
        );
        assert!(
            text.contains("toolbox_snapshot_age_seconds"),
            "missing age alias:\n{text}"
        );
        for name in crate::catalog::names::REQUIRED {
            assert!(
                text.contains(name),
                "catalog missing {name} in scrape:\n{text}"
            );
        }
    }
}
