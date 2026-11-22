//! Chained secret resolver and connector-config placeholder expansion.

use std::sync::{Arc, OnceLock};

use serde_json::{Map, Value};

use crate::backends::{EnvBackend, FileBackend, SecretBackend};
use crate::bytes::SecretBytes;
use crate::config::{BackendConfig, SecretsConfig};
use crate::error::{SecretError, warn_unresolved};

#[cfg(feature = "remote")]
use crate::http::default_http;

/// Ordered list of backends; first `Some` wins.
pub struct SecretResolver {
    backends: Vec<Arc<dyn SecretBackend>>,
    refs: std::collections::BTreeMap<String, String>,
    required: bool,
}

impl Default for SecretResolver {
    fn default() -> Self {
        Self {
            backends: vec![Arc::new(EnvBackend::process())],
            refs: Default::default(),
            required: false,
        }
    }
}

impl SecretResolver {
    pub fn new(backends: Vec<Arc<dyn SecretBackend>>) -> Self {
        Self {
            backends,
            refs: Default::default(),
            required: false,
        }
    }

    pub fn with_refs(mut self, refs: std::collections::BTreeMap<String, String>) -> Self {
        self.refs = refs;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn from_env_and_config(cfg: &SecretsConfig) -> Result<Self, SecretError> {
        let mut backends: Vec<Arc<dyn SecretBackend>> = Vec::new();
        #[cfg(feature = "remote")]
        let http = default_http()?;

        for b in &cfg.backends {
            match b {
                BackendConfig::Env => backends.push(Arc::new(EnvBackend::process())),
                BackendConfig::File { dir } => {
                    if let Some(dir) = dir {
                        backends.push(Arc::new(FileBackend::new(dir.clone())));
                    } else if let Ok(d) = std::env::var("TOOLBOX_SECRETS_DIR") {
                        backends.push(Arc::new(FileBackend::new(d)));
                    }
                }
                #[cfg(feature = "vault")]
                BackendConfig::Vault {
                    address,
                    mount,
                    prefix,
                    map,
                } => {
                    let addr = address
                        .clone()
                        .or_else(|| std::env::var("TOOLBOX_VAULT_ADDR").ok())
                        .or_else(|| std::env::var("VAULT_ADDR").ok())
                        .ok_or_else(|| {
                            SecretError::Config("vault backend needs address / VAULT_ADDR".into())
                        })?;
                    backends.push(Arc::new(crate::backends::vault::VaultBackend::from_config(
                        addr,
                        mount.clone(),
                        prefix.clone(),
                        map.clone(),
                        http.clone(),
                    )));
                }
                #[cfg(feature = "aws")]
                BackendConfig::AwsSecretsManager {
                    region,
                    endpoint,
                    map,
                } => {
                    backends.push(Arc::new(
                        crate::backends::aws::AwsSecretsManager::from_config(
                            region.clone(),
                            endpoint.clone(),
                            map.clone(),
                            http.clone(),
                        ),
                    ));
                }
                #[cfg(feature = "gcp")]
                BackendConfig::GcpSecretManager {
                    project,
                    endpoint,
                    map,
                } => {
                    let project = project
                        .clone()
                        .or_else(|| std::env::var("TOOLBOX_GCP_SECRETS_PROJECT").ok())
                        .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok())
                        .ok_or_else(|| {
                            SecretError::Config(
                                "gcp_secret_manager needs project / TOOLBOX_GCP_SECRETS_PROJECT"
                                    .into(),
                            )
                        })?;
                    backends.push(Arc::new(
                        crate::backends::gcp::GcpSecretManager::from_config(
                            project,
                            endpoint.clone(),
                            map.clone(),
                            http.clone(),
                        ),
                    ));
                }
                #[cfg(feature = "azure")]
                BackendConfig::AzureKeyVault {
                    vault_url,
                    endpoint,
                    map,
                } => {
                    backends.push(Arc::new(crate::backends::azure::AzureKeyVault::from_config(
                        vault_url.clone(),
                        endpoint.clone(),
                        map.clone(),
                        http.clone(),
                    )));
                }
            }
        }

        if backends.is_empty() {
            backends.push(Arc::new(EnvBackend::process()));
        }

        Ok(Self {
            backends,
            refs: cfg.refs.clone(),
            required: cfg.required,
        })
    }

