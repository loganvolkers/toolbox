//! Queue commands and track connector state on the portal side of the control channel.
//!
//! The hub keeps registrations, queues, and command history in memory. Restarting the portal
//! clears that state. Connector binaries implement [`crate::sdk::Source`].

use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Notify;

/// The lifecycle state of a connector command.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Queued or delivered without a matching acknowledgement yet.
    Pending,
    /// Acknowledged by the connector and awaiting completion.
    Acked,
    /// Completed successfully.
    Succeeded,
    /// Completed with partial results.
    Partial,
    /// Completed with a failure reported by the connector.
    Failed,
    /// Completed with cancellation reported by the connector.
    Cancelled,
    /// The hub observed that the deadline had passed. A late completion can still update the state.
    TimedOut,
}

impl Status {
    /// Returns whether this state is beyond queueing and acknowledgement.
    ///
    /// A terminal state can represent success, failure, cancellation, or timeout.
    pub fn terminal(&self) -> bool {
        !matches!(self, Status::Pending | Status::Acked)
    }
}

/// A queued or completed connector command and its observed result.
#[derive(Debug, Clone, Serialize)]
pub struct Command {
    /// Command ID assigned by the hub.
    pub id: String,
    /// Portal manifest name of the connector that receives this command.
    pub connector: String,
    /// Wire command name, such as `refresh` or `resync`.
    pub command: String,
    /// Record kind selected for a `refresh`.
    pub kind: Option<String>,
    /// Record kinds selected for a `resync`.
    pub kinds: Vec<String>,
    /// External IDs requested by a scoped refresh.
    pub external_ids: Vec<String>,
    /// Reason passed to the connector with the command.
    pub reason: String,
    /// Dispatch priority, with `interactive` served first.
    pub priority: String,
    /// Deadline in whole Unix seconds.
    pub deadline: u64,
    /// Principal that requested the command.
    pub requested_by: String,
    /// Latest lifecycle state observed by the hub.
    pub status: Status,
    /// Input version of the batch that carried `complete`, if received.
    pub completed_version: Option<u64>,
    /// Protocol error objects from the completion message, if any.
    pub errors: Vec<Value>,
    /// Counters reported in the completion message. Starts as an empty object.
    pub counts: Value,
    /// Other command IDs recorded as merged into this one. New commands start with an empty list.
    pub merged: Vec<String>,
}

impl Command {
    /// Builds the wire command sent to the connector.
    ///
    /// The message includes request context and a UTC deadline. For `refresh` it carries `kind`
    /// and `external_ids`. For `resync` it carries `kinds`.
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

/// Returns the current time in whole Unix seconds, or zero for a clock before the epoch.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Formats whole Unix seconds as UTC with a `Z` suffix and no fractional seconds.
///
/// Keep times within the four-digit year range for RFC 3339 and [`parse_rfc3339`].
///
/// # Examples
///
/// ```
/// use seikika_connect::hub::{parse_rfc3339, rfc3339};
///
/// let start = rfc3339(0);
/// assert_eq!(start, "1970-01-01T00:00:00Z");
/// assert_eq!(parse_rfc3339(&start), Some(0));
/// ```
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

/// Parses whole Unix seconds from the UTC format produced by [`rfc3339`].
///
/// Returns `None` if the expected fields cannot be read or the result precedes the Unix epoch.
/// Pass trusted timestamps in `YYYY-MM-DDTHH:MM:SSZ` form. This helper reads fixed-position fields
/// and performs limited format checks. Use a full RFC 3339 parser to validate user input or handle offsets.
pub fn parse_rfc3339(s: &str) -> Option<u64> {
    let b = s.as_bytes();
    if b.len() < 20 || b[4] != b'-' || b[10] != b'T' {
        return None;
    }
    let n = |r: std::ops::Range<usize>| s.get(r)?.parse::<i64>().ok();
    let (y, m, d) = (n(0..4)?, n(5..7)?, n(8..10)?);
    let (hh, mm, ss) = (n(11..13)?, n(14..16)?, n(17..19)?);
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    u64::try_from(days * 86_400 + hh * 3_600 + mm * 60 + ss).ok()
}

/// The most recent registration accepted for a connector name.
#[derive(Debug, Clone, Serialize)]
pub struct Registration {
    /// Instance name reported by the connector process.
    pub instance: String,
    /// Connector package version reported by that instance.
    pub version: String,
    /// Wire capability object used to determine supported kinds and scopes.
    pub capabilities: Value,
    /// Time the hub received the registration, in whole Unix seconds.
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
    /// The input version that holds each (connector, kind)'s first finished sync, successful or not.
    tried: HashMap<(String, String), u64>,
}

/// The portal's in-memory registry and dispatch queues for connector commands.
///
/// Create one per portal process and share it between command requesters and transports. Methods
/// that lock shared state can panic if another thread panicked while holding the lock.
#[derive(Default)]
pub struct Hub {
    inner: Mutex<Inner>,
    notify: Notify,
}

/// Requests a scoped refresh or full resync from a named connector.
pub struct Request {
    /// Connector name declared in the portal manifest.
    pub connector: String,
    /// Wire command name. Use `refresh` for external IDs and `resync` for a full read.
    pub command: &'static str,
    /// Single record kind for `refresh`. Use `None` for `resync`.
    pub kind: Option<String>,
    /// Record kinds for `resync`. Leave empty for `refresh`.
    pub kinds: Vec<String>,
    /// External IDs for `refresh`. Leave empty for `resync`.
    pub external_ids: Vec<String>,
    /// Reason included in the wire message.
    pub reason: String,
    /// Dispatch priority. In order, `interactive`, `verification`, `freshness`, and `background`.
    pub priority: &'static str,
    /// Time allowed for a new command, rounded down to whole seconds with a minimum of one.
    pub timeout: Duration,
    /// Principal that requested the command.
    pub requested_by: String,
}

impl Hub {
    /// Creates a hub with no registered connectors or queued commands.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues a command and returns its ID for [`Self::get`] or [`Self::wait`].
    ///
    /// Returns the existing ID if a nonterminal command already covers the request. A refresh
    /// covers a second refresh of the same kind if it includes all the requested IDs. A reused
    /// command keeps its original priority and deadline.
    ///
    /// # Panics
    ///
    /// Panics on a poisoned hub lock. An excessive timeout can overflow the Unix deadline.
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

