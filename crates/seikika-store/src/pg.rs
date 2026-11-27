//! `PgStore`: inputs in Postgres (docs/v1-spec.md §8). Every committed batch gets an input version
//! from a sequence; snapshots are `REPEATABLE READ` reads of all inputs at one version.

use crate::{InputSnapshot, NativeValue, Record, Store, StoreError};
use serde_json::Value;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Postgres, Row, Transaction};
use std::num::NonZeroU32;

/// Migrations, applied in order at start-up under an advisory lock. Each is idempotent
/// (expand-only, `IF NOT EXISTS`), so re-running is safe.
pub const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_inputs", include_str!("../migrations/0001_inputs.sql")),
    ("0002_tokens", include_str!("../migrations/0002_tokens.sql")),
    ("0003_runs", include_str!("../migrations/0003_runs.sql")),
];

#[derive(Clone)]
pub struct PgStore {
    pub pool: PgPool,
}

fn err(e: impl std::fmt::Display) -> StoreError {
    StoreError::Io(e.to_string())
}

impl PgStore {
    /// Connect and apply migrations.
    pub async fn connect(url: &str) -> Result<Self, StoreError> {
        Self::connect_with_max_connections(url, NonZeroU32::new(10).unwrap()).await
    }

    /// Connect and apply migrations with a limit that includes the publisher's advisory-lock session.
    pub async fn connect_with_max_connections(
        url: &str,
        max_connections: NonZeroU32,
    ) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections.get())
            .connect(url)
            .await
            .map_err(err)?;
        let mut tx = pool.begin().await.map_err(err)?;
        sqlx::query("SELECT pg_advisory_xact_lock(7462100000)")
            .execute(&mut *tx)
            .await
            .map_err(err)?;
        for (_, sql) in MIGRATIONS {
            // include_str! is &'static; iterating the slice yields &&str — deref for SqlSafeStr.
            sqlx::raw_sql(*sql).execute(&mut *tx).await.map_err(err)?;
        }
        tx.commit().await.map_err(err)?;
        Ok(PgStore { pool })
    }

    async fn new_version(
        tx: &mut Transaction<'_, Postgres>,
        connector: Option<&str>,
        kind: &str,
    ) -> Result<i64, StoreError> {
        sqlx::query_scalar("INSERT INTO seikika.input_versions (connector, kind) VALUES ($1, $2) RETURNING version")
            .bind(connector)
            .bind(kind)
            .fetch_one(&mut **tx)
            .await
            .map_err(err)
    }
}

