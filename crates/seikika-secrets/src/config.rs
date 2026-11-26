//! Operator-facing secrets configuration (env + optional JSON/YAML file).

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::error::SecretError;

/// One configured remote (or local) backend.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackendConfig {
    Env,
    File {
        #[serde(default)]
        dir: Option<PathBuf>,
    },
    #[cfg(feature = "vault")]
    Vault {
        #[serde(default)]
        address: Option<String>,
        #[serde(default)]
        mount: Option<String>,
        #[serde(default)]
        prefix: Option<String>,
        #[serde(default)]
        map: BTreeMap<String, String>,
    },
    #[cfg(feature = "aws")]
    AwsSecretsManager {
        #[serde(default)]
        region: Option<String>,
        #[serde(default)]
        endpoint: Option<String>,
        #[serde(default)]
        map: BTreeMap<String, String>,
    },
    #[cfg(feature = "gcp")]
    GcpSecretManager {
        #[serde(default)]
        project: Option<String>,
        #[serde(default)]
        endpoint: Option<String>,
        #[serde(default)]
        map: BTreeMap<String, String>,
    },
    #[cfg(feature = "azure")]
    AzureKeyVault {
        vault_url: String,
        #[serde(default)]
        endpoint: Option<String>,
        #[serde(default)]
        map: BTreeMap<String, String>,
    },
}

/// Runtime secrets configuration. Loaded from `SEIKIKA_SECRETS_CONFIG` or built from env.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SecretsConfig {
    /// When true, missing `${secret:}` values fail instead of soft-omit.
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub backends: Vec<BackendConfig>,
    /// Logical name → provider URI (`gcp-sm:…`, `vault:…`, …) for escape-hatch routing.
    #[serde(default)]
    pub refs: BTreeMap<String, String>,
}

impl SecretsConfig {
    /// Build from process environment.
    ///
    /// - `SEIKIKA_SECRETS_CONFIG` — path to JSON/YAML, or inline JSON
    /// - `SEIKIKA_SECRETS_DIR` — file backend directory
    /// - `SEIKIKA_SECRETS_REQUIRED=1|true` — hard fail on missing secrets
    /// - `SEIKIKA_SECRETS_REFS` — JSON object of logical → URI
    /// - Cloud: `SEIKIKA_VAULT_ADDR`, `SEIKIKA_GCP_SECRETS_PROJECT`, `SEIKIKA_AWS_SECRETS_ENABLE`,
    ///   `SEIKIKA_AZURE_KEY_VAULT_URL`, plus optional `SEIKIKA_*_SECRETS_MAP` JSON maps
    pub fn from_env() -> Result<Self, SecretError> {
        let mut cfg = if let Ok(raw) = std::env::var("SEIKIKA_SECRETS_CONFIG") {
            Self::parse_blob(&raw)?
        } else {
            Self::default()
        };

        if std::env::var("SEIKIKA_SECRETS_REQUIRED")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            cfg.required = true;
        }

        if let Ok(dir) = std::env::var("SEIKIKA_SECRETS_DIR")
            && !dir.is_empty()
        {
            let path = PathBuf::from(dir);
            let has_file = cfg
                .backends
                .iter()
                .any(|b| matches!(b, BackendConfig::File { .. }));
            if !has_file {
                cfg.backends
                    .insert(0, BackendConfig::File { dir: Some(path) });
            } else {
                for b in &mut cfg.backends {
                    if let BackendConfig::File { dir } = b
                        && dir.is_none()
                    {
                        *dir = Some(path.clone());
                    }
                }
            }
        }

        if let Ok(refs) = std::env::var("SEIKIKA_SECRETS_REFS") {
            let parsed: BTreeMap<String, String> = serde_json::from_str(&refs)
                .map_err(|e| SecretError::Config(format!("SEIKIKA_SECRETS_REFS: {e}")))?;
            cfg.refs.extend(parsed);
        }

        if cfg.backends.is_empty() {
            cfg.backends.push(BackendConfig::Env);
            cfg.backends.push(BackendConfig::File {
                dir: std::env::var_os("SEIKIKA_SECRETS_DIR").map(PathBuf::from),
            });
            Self::push_cloud_from_env(&mut cfg)?;
        } else if !cfg.backends.iter().any(|b| matches!(b, BackendConfig::Env)) {
            // Keep D55 env fallback even when operators list only remotes.
            cfg.backends.insert(0, BackendConfig::Env);
        }

