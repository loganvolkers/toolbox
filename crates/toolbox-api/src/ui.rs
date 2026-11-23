//! The minimal UI (docs/design/minimal-ui.md), built from `ui/` with Vite and embedded in the
//! binary, so the image needs no Node (G1). Any non-API path serves the single-page app.

use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};

const INDEX: &str = include_str!("../../../ui/dist/index.html");
const APP_JS: &str = include_str!("../../../ui/dist/assets/app.js");
const APP_CSS: &str = include_str!("../../../ui/dist/assets/app.css");

pub(crate) const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'";

/// The embedded UI: its assets by path, and the app shell for any other page.
pub async fn asset(uri: Uri) -> Response {
    let path = uri.path();
    let (ctype, body) = match path {
        "/assets/app.js" => ("text/javascript; charset=utf-8", APP_JS),
        "/assets/app.css" => ("text/css; charset=utf-8", APP_CSS),
        p if p.starts_with("/v1/") || p.starts_with("/mcp") || p.starts_with("/assets/") => {
            return crate::routes::problem(
                StatusCode::NOT_FOUND,
                "not-found",
                format!("no route {p}"),
            );
        }
        _ => ("text/html; charset=utf-8", INDEX),
    };
    (
        [
            (header::CONTENT_TYPE, ctype),
            (header::CONTENT_SECURITY_POLICY, CSP),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
        .into_response()
}