#[async_trait::async_trait]
impl Store for PgStore {
    async fn ingest(&self, connector: &str, batch: &[Value]) -> Result<u64, StoreError> {
        let mut tx = self.pool.begin().await.map_err(err)?;
        let version = Self::new_version(&mut tx, Some(connector), "ingest").await?;
        let (mut records, mut deletes) = (0i32, 0i32);
        let mut command: Option<String> = None;
        for msg in batch {
            let s = |k: &str| msg.get(k).and_then(Value::as_str).unwrap_or_default();
            if command.is_none() {
                command = msg
                    .get("command_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
            }
            match s("type") {
                "record" => {
                    records += 1;
                    sqlx::query(
                        "INSERT INTO seikika.raw_records (connector, kind, external_id, data, observed_at, version)
                         VALUES ($1, $2, $3, $4, $5::timestamptz, $6)
                         ON CONFLICT (connector, kind, external_id)
                         DO UPDATE SET data = EXCLUDED.data, observed_at = EXCLUDED.observed_at, version = EXCLUDED.version",
                    )
                    .bind(connector)
                    .bind(s("kind"))
                    .bind(s("externalId"))
                    .bind(msg.get("data").cloned().unwrap_or(Value::Null))
                    .bind(msg.get("observed_at").and_then(Value::as_str))
                    .bind(version)
                    .execute(&mut *tx)
                    .await
                    .map_err(err)?;
                }
                "delete" => {
                    deletes += 1;
                    sqlx::query("DELETE FROM seikika.raw_records WHERE connector = $1 AND kind = $2 AND external_id = $3")
                        .bind(connector)
                        .bind(s("kind"))
                        .bind(s("externalId"))
                        .execute(&mut *tx)
                        .await
                        .map_err(err)?;
                }
                _ => {}
            }
        }
        sqlx::query("UPDATE seikika.input_versions SET records = $2, deletes = $3, command_id = $4 WHERE version = $1")
            .bind(version)
            .bind(records)
            .bind(deletes)
            .bind(command)
            .execute(&mut *tx)
            .await
            .map_err(err)?;
        tx.commit().await.map_err(err)?;
        Ok(version as u64)
    }

    async fn put_native(&self, values: Vec<NativeValue>) -> Result<u64, StoreError> {
        let mut tx = self.pool.begin().await.map_err(err)?;
        let version = Self::new_version(&mut tx, None, "native").await?;
        for v in values {
            let current: Option<i64> = sqlx::query_scalar(
                "SELECT max(revision) FROM seikika.native_values WHERE entity_id = $1 AND field = $2",
            )
            .bind(&v.entity)
            .bind(&v.field)
            .fetch_one(&mut *tx)
            .await
            .map_err(err)?;
            let next = current.unwrap_or(0) + 1;
            if v.revision != 0 && v.revision as i64 != next {
                return Err(StoreError::Conflict(format!(
                    "{}.{}: expected revision {next}, got {}",
                    v.entity, v.field, v.revision
                )));
            }
            sqlx::query(
                "INSERT INTO seikika.native_values (entity_id, field, revision, value, author, reason, version)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(&v.entity)
            .bind(&v.field)
            .bind(next)
            .bind(&v.value)
            .bind(&v.author)
            .bind(&v.reason)
            .bind(version)
            .execute(&mut *tx)
            .await
            .map_err(err)?;
        }
        tx.commit().await.map_err(err)?;
        Ok(version as u64)
    }

    async fn input_version(&self) -> Result<u64, StoreError> {
        let v: Option<i64> = sqlx::query_scalar("SELECT max(version) FROM seikika.input_versions")
            .fetch_one(&self.pool)
            .await
            .map_err(err)?;
        Ok(v.unwrap_or(0) as u64)
    }

    async fn snapshot(&self) -> Result<InputSnapshot, StoreError> {
        let mut tx = self.pool.begin().await.map_err(err)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(err)?;
        let version: Option<i64> =
            sqlx::query_scalar("SELECT max(version) FROM seikika.input_versions")
                .fetch_one(&mut *tx)
                .await
                .map_err(err)?;
        let rows = sqlx::query(
            "SELECT connector, kind, external_id, data FROM seikika.raw_records ORDER BY connector, kind, external_id",
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(err)?;
        let records = rows
            .into_iter()
            .map(|r| Record {
                connector: r.get(0),
                kind: r.get(1),
                external_id: r.get(2),
                data: r.get(3),
            })
            .collect();
        let rows = sqlx::query(
            "SELECT DISTINCT ON (entity_id, field) entity_id, field, value, revision, author, reason, at::text
             FROM seikika.native_values ORDER BY entity_id, field, revision DESC",
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(err)?;
        let native = rows
            .into_iter()
            .map(|r| NativeValue {
                entity: r.get(0),
                field: r.get(1),
                value: r.get(2),
                revision: r.get::<i64, _>(3) as u64,
                author: r.get(4),
                reason: r.get(5),
                at: r.get(6),
            })
            .collect();
        tx.commit().await.map_err(err)?;
        Ok(InputSnapshot {
            version: version.unwrap_or(0) as u64,
            records,
            native,
        })
    }
}

/// A tuple triple: (object, relation, subject).
pub type Tuple = (String, String, String);

/// One snapshot to record, with the tuple delta since the last one (`full` replaces all tuples).
#[derive(Debug, Clone, Copy)]
pub struct Publication<'a> {
    pub snapshot_id: &'a str,
    pub input_version: u64,
    pub manifest_hash: &'a str,
    pub summary: &'a Value,
    pub added: &'a [Tuple],
    pub removed: &'a [Tuple],
    pub full: bool,
}

/// Publishing derived results (only the instance holding the advisory lock calls these).
impl PgStore {
    /// Try to become the publisher. Holds a dedicated connection for the lock's lifetime.
    pub async fn try_lead(
        &self,
    ) -> Result<Option<sqlx::pool::PoolConnection<Postgres>>, StoreError> {
        let mut conn = self.pool.acquire().await.map_err(err)?;
        let got: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock(7462100001)")
            .fetch_one(&mut *conn)
            .await
            .map_err(err)?;
        Ok(got.then_some(conn))
    }

    /// Record a snapshot and apply a tuple delta in one transaction.
    pub async fn publish(&self, p: &Publication<'_>) -> Result<(), StoreError> {
        let Publication {
            snapshot_id,
            input_version,
            manifest_hash,
            summary,
            added,
            removed,
            full,
        } = *p;
        let mut tx = self.pool.begin().await.map_err(err)?;
        if full {
            sqlx::query("DELETE FROM seikika.derived_tuples")
                .execute(&mut *tx)
                .await
                .map_err(err)?;
        }
        for (o, r, s) in removed {
            sqlx::query("DELETE FROM seikika.derived_tuples WHERE object = $1 AND relation = $2 AND subject = $3")
                .bind(o)
                .bind(r)
                .bind(s)
                .execute(&mut *tx)
                .await
                .map_err(err)?;
        }
        for chunk in added.chunks(1000) {
            let (os, (rs, ss)): (Vec<&str>, (Vec<&str>, Vec<&str>)) = chunk
                .iter()
                .map(|(o, r, s)| (o.as_str(), (r.as_str(), s.as_str())))
                .unzip();
            sqlx::query(
                "INSERT INTO seikika.derived_tuples (object, relation, subject)
                 SELECT * FROM unnest($1::text[], $2::text[], $3::text[]) ON CONFLICT DO NOTHING",
            )
            .bind(os)
            .bind(rs)
            .bind(ss)
            .execute(&mut *tx)
            .await
            .map_err(err)?;
        }
        sqlx::query(
            "INSERT INTO seikika.snapshots (snapshot_id, input_version, manifest_hash, summary)
             VALUES ($1, $2, $3, $4) ON CONFLICT (snapshot_id) DO NOTHING",
        )
        .bind(snapshot_id)
        .bind(input_version as i64)
        .bind(manifest_hash)
        .bind(summary)
        .execute(&mut *tx)
        .await
        .map_err(err)?;
        tx.commit().await.map_err(err)?;
        Ok(())
    }

    /// Store a personal access token hash for `subject`.
    pub async fn create_token(
        &self,
        hash: &str,
        subject: &str,
        label: Option<&str>,
    ) -> Result<(), StoreError> {
        sqlx::query("INSERT INTO seikika.tokens (token_hash, subject, label) VALUES ($1, $2, $3)")
            .bind(hash)
            .bind(subject)
            .bind(label)
            .execute(&self.pool)
            .await
            .map_err(err)?;
        Ok(())
    }

    /// The subject of a live (unrevoked, unexpired) token hash.
    pub async fn token_subject(&self, hash: &str) -> Option<String> {
        sqlx::query_scalar(
            "SELECT subject FROM seikika.tokens WHERE token_hash = $1 AND revoked_at IS NULL
             AND (expires_at IS NULL OR expires_at > now())",
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
    }

    /// Store an action run's record (a JSON document), replacing the one with its id.
    pub async fn put_run(&self, id: &str, doc: &str) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO seikika.runs (id, doc) VALUES ($1, $2::jsonb)
             ON CONFLICT (id) DO UPDATE SET doc = EXCLUDED.doc, updated_at = now()",
        )
        .bind(id)
        .bind(doc)
        .execute(&self.pool)
        .await
        .map_err(err)?;
        Ok(())
    }

    /// Every stored run record.
    pub async fn runs(&self) -> Result<Vec<String>, StoreError> {
        sqlx::query_scalar("SELECT doc::text FROM seikika.runs ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .map_err(err)
    }

    /// Record a spent confirm token.
    pub async fn put_spent(&self, token: &str, issued: i64) -> Result<(), StoreError> {
        sqlx::query("INSERT INTO seikika.spent_tokens (token, issued) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(token)
            .bind(issued)
            .execute(&self.pool)
            .await
            .map_err(err)?;
        Ok(())
    }

    /// Confirm tokens spent since `since` (seconds since the epoch).
    pub async fn spent(&self, since: i64) -> Result<Vec<(String, i64)>, StoreError> {
        sqlx::query_as("SELECT token, issued FROM seikika.spent_tokens WHERE issued >= $1")
            .bind(since)
            .fetch_all(&self.pool)
            .await
            .map_err(err)
    }
}