    pub fn get(&self, logical: &str) -> Result<Option<SecretBytes>, SecretError> {
        // Escape-hatch URI in refs or inline `scheme:…` logical names.
        let lookup = self
            .refs
            .get(logical)
            .map(String::as_str)
            .unwrap_or(logical);

        if let Some((scheme, rest)) = split_scheme(lookup) {
            return self.get_uri(scheme, rest);
        }

        let mut last_err: Option<SecretError> = None;
        for b in &self.backends {
            match b.get(logical) {
                Ok(Some(v)) => return Ok(Some(v)),
                Ok(None) => {}
                Err(e) => {
                    // Soft-skip backends that aren't authenticated yet; keep last hard error.
                    let msg = e.to_string();
                    if msg.contains("no access token")
                        || msg.contains("no VAULT_TOKEN")
                        || msg.contains("no AZURE_KEY_VAULT_TOKEN")
                    {
                        // Treat missing bootstrap creds as "not this backend".
                    } else {
                        last_err = Some(e);
                    }
                }
            }
        }
        if let Some(e) = last_err {
            return Err(e);
        }
        Ok(None)
    }

    fn get_uri(&self, scheme: &str, rest: &str) -> Result<Option<SecretBytes>, SecretError> {
        let logical = format!("{scheme}:{rest}");
        for b in &self.backends {
            let hit = match scheme {
                "gcp-sm" | "gcp" if b.name() == "gcp_secret_manager" => b.get(&logical)?,
                "aws-sm" | "aws" if b.name() == "aws_secrets_manager" => b.get(&logical)?,
                "vault" if b.name() == "vault" => b.get(&logical)?,
                "azure-kv" | "azure" if b.name() == "azure_key_vault" => b.get(&logical)?,
                "env" if b.name() == "env" => b.get(rest)?,
                "file" if b.name() == "file" => b.get(rest)?,
                _ => None,
            };
            if hit.is_some() {
                return Ok(hit);
            }
        }
        Ok(None)
    }
}

fn split_scheme(s: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = s.split_once(':')?;
    match scheme {
        "gcp-sm" | "gcp" | "aws-sm" | "aws" | "vault" | "azure-kv" | "azure" | "env" | "file" => {
            Some((scheme, rest))
        }
        _ => None,
    }
}

static GLOBAL: OnceLock<SecretResolver> = OnceLock::new();

/// Install a process-wide resolver (idempotent: first call wins).
pub fn set_global(resolver: SecretResolver) -> Result<(), SecretResolver> {
    GLOBAL.set(resolver)
}

/// Clear is not supported on OnceLock; tests should build a local resolver.
pub fn global() -> &'static SecretResolver {
    GLOBAL.get_or_init(|| {
        let cfg = SecretsConfig::from_env().unwrap_or_default();
        SecretResolver::from_env_and_config(&cfg).unwrap_or_default()
    })
}

/// Reset helper for tests that need a fresh env-derived resolver — builds a local one instead.
pub fn resolver_from_env() -> Result<SecretResolver, SecretError> {
    SecretResolver::from_env_and_config(&SecretsConfig::from_env()?)
}

/// Resolve `${env:…}` / `${secret:…}` throughout a JSON value (objects and arrays).
pub fn resolve_value(
    connector: &str,
    value: &Value,
    resolver: &SecretResolver,
) -> Result<Value, SecretError> {
    match value {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                match resolve_leaf(connector, k, v, resolver)? {
                    Some(resolved) => {
                        out.insert(k.clone(), resolved);
                    }
                    None => {
                        // omitted
                    }
                }
            }
            Ok(Value::Object(out))
        }
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for (i, v) in items.iter().enumerate() {
                let key = format!("[{i}]");
                if let Some(resolved) = resolve_leaf(connector, &key, v, resolver)? {
                    out.push(resolved);
                }
            }
            Ok(Value::Array(out))
        }
        other => Ok(other.clone()),
    }
}

