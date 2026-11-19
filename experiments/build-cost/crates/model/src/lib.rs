use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity { pub id: u64, pub kind: String, pub owner: u64, pub spec: serde_json::Value }
pub fn entity(id: u64) -> Entity { Entity { id, kind: "Service".into(), owner: id % 1000, spec: serde_json::json!({"tier": id % 4}) } }