        Ok(cfg)
    }

    fn parse_blob(raw: &str) -> Result<Self, SecretError> {
        let trimmed = raw.trim();
        if trimmed.starts_with('{') {
            return serde_json::from_str(trimmed)
                .map_err(|e| SecretError::Config(format!("SEIKIKA_SECRETS_CONFIG JSON: {e}")));
        }
        let path = PathBuf::from(trimmed);
        let text = std::fs::read_to_string(&path).map_err(|e| {
            SecretError::Config(format!(
                "SEIKIKA_SECRETS_CONFIG file {}: {e}",
                path.display()
            ))
        })?;
        if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            serde_norway::from_str(&text).map_err(|e| {
                SecretError::Config(format!(
                    "SEIKIKA_SECRETS_CONFIG YAML {}: {e}",
                    path.display()
                ))
            })
        } else {
            serde_json::from_str(&text)
                .map_err(|e| SecretError::Config(format!("SEIKIKA_SECRETS_CONFIG: {e}")))
        }
    }

    fn push_cloud_from_env(cfg: &mut Self) -> Result<(), SecretError> {
        #[cfg(feature = "vault")]
        if std::env::var_os("SEIKIKA_VAULT_ADDR").is_some()
            || std::env::var_os("VAULT_ADDR").is_some()
        {
            cfg.backends.push(BackendConfig::Vault {
                address: std::env::var("SEIKIKA_VAULT_ADDR")
                    .or_else(|_| std::env::var("VAULT_ADDR"))
                    .ok(),
                mount: std::env::var("SEIKIKA_VAULT_MOUNT").ok(),
                prefix: std::env::var("SEIKIKA_VAULT_PREFIX").ok(),
                map: load_map_env("SEIKIKA_VAULT_SECRETS_MAP")?,
            });
        }
        #[cfg(feature = "gcp")]
        if let Ok(project) = std::env::var("SEIKIKA_GCP_SECRETS_PROJECT")
            .or_else(|_| std::env::var("GOOGLE_CLOUD_PROJECT"))
            && !project.is_empty()
        {
            cfg.backends.push(BackendConfig::GcpSecretManager {
                project: Some(project),
                endpoint: std::env::var("SEIKIKA_GCP_SECRETS_ENDPOINT").ok(),
                map: load_map_env("SEIKIKA_GCP_SECRETS_MAP")?,
            });
        }
        #[cfg(feature = "aws")]
        if std::env::var_os("SEIKIKA_AWS_SECRETS_ENABLE").is_some()
            || std::env::var_os("SEIKIKA_AWS_SECRETS_ENDPOINT").is_some()
        {
            cfg.backends.push(BackendConfig::AwsSecretsManager {
                region: std::env::var("SEIKIKA_AWS_REGION")
                    .or_else(|_| std::env::var("AWS_REGION"))
                    .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
                    .ok(),
                endpoint: std::env::var("SEIKIKA_AWS_SECRETS_ENDPOINT").ok(),
                map: load_map_env("SEIKIKA_AWS_SECRETS_MAP")?,
            });
        }
        #[cfg(feature = "azure")]
        if let Ok(url) = std::env::var("SEIKIKA_AZURE_KEY_VAULT_URL")
            && !url.is_empty()
        {
            cfg.backends.push(BackendConfig::AzureKeyVault {
                vault_url: url,
                endpoint: std::env::var("SEIKIKA_AZURE_KEY_VAULT_ENDPOINT").ok(),
                map: load_map_env("SEIKIKA_AZURE_SECRETS_MAP")?,
            });
        }
        let _ = cfg;
        Ok(())
    }
}

fn load_map_env(name: &str) -> Result<BTreeMap<String, String>, SecretError> {
    match std::env::var(name) {
        Ok(raw) if !raw.is_empty() => {
            serde_json::from_str(&raw).map_err(|e| SecretError::Config(format!("{name}: {e}")))
        }
        _ => Ok(BTreeMap::new()),
    }
}

/// Env var name for a logical secret (`github-app-private-key` → `SEIKIKA_SECRET_GITHUB_APP_PRIVATE_KEY`).
pub fn env_var_for(logical: &str) -> String {
    format!(
        "SEIKIKA_SECRET_{}",
        logical.to_uppercase().replace('-', "_")
    )
}
