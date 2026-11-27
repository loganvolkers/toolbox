//! The Astro portal build, embedded so the server can ship in the Rust binary.

use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};

const REPORT: &str = include_str!("../ui-dist/report/index.html");
const INDEX: &str = include_str!("../ui-dist/index.html");
include!("../ui-dist/assets.rs");

const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'";

fn response(content_type: &str, body: impl IntoResponse) -> Response {
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_SECURITY_POLICY, CSP),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
        .into_response()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn entity(title: &str, owner: Option<&str>) -> Response {
    let title = escape_html(title);
    let owner = owner
        .map(|owner| format!("<p>Owner: {}</p>", escape_html(owner)))
        .unwrap_or_default();
    let header = format!("<header id=\"seikika-initial-entity\"><h1>{title}</h1>{owner}</header>");
    let page = INDEX
        .replace(
            "<title>Seikika</title>",
            &format!("<title>{title} · Seikika</title>"),
        )
        .replace("<!-- seikika:entity -->", &header);
    let mut result = response("text/html; charset=utf-8", page);
    result.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("private, no-store"),
    );
    result
}

/// Serve the embedded assets and the portal shell for client-side routes.
pub async fn asset(uri: Uri) -> Response {
    let path = uri.path();
    if matches!(path, "/report" | "/report/") {
        return response("text/html; charset=utf-8", REPORT);
    }
    if let Some((_, ctype, body)) = ASSETS.iter().find(|(asset, _, _)| *asset == path) {
        return response(ctype, *body);
    }
    if path.starts_with("/v1/") || path.starts_with("/mcp") || path.starts_with("/assets/") {
        return crate::routes::problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no route {path}"),
        );
    }
    response("text/html; charset=utf-8", INDEX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn entity_uses_the_astro_document_and_escapes_text() {
        let response = entity("<Checkout>", Some("Payments & Billing"));
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        let body = axum::body::to_bytes(response.into_body(), 16_384)
            .await
            .unwrap();
        let html = std::str::from_utf8(&body).unwrap();
        assert!(html.contains("&lt;Checkout&gt; · Seikika"));
        assert!(html.contains("Owner: Payments &amp; Billing"));
        assert!(ASSETS.iter().any(|(path, content_type, _)| {
            content_type.starts_with("text/javascript") && html.contains(path)
        }));
        assert!(ASSETS.iter().any(|(path, content_type, _)| {
            content_type.starts_with("text/css") && html.contains(path)
        }));
    }
}
