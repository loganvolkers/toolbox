//! HTTP request spans for OTLP (design §4.1 / testbed rubric).

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use tracing::Instrument;

/// Axum middleware: one `http.server.request` span per request, with a correlated log line.
pub async fn layer(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let span = tracing::info_span!(
        "http.server.request",
        "http.request.method" = %method,
        "url.path" = %path,
        "http.response.status_code" = tracing::field::Empty,
        "otel.status_code" = tracing::field::Empty,
        "otel.kind" = "server",
    );
    async move {
        let response = next.run(req).await;
        let status = response.status();
        let span = tracing::Span::current();
        span.record("http.response.status_code", i64::from(status.as_u16()));
        if status.is_server_error() {
            span.record("otel.status_code", "ERROR");
        }
        let trace_id = seikika_telemetry::current_trace_id();
        tracing::info!(
            status = %status.as_u16(),
            trace_id = trace_id.as_deref(),
            "http.request.method" = %method,
            "url.path" = %path,
            "http request completed"
        );
        response
    }
    .instrument(span)
    .await
}