fn resolve_leaf(
    connector: &str,
    key: &str,
    value: &Value,
    resolver: &SecretResolver,
) -> Result<Option<Value>, SecretError> {
    match value {
        Value::Object(_) | Value::Array(_) => Ok(Some(resolve_value(connector, value, resolver)?)),
        Value::String(s) if s.starts_with("${") && s.ends_with('}') => {
            let inner = &s[2..s.len() - 1];
            let resolved = if let Some(name) = inner.strip_prefix("env:") {
                std::env::var(name).ok().map(Value::String)
            } else if let Some(name) = inner.strip_prefix("secret:") {
                resolver
                    .get(name)?
                    .map(|secret| Value::String(secret.into_string()))
            } else {
                None
            };
            match resolved {
                Some(v) => Ok(Some(v)),
                None => {
                    if resolver.required() && inner.starts_with("secret:") {
                        return Err(SecretError::Required {
                            name: inner.trim_start_matches("secret:").to_owned(),
                        });
                    }
                    warn_unresolved(connector, s, key);
                    Ok(None)
                }
            }
        }
        other => Ok(Some(other.clone())),
    }
}

/// Resolve connector config and merge `TOOLBOX_CONNECTOR_CONFIG_OVERRIDES`.
pub fn resolve_config(connector: &str, cfg: &Value, resolver: &SecretResolver) -> Value {
    let mut base = match cfg {
        Value::Object(_) | Value::Null => cfg.clone(),
        other => {
            // Non-object configs are passed through unchanged historically as empty object when Null.
            let _ = other;
            Value::Object(Map::new())
        }
    };
    if base.is_null() {
        base = Value::Object(Map::new());
    }

    let mut out = match resolve_value(connector, &base, resolver) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("toolbox: connector {connector}: secrets resolve failed: {e}");
            Value::Object(Map::new())
        }
    };

    if let Ok(overrides) = std::env::var("TOOLBOX_CONNECTOR_CONFIG_OVERRIDES")
        && let Ok(Value::Object(all)) = serde_json::from_str::<Value>(&overrides)
        && let Some(Value::Object(mine)) = all.get(connector)
        && let Some(obj) = out.as_object_mut()
    {
        obj.extend(mine.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn env_before_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("my-secret"), "from-file").unwrap();
        let with_env = SecretResolver::new(vec![
            Arc::new(EnvBackend::from_map(BTreeMap::from([(
                "my-secret".into(),
                "from-env".into(),
            )]))),
            Arc::new(FileBackend::new(dir.path())),
        ]);
        assert_eq!(
            with_env.get("my-secret").unwrap().unwrap().expose(),
            "from-env"
        );
        let file_only = SecretResolver::new(vec![Arc::new(FileBackend::new(dir.path()))]);
        assert_eq!(
            file_only.get("my-secret").unwrap().unwrap().expose(),
            "from-file"
        );
    }

    #[test]
    fn nested_resolve_and_omit() {
        let r = SecretResolver::new(vec![Arc::new(EnvBackend::from_map(BTreeMap::from([(
            "nested-key".into(),
            "nested-val".into(),
        )])))]);
        let cfg = json!({
            "outer": {
                "token": "${secret:nested-key}",
                "missing": "${secret:missing}",
                "plain": "x"
            },
            "list": ["${secret:nested-key}", "y"]
        });
        let out = resolve_config("c", &cfg, &r);
        assert_eq!(out["outer"]["token"], "nested-val");
        assert!(out["outer"].get("missing").is_none());
        assert_eq!(out["outer"]["plain"], "x");
        assert_eq!(out["list"][0], "nested-val");
    }

    #[test]
    fn required_mode_errors() {
        let r = SecretResolver::new(vec![Arc::new(EnvBackend::process())]).with_required(true);
        let cfg = json!({"k": "${secret:does-not-exist-xyz}"});
        let err = resolve_value("c", &cfg, &r).unwrap_err();
        assert!(matches!(err, SecretError::Required { .. }));
    }
}
