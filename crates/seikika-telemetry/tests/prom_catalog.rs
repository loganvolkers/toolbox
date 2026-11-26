//! Assert the Prometheus scrape text includes every v1 catalog name.

#[test]
fn metrics_text_lists_required_catalog_names() {
    let _g = seikika_telemetry::init(seikika_telemetry::TelemetryOptions {
        mode: Some(seikika_telemetry::TelemetryMode::Local),
        service_name: Some("seikika-prom-test".into()),
        prometheus: Some(true),
        log_format: Some("pretty".into()),
    })
    .expect("init");

    seikika_telemetry::catalog::record_ingest(2);
    seikika_telemetry::catalog::record_recompute(0.05);
    seikika_telemetry::catalog::set_snapshot_lag(3.0);
    seikika_telemetry::catalog::record_authz("allow");
    seikika_telemetry::catalog::record_mcp_tool("get", "ok");
    seikika_telemetry::catalog::record_command("ok", 0.01);

    let text = seikika_telemetry::metrics_text();
    for name in seikika_telemetry::catalog::names::REQUIRED {
        assert!(
            text.contains(name),
            "scrape missing required metric {name}:\n{text}"
        );
    }
    assert!(
        text.contains("seikika_snapshot_age_seconds"),
        "deprecated age alias missing while TELEMETRY_PROM_ALIASES default on:\n{text}"
    );
    assert!(
        text.contains("# TYPE seikika_ingest_records_total counter"),
        "counter type line missing:\n{text}"
    );
    assert!(
        text.contains("# TYPE seikika_snapshot_lag_seconds gauge"),
        "lag gauge type missing:\n{text}"
    );
}
