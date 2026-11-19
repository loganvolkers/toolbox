use axum::{routing::get, Router, Json};
pub fn router() -> Router { Router::new().route("/v1/entities/{id}", get(|axum::extract::Path(id): axum::extract::Path<u64>| async move { Json(serde_json_value(id)) })) }
fn serde_json_value(id: u64) -> String { format!("{:?}", model::entity(id)) }
