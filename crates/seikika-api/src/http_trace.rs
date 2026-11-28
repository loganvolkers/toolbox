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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::{Body, to_bytes},
        middleware,
        routing::get,
    };
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use std::sync::{Arc, Mutex};
    use tower::ServiceExt as _;
    use tracing::instrument::WithSubscriber as _;
    use tracing_subscriber::prelude::*;

    struct LogWriter(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for LogWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn request_completion_log_matches_the_active_trace() {
        let logs = Arc::new(Mutex::new(Vec::new()));
        let capture = logs.clone();
        let provider = SdkTracerProvider::builder().build();
        let subscriber = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("http-test")))
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(move || LogWriter(capture.clone())),
            );
        let app = Router::new()
            .route(
                "/",
                get(|| async {
                    seikika_telemetry::current_trace_id().expect("request has an OTel trace")
                }),
            )
            .layer(middleware::from_fn(layer));

        let response = app
            .oneshot(Request::new(Body::empty()))
            .with_subscriber(subscriber)
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 200);
        let body = to_bytes(response.into_body(), 32).await.unwrap();
        let trace_id = std::str::from_utf8(&body).unwrap();
        assert_eq!(trace_id.len(), 32);

        let output = logs.lock().unwrap();
        let completion = std::str::from_utf8(&output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("valid JSON log"))
            .find(|event| event["fields"]["message"] == "http request completed")
            .expect("request completion event");
        assert_eq!(completion["fields"]["trace_id"], trace_id);
        assert_eq!(completion["fields"]["status"], "200");
    }
}
