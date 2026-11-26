//! `tracing` subscriber: JSON/pretty fmt + optional OTel bridge.

use crate::init::{TelemetryMode, TelemetryOptions};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

pub(crate) struct TraceStack {
    pub tracer_provider: Option<SdkTracerProvider>,
    pub logger_provider: Option<SdkLoggerProvider>,
}

pub(crate) fn install_subscriber(
    mode: TelemetryMode,
    opts: &TelemetryOptions,
    tracer_provider: Option<SdkTracerProvider>,
    logger_provider: Option<SdkLoggerProvider>,
) -> TraceStack {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(match mode {
            TelemetryMode::Local => "info",
            TelemetryMode::Ci | TelemetryMode::Prod => "info",
        })
    });

    let json = match opts.log_format.as_deref() {
        Some("pretty") => false,
        Some("json") => true,
        _ => {
            matches!(mode, TelemetryMode::Ci | TelemetryMode::Prod)
                || std::env::var("SEIKIKA_LOG_FORMAT")
                    .map(|v| v.eq_ignore_ascii_case("json"))
                    .unwrap_or(false)
        }
    };

    // Always stderr so structured CLI stdout (describe, report -o json, MCP) stays pure JSON.
    let fmt_layer = if json {
        tracing_subscriber::fmt::layer()
            .json()
            .with_writer(std::io::stderr)
            .with_current_span(true)
            .with_span_list(false)
            .with_target(true)
            .boxed()
    } else {
        tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(true)
            .boxed()
    };

    let mut layers = vec![fmt_layer];

    if let Some(ref provider) = tracer_provider {
        let tracer = provider.tracer("seikika");
        layers.push(tracing_opentelemetry::layer().with_tracer(tracer).boxed());
    }

    if let Some(ref provider) = logger_provider {
        layers.push(
            opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(provider)
                .boxed(),
        );
    }

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .try_init();

    TraceStack {
        tracer_provider,
        logger_provider,
    }
}
