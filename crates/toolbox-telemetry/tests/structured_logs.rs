//! Structured JSON logging is available via TOOLBOX_LOG_FORMAT / options.

#[test]
fn init_accepts_json_log_format() {
    let g = toolbox_telemetry::init(toolbox_telemetry::TelemetryOptions {
        mode: Some(toolbox_telemetry::TelemetryMode::Ci),
        service_name: Some("toolbox-log-test".into()),
        prometheus: Some(true),
        log_format: Some("json".into()),
    })
    .expect("init with json logs");
    assert_eq!(g.service_name(), "toolbox-log-test");
    assert_eq!(g.mode(), toolbox_telemetry::TelemetryMode::Ci);
    tracing::info!(target: "toolbox_telemetry_test", "json log path ok");
}

#[test]
fn scrub_hides_secrets() {
    assert_eq!(
        toolbox_telemetry::scrub_value("api_token", "secret-value"),
        "[redacted]"
    );
}
