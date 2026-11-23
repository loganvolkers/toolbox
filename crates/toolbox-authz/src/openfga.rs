//! OpenFGA inside the Toolbox container (D38, Q11 default): the unmodified upstream server run
//! as a supervised co-process on localhost, plus a small HTTP client. With Postgres, OpenFGA's
//! `tuple` table is replaced by a read-only view over `toolbox.derived_tuples`, so tuples are
//! never written to OpenFGA (D26). With the memory datastore (tests, `toolbox dev`), tuple
//! deltas are written through its API instead.

use serde_json::{Value, json};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};

pub type Tuple = (String, String, String); // (user, relation, object)

#[derive(Debug, Clone)]
pub enum Datastore {
    Memory,
    /// A Postgres URI OpenFGA can use (its own tables live in the `public` schema).
    Postgres(String),
}

/// A running OpenFGA with one store and one model.
pub struct OpenFga {
    http: reqwest::Client,
    pub base: String,
    pub store: String,
    pub model_id: String,
    pub model: Value,
    pub datastore: Datastore,
    _child: Option<Child>,
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .unwrap_or(18081)
}

/// A store name OpenFGA accepts (3–64 of letters, digits and `-`).
pub fn store_name(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .take(64)
        .collect();
    while s.len() < 3 {
        s.push('-');
    }
    s
}

/// The OpenFGA binary: `TOOLBOX_OPENFGA_BIN`, else `openfga` on PATH.
pub fn binary() -> String {
    std::env::var("TOOLBOX_OPENFGA_BIN").unwrap_or_else(|_| "openfga".into())
}

/// The answers of one batch-check response, in request order. A per-check error (e.g. a datastore
/// timeout) is not a denial: it fails the batch, so the API answers 502 instead of pretending the
/// entity doesn't exist. An absent `allowed` with no error is `false` (proto3 omits default values).
fn batch_results(r: &Value, chunk: &[Tuple]) -> Result<Vec<bool>, String> {
    (0..chunk.len())
        .map(|i| {
            let item = &r["result"][format!("c{i}")];
            match (&item["error"], item["allowed"].as_bool()) {
                (e, _) if !e.is_null() => Err(format!("check {:?} failed: {e}", chunk[i])),
                (_, a) if item.is_object() => Ok(a.unwrap_or(false)),
                _ => Err(format!("check {:?}: no result in the response", chunk[i])),
            }
        })
        .collect()
}

