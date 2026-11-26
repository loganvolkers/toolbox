//! Structured JSON logging is available via SEIKIKA_LOG_FORMAT / options.

#[test]
fn init_accepts_json_log_format() {
    let g = seikika_telemetry::init(seikika_telemetry::TelemetryOptions {
        mode: Some(seikika_telemetry::TelemetryMode::Ci),
        service_name: Some("seikika-log-test".into()),
        prometheus: Some(true),
        log_format: Some("json".into()),
    })
    .expect("init with json logs");
    assert_eq!(g.service_name(), "seikika-log-test");
    assert_eq!(g.mode(), seikika_telemetry::TelemetryMode::Ci);
    tracing::info!(target: "seikika_telemetry_test", "json log path ok");
}

#[test]
fn scrub_hides_secrets() {
    assert_eq!(
        seikika_telemetry::scrub_value("api_token", "secret-value"),
        "[redacted]"
    );
}
