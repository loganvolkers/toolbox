//! Server state and the snapshot loop (docs/v1-spec.md §6.1, §8): wait for input changes,
//! debounce, take a consistent snapshot, recompute, swap the result in atomically, and (on the
//! publishing instance) record the snapshot and the tuple delta in Postgres.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{Notify, watch};
use toolbox_derive::{Derived, DerivedState, FullRecompute};
use toolbox_model::Manifest;
use toolbox_store::{PgStore, Store};

/// One published snapshot.
#[derive(Debug)]
pub struct Published {
    pub snapshot_id: String,
    pub input_version: u64,
    pub derived: Arc<Derived>,
    pub recompute: Duration,
    pub published_at: std::time::SystemTime,
    /// `Some` only on the instance that wrote this snapshot's tuples: a follower can ask OpenFGA
    /// before the leader has written them, so it must not keep the answer.
    pub decisions: Option<Decisions>,
    pub by_field: FieldIndex,
}

/// Entity ids by the string value of one declared field of one kind, built for a snapshot the
/// first time a list filters on that field.
#[derive(Debug, Default)]
pub struct FieldIndex(Mutex<HashMap<(String, String), Arc<IdsByValue>>>);

pub type IdsByValue = HashMap<String, Vec<String>>;

impl FieldIndex {
    /// Ids in id order for each value of `field` across entities of `kind`.
    pub fn get(
        &self,
        derived: &Derived,
        kind: &str,
        field: &str,
    ) -> Arc<HashMap<String, Vec<String>>> {
        let key = (kind.to_owned(), field.to_owned());
        if let Some(ix) = self.0.lock().unwrap().get(&key) {
            return ix.clone();
        }
        let mut ix = IdsByValue::new();
        for (id, e) in derived.entities.iter().filter(|(_, e)| e.kind == kind) {
            if let Some(v) = e.fields.get(field).and_then(Value::as_str) {
                ix.entry(v.to_owned()).or_default().push(id.clone());
            }
        }
        let ix = Arc::new(ix);
        self.0.lock().unwrap().insert(key, ix.clone());
        ix
    }
}

/// Authorization decisions made against one snapshot. Every tuple is derived, so tuples change
/// only when a snapshot is published, and a decision holds for exactly the snapshot it was made
/// in: unlike a TTL cache, a new entity or a revoked grant is never served stale.
#[derive(Debug, Default)]
pub struct Decisions(Mutex<HashMap<toolbox_authz::Tuple, bool>>);

impl Decisions {
    /// Bounds memory per snapshot; past it, decisions are still made, just not kept.
    const MAX: usize = 200_000;

    pub fn get(&self, t: &toolbox_authz::Tuple) -> Option<bool> {
        self.0.lock().unwrap().get(t).copied()
    }

    pub fn put(&self, t: toolbox_authz::Tuple, allowed: bool) {
        let mut m = self.0.lock().unwrap();
        if m.len() < Self::MAX {
            m.insert(t, allowed);
        }
    }
}

pub struct AppState {
    pub hub: toolbox_connect::Hub,
    pub authn: toolbox_authz::authn::Authenticator,
    /// `None` = no authorization (in-memory development without OpenFGA): everything is visible.
    pub authz: Option<Arc<dyn toolbox_authz::Authorizer>>,
    pub manifest: Arc<Manifest>,
    pub manifest_hash: String,
    pub store: Arc<dyn Store>,
    pub pg: Option<PgStore>,
    pub current: watch::Sender<Option<Arc<Published>>>,
    pub changed: Notify,
    pub config: Config,
}

#[derive(Debug, Clone)]
pub struct Config {
    /// `dev` trusts `X-Toolbox-User` / `X-Toolbox-Connector`; production modes arrive in M3.
    pub auth_mode: String,
    /// connector name → bearer token
    pub connector_tokens: Vec<(String, String)>,
    pub debounce: Duration,
    pub max_lag: Duration,
    pub poll: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            auth_mode: "dev".into(),
            connector_tokens: Vec::new(),
            debounce: Duration::from_millis(500),
            max_lag: Duration::from_secs(5),
            poll: Duration::from_secs(1),
        }
    }
}

/// sha256 of the canonical (key-sorted) manifest JSON, first 12 hex digits (D53).
pub fn manifest_hash(doc: &Value) -> String {
    let digest = Sha256::digest(toolbox_derive::pyjson::compact(doc).as_bytes());
    digest.iter().take(6).map(|b| format!("{b:02x}")).collect()
}

/// Parse `min_snapshot`: an input version, or a full snapshot id `<version>.<hash>`.
pub fn parse_min_snapshot(s: &str) -> Option<u64> {
    s.split('.').next()?.parse().ok()
}

impl AppState {
    pub fn snapshot(&self) -> Option<Arc<Published>> {
        self.current.borrow().clone()
    }

