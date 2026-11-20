//! The server side of the control channel (connector-control §4–§5): registrations, a durable-ish
//! command registry with per-connector queues, coalescing, priorities, deadlines, acknowledgements
//! and completion. In-memory in v1 (commands are lost on restart; durable workflows arrive in M5).

use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Notify;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pending,
    Acked,
    Succeeded,
    Partial,
    Failed,
    Cancelled,
    TimedOut,
}

impl Status {
    pub fn terminal(&self) -> bool {
        !matches!(self, Status::Pending | Status::Acked)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Command {
    pub id: String,
    pub connector: String,
    pub command: String,
    pub kind: Option<String>,
    pub kinds: Vec<String>,
    pub external_ids: Vec<String>,
    pub reason: String,
    pub priority: String,
    pub deadline: u64,
    pub requested_by: String,
    pub status: Status,
    /// Input version of the batch that carried `complete`; the result snapshot covers it.
    pub completed_version: Option<u64>,
    pub errors: Vec<Value>,
    pub counts: Value,
    /// Other command ids coalesced into this one.
    pub merged: Vec<String>,
}

impl Command {
    /// The wire form (schema/connector-messages.schema.json `command`).
    pub fn message(&self) -> Value {
        let mut m = json!({"type": "command", "command_id": self.id, "command": self.command,
                           "reason": self.reason, "priority": self.priority,
                           "deadline": rfc3339(self.deadline)});
        match self.command.as_str() {
            "refresh" => {
                m["kind"] = json!(self.kind);
                m["external_ids"] = json!(self.external_ids);
            }
            "resync" => m["kinds"] = json!(self.kinds),
            _ => {}
        }
        m
    }
}

/// Seconds since the epoch.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// RFC 3339 UTC from epoch seconds (no external date library).
pub fn rfc3339(secs: u64) -> String {
    let days = secs / 86_400;
    let (h, m, s) = ((secs % 86_400) / 3600, (secs % 3600) / 60, secs % 60);
    // Civil-from-days (Howard Hinnant's algorithm).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(mo <= 2);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

#[derive(Debug, Clone, Serialize)]
pub struct Registration {
    pub instance: String,
    pub version: String,
    pub capabilities: Value,
    pub at: u64,
}

#[derive(Default)]
struct Inner {
    seq: u64,
    commands: BTreeMap<String, Command>,
    queues: HashMap<String, VecDeque<String>>,
    registrations: HashMap<String, Registration>,
    /// (command, kind) → external ids seen, for resync reconciliation.
    seen: HashMap<(String, String), BTreeSet<String>>,
    /// (connector, kind) → last successful full read or refresh (epoch secs).
    last_read: HashMap<(String, String), u64>,
}

#[derive(Default)]
pub struct Hub {
    inner: Mutex<Inner>,
    notify: Notify,
}

/// What a scoped refresh or resync asks for.
pub struct Request {
    pub connector: String,
    pub command: &'static str,
    pub kind: Option<String>,
    pub kinds: Vec<String>,
    pub external_ids: Vec<String>,
    pub reason: String,
    pub priority: &'static str,
    pub timeout: Duration,
    pub requested_by: String,
}

impl Hub {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a command, coalescing with a pending one for the same connector and scope
    /// (connector-control §5). Returns the id the caller should wait on.
    pub fn enqueue(&self, r: Request) -> String {
        let mut g = self.inner.lock().unwrap();
        let ids: BTreeSet<&String> = r.external_ids.iter().collect();
        let existing = g.commands.values().find(|c| {
            !c.status.terminal()
                && c.connector == r.connector
                && c.command == r.command
                && c.kind == r.kind
                && c.kinds == r.kinds
                && (r.command != "refresh" || ids.iter().all(|i| c.external_ids.contains(i)))
        });
        if let Some(c) = existing {
            return c.id.clone();
        }
        g.seq += 1;
        let id = format!("cmd_{:06}{}", g.seq, now() % 100_000);
        let priority_rank = |p: &str| match p {
            "interactive" => 0,
            "verification" => 1,
            "freshness" => 2,
            _ => 3,
        };
        let cmd = Command {
            id: id.clone(),
            connector: r.connector.clone(),
            command: r.command.into(),
            kind: r.kind,
            kinds: r.kinds,
            external_ids: r.external_ids,
            reason: r.reason,
            priority: r.priority.into(),
            deadline: now() + r.timeout.as_secs().max(1),
            requested_by: r.requested_by,
            status: Status::Pending,
            completed_version: None,
            errors: vec![],
            counts: json!({}),
            merged: vec![],
        };
        let rank = priority_rank(&cmd.priority);
        g.commands.insert(id.clone(), cmd);
        let Inner {
            queues, commands, ..
        } = &mut *g;
        let q = queues.entry(r.connector).or_default();
        let pos = q
            .iter()
            .position(|other| {
                commands
                    .get(other)
                    .is_some_and(|c| priority_rank(&c.priority) > rank)
            })
            .unwrap_or(q.len());
        q.insert(pos, id.clone());
        drop(g);
        self.notify.notify_waiters();
        id
    }

    /// Take the queued commands for `connector`, waiting up to `wait` for one to arrive.
    pub async fn take(&self, connector: &str, wait: Duration) -> Vec<Value> {
        let deadline = tokio::time::Instant::now() + wait;
        loop {
            let notified = self.notify.notified();
            {
                let mut g = self.inner.lock().unwrap();
                self.expire(&mut g);
                let ids: Vec<String> = g
                    .queues
                    .get_mut(connector)
                    .map(|q| q.drain(..).collect())
                    .unwrap_or_default();
                let msgs: Vec<Value> = ids
                    .iter()
                    .filter_map(|id| g.commands.get(id))
                    .filter(|c| !c.status.terminal())
                    .map(Command::message)
                    .collect();
                if !msgs.is_empty() {
                    return msgs;
                }
            }
            if tokio::time::timeout_at(deadline, notified).await.is_err() {
                return vec![];
            }
        }
    }

    fn expire(&self, g: &mut Inner) {
        let t = now();
        for c in g.commands.values_mut() {
            if !c.status.terminal() && c.deadline < t {
                c.status = Status::TimedOut;
            }
        }
    }

    /// Record control messages from `connector` that arrived in the batch committed as
    /// `version`. Returns resync reconciliations to perform: (kind, external ids seen).
    pub fn on_messages(
        &self,
        connector: &str,
        msgs: &[Value],
        version: u64,
    ) -> Vec<(String, BTreeSet<String>)> {
        let mut g = self.inner.lock().unwrap();
        let mut reconcile = Vec::new();
        for m in msgs {
            let s = |k: &str| {
                m.get(k)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned()
            };
            let cid = s("command_id");
            match s("type").as_str() {
                "register" => {
                    g.registrations.insert(
                        connector.to_owned(),
                        Registration {
                            instance: s("instance"),
                            version: s("version"),
                            capabilities: m["capabilities"].clone(),
                            at: now(),
                        },
                    );
                }
                "ack" => {
                    if let Some(c) = g.commands.get_mut(&cid)
                        && c.status == Status::Pending
                    {
                        c.status = Status::Acked;
                    }
                }
                "record" | "delete" | "not_found" if !cid.is_empty() => {
                    let key = (cid.clone(), s("kind"));
                    g.seen.entry(key).or_default().insert(s("externalId"));
                }
                "snapshot_complete" => {
                    let key = (cid.clone(), s("kind"));
                    let seen = g.seen.remove(&key).unwrap_or_default();
                    let failed = g.commands.get(&cid).is_some_and(|c| !c.errors.is_empty());
                    if !failed {
                        reconcile.push((s("kind"), seen));
                    }
                }
                "complete" => {
                    let status = match s("status").as_str() {
                        "succeeded" => Status::Succeeded,
                        "partial" => Status::Partial,
                        "cancelled" => Status::Cancelled,
                        _ => Status::Failed,
                    };
                    let mut read = Vec::new();
                    if let Some(c) = g.commands.get_mut(&cid) {
                        if status == Status::Succeeded {
                            read.extend(c.kind.iter().cloned().chain(c.kinds.iter().cloned()));
                        }
                        c.status = status;
                        c.completed_version = Some(version);
                        c.errors = m["errors"].as_array().cloned().unwrap_or_default();
                        c.counts = m.get("counts").cloned().unwrap_or(json!({}));
                    }
                    for k in read {
                        g.last_read.insert((connector.to_owned(), k), now());
                    }
                }
                _ => {}
            }
        }
        drop(g);
        self.notify.notify_waiters();
        reconcile
    }

    pub fn get(&self, id: &str) -> Option<Command> {
        let mut g = self.inner.lock().unwrap();
        self.expire(&mut g);
        g.commands.get(id).cloned()
    }

    /// Wait until command `id` is terminal (or `wait` passes).
    pub async fn wait(&self, id: &str, wait: Duration) -> Option<Command> {
        let deadline = tokio::time::Instant::now() + wait;
        loop {
            let notified = self.notify.notified();
            let c = self.get(id)?;
            if c.status.terminal() || tokio::time::Instant::now() >= deadline {
                return Some(c);
            }
            let _ = tokio::time::timeout_at(
                deadline.min(tokio::time::Instant::now() + Duration::from_millis(500)),
                notified,
            )
            .await;
        }
    }

    pub fn registration(&self, connector: &str) -> Option<Registration> {
        self.inner
            .lock()
            .unwrap()
            .registrations
            .get(connector)
            .cloned()
    }

    pub fn registrations(&self) -> HashMap<String, Registration> {
        self.inner.lock().unwrap().registrations.clone()
    }

    pub fn last_read(&self, connector: &str, kind: &str) -> Option<u64> {
        self.inner
            .lock()
            .unwrap()
            .last_read
            .get(&(connector.to_owned(), kind.to_owned()))
            .copied()
    }

    pub fn pending(&self) -> Vec<Command> {
        let mut g = self.inner.lock().unwrap();
        self.expire(&mut g);
        g.commands
            .values()
            .filter(|c| !c.status.terminal())
            .cloned()
            .collect()
    }

    /// Whether the connector registered `scope` for refreshing `kind`.
    pub fn can_refresh(&self, connector: &str, kind: &str, scope: &str) -> bool {
        self.registration(connector).is_some_and(|r| {
            r.capabilities["refresh"][kind]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == scope))
        })
    }

    pub fn can_resync(&self, connector: &str, kind: &str) -> bool {
        self.registration(connector).is_some_and(|r| {
            r.capabilities["resync"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == kind))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(ids: &[&str], priority: &'static str) -> Request {
        Request {
            connector: "github".into(),
            command: "refresh",
            kind: Some("repository".into()),
            kinds: vec![],
            external_ids: ids.iter().map(|s| s.to_string()).collect(),
            reason: "test".into(),
            priority,
            timeout: Duration::from_secs(60),
            requested_by: "user:alice".into(),
        }
    }

    #[tokio::test]
    async fn coalescing_priorities_and_completion() {
        let hub = Hub::new();
        let a = hub.enqueue(req(&["1", "2"], "background"));
        assert_eq!(
            hub.enqueue(req(&["2"], "interactive")),
            a,
            "covered by a pending command"
        );
        let b = hub.enqueue(req(&["3"], "interactive"));
        let msgs = hub.take("github", Duration::from_millis(10)).await;
        assert_eq!(msgs[0]["command_id"], b.as_str(), "interactive first");
        assert_eq!(msgs[1]["command_id"], a.as_str());
        hub.on_messages("github", &[json!({"type": "ack", "command_id": a})], 1);
        assert_eq!(hub.get(&a).unwrap().status, Status::Acked);
        hub.on_messages(
            "github",
            &[json!({"type": "complete", "command_id": a, "status": "succeeded"})],
            7,
        );
        let c = hub.wait(&a, Duration::from_secs(1)).await.unwrap();
        assert_eq!(
            (c.status, c.completed_version),
            (Status::Succeeded, Some(7))
        );
        assert!(
            rfc3339(0) == "1970-01-01T00:00:00Z"
                && rfc3339(1_763_632_800) == "2025-11-20T10:00:00Z"
        );
    }
}