    /// Drains queued command messages for `connector` in priority order.
    ///
    /// Run this on a Tokio runtime with time enabled. Returns an empty list if no dispatchable
    /// command arrives before `wait`. Delivery leaves a command pending until acknowledgement.
    ///
    /// # Panics
    ///
    /// Panics on a poisoned hub lock, or if it needs to wait and Tokio time is unavailable.
    /// An excessive wait can overflow the deadline.
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

    /// Applies control messages from a connector batch committed as input `version`.
    ///
    /// Pass validated messages and the authenticated connector name. Each returned pair contains
    /// a kind to reconcile and the external IDs seen for that kind during the resync. The caller
    /// applies the reconciliation to the input store.
    ///
    /// # Panics
    ///
    /// Panics on a poisoned hub lock.
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
                "complete" => Self::complete(&mut g, connector, &cid, m, version),
                _ => {}
            }
        }
        drop(g);
        self.notify.notify_waiters();
        reconcile
    }

    /// Returns a copy of the command, marking overdue work before reading its status.
    ///
    /// Returns `None` for an unknown ID. See [`Hub`] for lock poisoning behavior.
    pub fn get(&self, id: &str) -> Option<Command> {
        let mut g = self.inner.lock().unwrap();
        self.expire(&mut g);
        g.commands.get(id).cloned()
    }

    /// Waits for a terminal state or returns the latest command when `wait` elapses.
    ///
    /// Returns `None` immediately for an unknown ID. Check [`Command::status`] to see whether the
    /// command finished. Run this on a Tokio runtime with time enabled.
    ///
    /// # Panics
    ///
    /// Panics on a poisoned hub lock, or if it needs to wait and Tokio time is unavailable.
    /// An excessive wait can overflow the deadline.
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

    /// Returns the latest registration for `connector`, if it has registered in this process.
    pub fn registration(&self, connector: &str) -> Option<Registration> {
        self.inner
            .lock()
            .unwrap()
            .registrations
            .get(connector)
            .cloned()
    }

    /// Returns a copy of the latest registrations keyed by connector name.
    pub fn registrations(&self) -> HashMap<String, Registration> {
        self.inner.lock().unwrap().registrations.clone()
    }

    /// Returns the completion time of the last successful read, in whole Unix seconds.
    ///
    /// Both a scoped refresh and a full resync can update the timestamp for `kind`. Returns
    /// `None` if no successful read of `kind` has been recorded for the connector.
    pub fn last_read(&self, connector: &str, kind: &str) -> Option<u64> {
        self.inner
            .lock()
            .unwrap()
            .last_read
            .get(&(connector.to_owned(), kind.to_owned()))
            .copied()
    }

    /// A command finished: record its outcome and the kinds it read.
    fn complete(g: &mut Inner, connector: &str, cid: &str, m: &Value, version: u64) {
        let status = match m.get("status").and_then(Value::as_str).unwrap_or_default() {
            "succeeded" => Status::Succeeded,
            "partial" => Status::Partial,
            "cancelled" => Status::Cancelled,
            _ => Status::Failed,
        };
        let mut read = Vec::new();
        let mut tried = Vec::new();
        if let Some(c) = g.commands.get_mut(cid) {
            tried.extend(c.kind.iter().cloned().chain(c.kinds.iter().cloned()));
            if status == Status::Succeeded {
                read.extend(tried.iter().cloned());
            }
            c.status = status;
            c.completed_version = Some(version);
            c.errors = m["errors"].as_array().cloned().unwrap_or_default();
            c.counts = m.get("counts").cloned().unwrap_or(json!({}));
        }
        for k in read {
            g.last_read.insert((connector.to_owned(), k), now());
        }
        for k in tried {
            g.tried.entry((connector.to_owned(), k)).or_insert(version);
        }
    }

    /// Returns the first input version where `connector` reported completion for `kind`.
    ///
    /// This includes failed or cancelled completions. Returns `None` until the connector reports
    /// a completion for a command covering the kind.
    pub fn first_sync(&self, connector: &str, kind: &str) -> Option<u64> {
        self.inner
            .lock()
            .unwrap()
            .tried
            .get(&(connector.to_owned(), kind.to_owned()))
            .copied()
    }

    /// Returns queued and acknowledged commands, after marking overdue work as timed out.
    pub fn pending(&self) -> Vec<Command> {
        let mut g = self.inner.lock().unwrap();
        self.expire(&mut g);
        g.commands
            .values()
            .filter(|c| !c.status.terminal())
            .cloned()
            .collect()
    }

    /// Returns whether the connector's latest registration supports refreshing `kind` by `scope`.
    pub fn can_refresh(&self, connector: &str, kind: &str, scope: &str) -> bool {
        self.registration(connector).is_some_and(|r| {
            r.capabilities["refresh"][kind]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == scope))
        })
    }

    /// Returns whether the connector's latest registration lists `kind` for a full resync.
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