    /// Wait until the published snapshot covers `version` (or `timeout` passes).
    pub async fn wait_for(&self, version: u64, timeout: Duration) -> Option<Arc<Published>> {
        let mut rx = self.current.subscribe();
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(p) = rx.borrow().clone()
                && p.input_version >= version
            {
                return Some(p);
            }
            if tokio::time::timeout_at(deadline, rx.changed())
                .await
                .is_err()
            {
                return None;
            }
        }
    }

    async fn recompute_and_publish(
        &self,
        pg: Option<&PgStore>,
        previous_tuples: &mut Option<BTreeSet<(String, String, String)>>,
        synced: &mut BTreeSet<(String, String, String)>,
    ) {
        let started = Instant::now();
        let inputs = match self.store.snapshot().await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "snapshot failed");
                return;
            }
        };
        let manifest = self.manifest.clone();
        let derived =
            tokio::task::spawn_blocking(move || FullRecompute.recompute(&manifest, &inputs))
                .await
                .expect("recompute task");
        let input_version: u64 = derived.snapshot.parse().unwrap_or(0);
        let snapshot_id = format!("{input_version}.{}", self.manifest_hash);
        let elapsed = started.elapsed();
        toolbox_telemetry::catalog::record_recompute(elapsed.as_secs_f64());
        let mut tuples_written = false;
        if let Some(pg) = pg {
            let tuples: BTreeSet<(String, String, String)> = derived
                .tuples
                .iter()
                .map(|t| {
                    let s = |k: &str| {
                        t.get(k)
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned()
                    };
                    (s("object"), s("relation"), s("user"))
                })
                .collect();
            let (added, removed, full) = match previous_tuples {
                Some(prev) => (
                    tuples.difference(prev).cloned().collect(),
                    prev.difference(&tuples).cloned().collect(),
                    false,
                ),
                None => (tuples.iter().cloned().collect::<Vec<_>>(), Vec::new(), true),
            };
            let summary = json!({
                "entities": derived.entities.len(),
                "tuples": derived.tuples.len(),
                "orphans": derived.orphans.len(),
                "diagnostics": derived.diagnostics.len(),
                "recompute_ms": elapsed.as_millis() as u64,
            });
            let publication = toolbox_store::Publication {
                snapshot_id: &snapshot_id,
                input_version,
                manifest_hash: &self.manifest_hash,
                summary: &summary,
                added: &added,
                removed: &removed,
                full,
            };
            match pg.publish(&publication).await {
                Ok(()) => {
                    *previous_tuples = Some(tuples);
                    tuples_written = true;
                }
                Err(e) => tracing::error!(error = %e, "publish failed"),
            }
        }
        // In-memory OpenFGA holds tuples itself: apply the delta before publishing the snapshot,
        // so authorization never lags the data it guards. (With Postgres, OpenFGA reads the view.)
        if let Some(authz) = &self.authz {
            let now: BTreeSet<(String, String, String)> = derived
                .tuples
                .iter()
                .map(|t| {
                    let s = |k: &str| {
                        t.get(k)
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_owned()
                    };
                    (s("user"), s("relation"), s("object"))
                })
                .collect();
            let added: Vec<_> = now.difference(synced).cloned().collect();
            let removed: Vec<_> = synced.difference(&now).cloned().collect();
            match authz.sync_tuples(&added, &removed).await {
                Ok(()) => {
                    *synced = now;
                    // With Postgres, OpenFGA reads the tuples the publish above wrote.
                    tuples_written |= self.pg.is_none();
                }
                Err(e) => tracing::error!(error = %e, "authorization tuple sync failed"),
            }
        }
        let published = Arc::new(Published {
            snapshot_id,
            input_version,
            derived: Arc::new(derived),
            recompute: elapsed,
            published_at: std::time::SystemTime::now(),
            decisions: tuples_written.then(Decisions::default),
            by_field: FieldIndex::default(),
        });
        self.current.send_replace(Some(published));
    }

    /// The snapshot loop. Runs until the process exits.
    pub async fn run_snapshot_loop(self: Arc<Self>) {
        // Only the advisory-lock holder publishes snapshots and tuple deltas (v1-spec §8).
        let mut leader = None;
        let mut previous_tuples = None;
        let mut synced = BTreeSet::new();
        loop {
            if let (Some(pg), None) = (&self.pg, &leader) {
                leader = pg.try_lead().await.ok().flatten();
            }
            let current = self.snapshot().map(|p| p.input_version);
            let latest = self.store.input_version().await.unwrap_or(0);
            if current.is_none() || current != Some(latest) {
                // Debounce: wait until inputs stop changing for `debounce`, at most `max_lag`.
                let first_seen = Instant::now();
                let mut seen = latest;
                loop {
                    tokio::select! {
                        _ = self.changed.notified() => {}
                        _ = tokio::time::sleep(self.config.debounce) => {}
                    }
                    let now = self.store.input_version().await.unwrap_or(seen);
                    if (now == seen && current.is_some())
                        || first_seen.elapsed() >= self.config.max_lag
                        || current.is_none()
                    {
                        break;
                    }
                    seen = now;
                }
                let pg = if leader.is_some() {
                    self.pg.as_ref()
                } else {
                    None
                };
                self.recompute_and_publish(pg, &mut previous_tuples, &mut synced)
                    .await;
                continue;
            }
            tokio::select! {
                _ = self.changed.notified() => {}
                _ = tokio::time::sleep(self.config.poll) => {}
            }
        }
    }
}
