//! Inputs to the derived-state pipeline (docs/v1-spec.md §6.1, §8): raw connector records and
//! native values, a consistent [`InputSnapshot`] of them, an in-memory store for tests and
//! `seikika dev`, and the NDJSON fixture loader. `PgStore` arrives with M2.

pub mod pg;

pub use pg::{PgStore, Publication};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// The latest version of one raw record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub connector: String,
    pub kind: String,
    pub external_id: String,
    pub data: Value,
}

/// One native value revision (a field whose source of truth is Seikika, D13).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NativeValue {
    pub entity: String,
    pub field: String,
    pub value: Value,
    #[serde(default)]
    pub revision: u64,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub at: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

/// A consistent read of every input. Records are ordered by (connector, kind, external id) and
/// native values by (entity, field), so the pipeline's output never depends on arrival order (P1).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputSnapshot {
    pub version: u64,
    pub records: Vec<Record>,
    pub native: Vec<NativeValue>,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("{0}")]
    Io(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("{file}:{line}: {message}")]
    Message {
        file: String,
        line: usize,
        message: String,
    },
}

/// The inputs contract (docs/v1-spec.md §7.1): MemStore for tests and `seikika dev`, PgStore in
/// production.
#[async_trait::async_trait]
pub trait Store: Send + Sync {
    /// Apply one batch of connector messages atomically; returns the new input version.
    async fn ingest(&self, connector: &str, batch: &[Value]) -> Result<u64, StoreError>;
    /// Write native values; a non-zero `revision` must be the next revision (optimistic locking).
    async fn put_native(&self, values: Vec<NativeValue>) -> Result<u64, StoreError>;
    /// The latest committed input version.
    async fn input_version(&self) -> Result<u64, StoreError>;
    /// A consistent read of all inputs.
    async fn snapshot(&self) -> Result<InputSnapshot, StoreError>;
}

/// `MemStore` behind a lock, as a [`Store`].
#[derive(Debug, Default)]
pub struct SharedMem(pub std::sync::Mutex<MemStore>);

#[async_trait::async_trait]
impl Store for SharedMem {
    async fn ingest(&self, connector: &str, batch: &[Value]) -> Result<u64, StoreError> {
        Ok(self.0.lock().unwrap().ingest(connector, batch))
    }
    async fn put_native(&self, values: Vec<NativeValue>) -> Result<u64, StoreError> {
        Ok(self.0.lock().unwrap().put_native(values))
    }
    async fn input_version(&self) -> Result<u64, StoreError> {
        Ok(self.0.lock().unwrap().version)
    }
    async fn snapshot(&self) -> Result<InputSnapshot, StoreError> {
        Ok(self.0.lock().unwrap().snapshot())
    }
}

/// In-memory inputs: the test and `seikika dev` store.
#[derive(Debug, Clone, Default)]
pub struct MemStore {
    version: u64,
    records: BTreeMap<(String, String, String), Value>,
    native: BTreeMap<(String, String), NativeValue>,
}

impl MemStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply connector messages from `connector` as one batch. `record` upserts, `delete` removes;
    /// other message types (register, ack, complete, …) carry no input data and are ignored here.
    pub fn ingest(&mut self, connector: &str, batch: &[Value]) -> u64 {
        for msg in batch {
            let field = |k: &str| {
                msg.get(k)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned()
            };
            let key = (connector.to_owned(), field("kind"), field("externalId"));
            match msg.get("type").and_then(Value::as_str) {
                Some("record") => {
                    self.records
                        .insert(key, msg.get("data").cloned().unwrap_or(Value::Null));
                }
                Some("delete") => {
                    self.records.remove(&key);
                }
                _ => {}
            }
        }
        self.version += 1;
        self.version
    }

    /// Write native values; for one (entity, field) the highest revision wins, ties go to the later write.
    pub fn put_native(&mut self, values: impl IntoIterator<Item = NativeValue>) -> u64 {
        for v in values {
            let key = (v.entity.clone(), v.field.clone());
            match self.native.get(&key) {
                Some(old) if old.revision > v.revision => {}
                _ => {
                    self.native.insert(key, v);
                }
            }
        }
        self.version += 1;
        self.version
    }

    pub fn snapshot(&self) -> InputSnapshot {
        InputSnapshot {
            version: self.version,
            records: self
                .records
                .iter()
                .map(|((c, k, id), data)| Record {
                    connector: c.clone(),
                    kind: k.clone(),
                    external_id: id.clone(),
                    data: data.clone(),
                })
                .collect(),
            native: self.native.values().cloned().collect(),
        }
    }
}