impl OpenFga {
    /// Start OpenFGA, create (or reuse) the store `store_name` and write `model` (JSON).
    pub async fn start(
        datastore: Datastore,
        store_name: &str,
        model: Value,
    ) -> Result<OpenFga, String> {
        let bin = binary();
        let mut args: Vec<String> = Vec::new();
        if let Datastore::Postgres(uri) = &datastore {
            let out = Command::new(&bin)
                .args([
                    "migrate",
                    "--datastore-engine",
                    "postgres",
                    "--datastore-uri",
                    uri,
                ])
                .output()
                .await
                .map_err(|e| format!("cannot run {bin}: {e}"))?;
            if !out.status.success() {
                return Err(format!(
                    "openfga migrate failed: {}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
            args.extend([
                "--datastore-engine".into(),
                "postgres".into(),
                "--datastore-uri".into(),
                uri.clone(),
            ]);
        } else {
            args.extend(["--datastore-engine".into(), "memory".into()]);
        }
        let (http_port, grpc_port) = (free_port(), free_port());
        args.extend([
            "--http-addr".into(),
            format!("127.0.0.1:{http_port}"),
            "--grpc-addr".into(),
            format!("127.0.0.1:{grpc_port}"),
            "--playground-enabled=false".into(),
            "--metrics-enabled=false".into(),
            "--log-level".into(),
            "warn".into(),
        ]);
        let child = Command::new(&bin)
            .arg("run")
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("cannot start {bin}: {e}"))?;
        let base = format!("http://127.0.0.1:{http_port}");
        let mut fga = OpenFga {
            http: reqwest::Client::new(),
            base,
            store: String::new(),
            model_id: String::new(),
            model: model.clone(),
            datastore,
            _child: Some(child),
        };
        fga.wait_healthy(Duration::from_secs(20)).await?;
        fga.store = fga
            .find_or_create_store(&self::store_name(store_name))
            .await?;
        fga.model_id = fga.write_model(&model).await?;
        Ok(fga)
    }

    async fn wait_healthy(&self, timeout: Duration) -> Result<(), String> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Ok(r) = self.http.get(format!("{}/healthz", self.base)).send().await
                && r.status().is_success()
            {
                return Ok(());
            }
            if tokio::time::Instant::now() > deadline {
                return Err("OpenFGA did not become healthy".into());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn call(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, String> {
        let mut req = self.http.request(method, format!("{}{path}", self.base));
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("OpenFGA {path}: {e}"))?;
        let status = resp.status();
        let v: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(format!("OpenFGA {path}: {status} {v}"));
        }
        Ok(v)
    }

    async fn find_or_create_store(&self, name: &str) -> Result<String, String> {
        let list = self
            .call(reqwest::Method::GET, "/stores?page_size=100", None)
            .await?;
        if let Some(id) = list["stores"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|s| s["name"] == name)
            .and_then(|s| s["id"].as_str())
        {
            return Ok(id.to_owned());
        }
        let created = self
            .call(
                reqwest::Method::POST,
                "/stores",
                Some(json!({"name": name})),
            )
            .await?;
        created["id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "OpenFGA: store id missing".into())
    }

    /// Switch to a new, empty store (with the same model), e.g. one per test.
    pub async fn fresh_store(&mut self, name: &str) -> Result<(), String> {
        let created = self
            .call(
                reqwest::Method::POST,
                "/stores",
                Some(json!({"name": store_name(name)})),
            )
            .await?;
        self.store = created["id"]
            .as_str()
            .map(str::to_owned)
            .ok_or("OpenFGA: store id missing")?;
        self.model_id = self.write_model(&self.model.clone()).await?;
        Ok(())
    }

    pub async fn write_model(&self, model: &Value) -> Result<String, String> {
        let r = self
            .call(
                reqwest::Method::POST,
                &format!("/stores/{}/authorization-models", self.store),
                Some(model.clone()),
            )
            .await?;
        r["authorization_model_id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "OpenFGA: model id missing".into())
    }

    /// Memory datastore only: apply a tuple delta through the write API (100 per request).
    pub async fn write_tuples(&self, added: &[Tuple], removed: &[Tuple]) -> Result<(), String> {
        let key = |(u, r, o): &Tuple| json!({"user": u, "relation": r, "object": o});
        for chunk in removed.chunks(100) {
            let body = json!({"deletes": {"tuple_keys": chunk.iter().map(key).collect::<Vec<_>>()}, "authorization_model_id": self.model_id});
            self.call(
                reqwest::Method::POST,
                &format!("/stores/{}/write", self.store),
                Some(body),
            )
            .await?;
        }
        for chunk in added.chunks(100) {
            let body = json!({"writes": {"tuple_keys": chunk.iter().map(key).collect::<Vec<_>>()}, "authorization_model_id": self.model_id});
            self.call(
                reqwest::Method::POST,
                &format!("/stores/{}/write", self.store),
                Some(body),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn check(&self, user: &str, relation: &str, object: &str) -> Result<bool, String> {
        let body = json!({"tuple_key": {"user": user, "relation": relation, "object": object}, "authorization_model_id": self.model_id});
        let r = self
            .call(
                reqwest::Method::POST,
                &format!("/stores/{}/check", self.store),
                Some(body),
            )
            .await?;
        Ok(r["allowed"].as_bool().unwrap_or(false))
    }

    /// Many checks, 50 per request (OpenFGA's default batch limit).
    pub async fn batch_check(&self, checks: &[Tuple]) -> Result<Vec<bool>, String> {
        let mut out = Vec::with_capacity(checks.len());
        for chunk in checks.chunks(50) {
            let items: Vec<Value> = chunk
                .iter()
                .enumerate()
                .map(|(i, (u, r, o))| json!({"tuple_key": {"user": u, "relation": r, "object": o}, "correlation_id": format!("c{i}")}))
                .collect();
            let body = json!({"checks": items, "authorization_model_id": self.model_id});
            let r = self
                .call(
                    reqwest::Method::POST,
                    &format!("/stores/{}/batch-check", self.store),
                    Some(body),
                )
                .await?;
            out.extend(batch_results(&r, chunk)?);
        }
        Ok(out)
    }

    pub async fn list_objects(
        &self,
        user: &str,
        relation: &str,
        object_type: &str,
    ) -> Result<Vec<String>, String> {
        let body = json!({"type": object_type, "relation": relation, "user": user, "authorization_model_id": self.model_id});
        let r = self
            .call(
                reqwest::Method::POST,
                &format!("/stores/{}/list-objects", self.store),
                Some(body),
            )
            .await?;
        let mut objs: Vec<String> = r["objects"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|o| o.as_str().map(str::to_owned))
            .collect();
        objs.sort();
        Ok(objs)
    }
}

/// Replace OpenFGA's `tuple` table with a read-only view over `toolbox.derived_tuples` for `store`.
/// Idempotent: safe on every start.
///
/// The relation replaced is the `tuple` OpenFGA actually reads: the first base table of that name
/// on this connection's `search_path`, else the view already installed there. A role named
/// `toolbox` resolves unqualified names into the `toolbox` schema (Postgres puts `$user` ahead of
/// `public`), and that is where `openfga migrate` creates its tables. Replacing only `public.tuple`
/// leaves those checks reading an empty table, so every `can_view` denies.
///
/// Every column must hold what OpenFGA's own writes would. `user_type` in particular follows
/// `tuple.GetUserTypeFromUser` (OpenFGA v1.22.1, `pkg/tuple/tuple.go`): a userset (`type:id#rel`) or
/// a wildcard (`*`, `type:*`) is `userset`, anything else `user`. Some of OpenFGA's reads filter on
/// it and others don't, so a wrong value denies intermittently, whenever the check takes a path that
/// filters (docs/research-log.md, *OpenFGA wrong denials*).
pub async fn install_tuple_view(pool: &sqlx::PgPool, store: &str) -> Result<(), String> {
    if !store.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(format!("unexpected OpenFGA store id {store}"));
    }
    let found: Option<String> = sqlx::query_scalar(
        r#"SELECT n.nspname::text
           FROM pg_class c
           JOIN pg_namespace n ON n.oid = c.relnamespace
           WHERE c.relname = 'tuple'
             AND c.relkind IN ('r', 'v')
             AND n.nspname = ANY (current_schemas(false))
           ORDER BY CASE c.relkind WHEN 'r' THEN 0 ELSE 1 END,
                    array_position(current_schemas(false), n.nspname)
           LIMIT 1"#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("finding OpenFGA's tuple relation: {e}"))?;
    let schema = found.unwrap_or_else(|| "public".to_owned());
    if !schema
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(format!(
            "unexpected schema for OpenFGA's tuple relation: {schema}"
        ));
    }
    let sql = format!(
        r#"
DO $$ BEGIN
  IF EXISTS (
    SELECT 1 FROM pg_class c
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname = '{schema}' AND c.relname = 'tuple' AND c.relkind = 'r'
  ) THEN
    DROP TABLE IF EXISTS {schema}.tuple_openfga_unused;
    ALTER TABLE {schema}.tuple RENAME TO tuple_openfga_unused;
  END IF;
END $$;
CREATE OR REPLACE VIEW {schema}.tuple AS
  SELECT '{store}'::text AS store,
         split_part(object, ':', 1) AS object_type,
         substr(object, strpos(object, ':') + 1) AS object_id,
         relation,
         subject AS _user,
         CASE WHEN subject LIKE '%#%' OR subject = '*' OR subject ~ '^[^:]+:\*$'
              THEN 'userset' ELSE 'user' END AS user_type,
         NULL::text AS condition_name,
         NULL::bytea AS condition_context,
         md5(object || '|' || relation || '|' || subject) AS ulid,
         now() AS inserted_at
  FROM toolbox.derived_tuples;
-- OpenFGA filters on the view's computed columns; these let Postgres use an index for them
-- instead of scanning every tuple on every read of a check.
CREATE INDEX IF NOT EXISTS derived_tuples_by_object ON toolbox.derived_tuples
  (split_part(object, ':', 1), substr(object, strpos(object, ':') + 1), relation);
CREATE INDEX IF NOT EXISTS derived_tuples_by_user ON toolbox.derived_tuples
  (split_part(object, ':', 1), relation, subject);
"#
    );
    // store was validated as ASCII alphanumeric above; AssertSqlSafe is sqlx 0.9's audited bypass.
    sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
        .execute(pool)
        .await
        .map_err(|e| format!("installing the tuple view: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_errors_are_not_denials() {
        let t = |o: &str| ("user:alice".to_owned(), "viewer".to_owned(), o.to_owned());
        let chunk = [t("repository:a"), t("repository:b")];
        let ok = json!({"result": {"c0": {"allowed": true}, "c1": {}}});
        assert_eq!(batch_results(&ok, &chunk).unwrap(), vec![true, false]);
        let err =
            json!({"result": {"c0": {"allowed": true}, "c1": {"error": {"message": "timeout"}}}});
        assert!(batch_results(&err, &chunk).unwrap_err().contains("timeout"));
        let missing = json!({"result": {"c0": {"allowed": true}}});
        assert!(batch_results(&missing, &chunk).is_err());
    }
}
