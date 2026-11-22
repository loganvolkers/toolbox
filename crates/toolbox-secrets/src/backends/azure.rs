//! Azure Key Vault secrets backend.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::backends::{SecretBackend, mapped};
use crate::bytes::SecretBytes;
use crate::error::SecretError;
use crate::http::SharedHttp;

type TokenFn = Arc<dyn Fn() -> Result<Option<String>, SecretError> + Send + Sync>;

/// GET `{vault}/secrets/{name}?api-version=7.4`.
pub struct AzureKeyVault {
    vault_url: String,
    map: BTreeMap<String, String>,
    http: SharedHttp,
    token: TokenFn,
}

impl AzureKeyVault {
    pub fn new(
        vault_url: impl Into<String>,
        endpoint_override: Option<String>,
        map: BTreeMap<String, String>,
        http: SharedHttp,
        token: TokenFn,
    ) -> Self {
        Self {
            vault_url: endpoint_override
                .unwrap_or_else(|| vault_url.into())
                .trim_end_matches('/')
                .to_owned(),
            map,
            http,
            token,
        }
    }

    pub fn from_config(
        vault_url: impl Into<String>,
        endpoint: Option<String>,
        map: BTreeMap<String, String>,
        http: SharedHttp,
    ) -> Self {
        Self::new(
            vault_url,
            endpoint,
            map,
            http,
            Arc::new(|| {
                Ok(std::env::var("AZURE_KEY_VAULT_TOKEN")
                    .or_else(|_| std::env::var("TOOLBOX_AZURE_KEY_VAULT_TOKEN"))
                    .ok())
            }),
        )
    }

    fn secret_name(&self, logical: &str) -> String {
        let id = mapped(&self.map, logical);
        id.strip_prefix("azure-kv:")
            .unwrap_or(id)
            .trim_start_matches('/')
            .to_owned()
    }
}

impl SecretBackend for AzureKeyVault {
    fn name(&self) -> &'static str {
        "azure_key_vault"
    }

    fn get(&self, logical: &str) -> Result<Option<SecretBytes>, SecretError> {
        let Some(token) = (self.token)()? else {
            return Err(SecretError::backend(
                "azure",
                "no AZURE_KEY_VAULT_TOKEN / TOOLBOX_AZURE_KEY_VAULT_TOKEN",
            ));
        };
        let name = self.secret_name(logical);
        let url = format!(
            "{}/secrets/{}?api-version=7.4",
            self.vault_url, name
        );
        let auth = format!("Bearer {token}");
        let resp = self.http.get(&url, &[("Authorization", &auth)])?;
        match resp.status {
            200 => {
                let v: serde_json::Value = serde_json::from_str(&resp.body)
                    .map_err(|e| SecretError::backend("azure", e.to_string()))?;
                let text = v
                    .get("value")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| SecretError::backend("azure", "missing value"))?;
                Ok(Some(SecretBytes::new(text.to_owned())))
            }
            404 => Ok(None),
            401 | 403 => Err(SecretError::backend(
                "azure",
                format!("access denied ({})", resp.status),
            )),
            other => Err(SecretError::backend(
                "azure",
                format!("unexpected status {other}"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{FakeTransport, HttpResponse};
    use std::collections::HashMap;

    #[test]
    fn reads_secret_value() {
        let url = "https://kv.test/secrets/toolbox-jira-token?api-version=7.4";
        let mut gets = HashMap::new();
        gets.insert(
            url.into(),
            HttpResponse {
                status: 200,
                body: r#"{"value":"az-tok"}"#.into(),
            },
        );
        let http = Arc::new(FakeTransport {
            gets,
            posts: HashMap::new(),
        });
        let b = AzureKeyVault::new(
            "https://kv.test",
            None,
            BTreeMap::from([("jira-app-token".into(), "toolbox-jira-token".into())]),
            http,
            Arc::new(|| Ok(Some("aad".into()))),
        );
        assert_eq!(b.get("jira-app-token").unwrap().unwrap().expose(), "az-tok");
    }
}