/// Message batches per connector, in file order.
pub type Batches = Vec<(String, Vec<Value>)>;

/// Read `fixtures/<connector>.ndjson` files (connector = file stem) and `fixtures/native.json`.
pub fn read_fixtures(root: &Path) -> Result<(Batches, Vec<NativeValue>), StoreError> {
    let dir = root.join("fixtures");
    let mut batches = Vec::new();
    let mut native = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok((batches, native));
    };
    let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let text =
            std::fs::read_to_string(&path).map_err(|e| StoreError::Io(format!("{name}: {e}")))?;
        if name.ends_with(".ndjson") {
            let connector = name.trim_end_matches(".ndjson").to_owned();
            let mut batch = Vec::new();
            for (i, line) in text.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                batch.push(serde_json::from_str(line).map_err(|e| StoreError::Message {
                    file: format!("fixtures/{name}"),
                    line: i + 1,
                    message: e.to_string(),
                })?);
            }
            batches.push((connector, batch));
        } else if name == "native.json" {
            native = serde_json::from_str(&text).map_err(|e| StoreError::Message {
                file: "fixtures/native.json".into(),
                line: 0,
                message: e.to_string(),
            })?;
        }
    }
    Ok((batches, native))
}

/// A `MemStore` loaded with a portal's fixtures.
pub fn load_fixtures(root: &Path) -> Result<MemStore, StoreError> {
    let (batches, native) = read_fixtures(root)?;
    let mut store = MemStore::new();
    for (connector, batch) in batches {
        store.ingest(&connector, &batch);
    }
    if !native.is_empty() {
        store.put_native(native);
    }
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn snapshot_is_ordered_and_deletes_apply() {
        let mut s = MemStore::new();
        s.ingest(
            "b",
            &[json!({"type": "record", "kind": "k", "externalId": "2", "data": {"v": 2}})],
        );
        s.ingest(
            "a",
            &[
                json!({"type": "record", "kind": "k", "externalId": "9", "data": {}}),
                json!({"type": "record", "kind": "k", "externalId": "1", "data": {}}),
                json!({"type": "delete", "kind": "k", "externalId": "9"}),
            ],
        );
        let snap = s.snapshot();
        let ids: Vec<_> = snap
            .records
            .iter()
            .map(|r| (r.connector.as_str(), r.external_id.as_str()))
            .collect();
        assert_eq!(ids, vec![("a", "1"), ("b", "2")]);
        assert_eq!(snap.version, 2);
    }

    fn native(field: &str, value: i64, revision: u64) -> NativeValue {
        serde_json::from_value(
            json!({"entity": "e", "field": field, "value": value, "revision": revision}),
        )
        .expect("a native value")
    }

    #[test]
    fn the_highest_native_revision_wins_and_ties_go_to_the_later_write() {
        let mut s = MemStore::new();
        s.put_native([native("a", 1, 2), native("b", 1, 1)]);
        s.put_native([native("a", 9, 1), native("b", 2, 1)]);
        let snap = s.snapshot();
        let got: Vec<_> = snap
            .native
            .iter()
            .map(|v| (v.field.as_str(), v.value.clone()))
            .collect();
        assert_eq!(got, vec![("a", json!(1)), ("b", json!(2))]);
    }

    #[test]
    fn fixtures_load_per_connector_and_bad_lines_name_their_place() {
        let root = std::env::temp_dir().join(format!("seikika-store-{}", std::process::id()));
        let dir = root.join("fixtures");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("git.ndjson"),
            "{\"type\": \"record\", \"kind\": \"k\", \"externalId\": \"1\", \"data\": {}}\n\n",
        )
        .unwrap();
        let store = load_fixtures(&root).unwrap();
        assert_eq!(store.snapshot().records[0].connector, "git");
        std::fs::write(dir.join("bad.ndjson"), "{}\nnot json\n").unwrap();
        let err = read_fixtures(&root).unwrap_err().to_string();
        assert!(err.starts_with("fixtures/bad.ndjson:2:"), "{err}");
        let _ = std::fs::remove_dir_all(&root);
        assert!(read_fixtures(&root).unwrap().0.is_empty());
    }
}
