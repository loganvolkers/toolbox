//! HashiCorp Vault KV v2 backend.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::backends::{SecretBackend, mapped};
use crate::bytes::SecretBytes;
use crate::error::SecretError;
use crate::http::SharedHttp;

type TokenFn = Arc<dyn Fn() -> Result<Option<String>, SecretError> + Send + Sync>;

/// Reads KV v2 secrets. Default path: `{mount}/data/{prefix}/{logical}`.
pub struct VaultBackend {
    address: String,
    mount: String,
    prefix: String,
    map: BTreeMap<String, String>,
    http: SharedHttp,
    token: TokenFn,
}

impl VaultBackend {
    pub fn new(
        address: impl Into<String>,
        mount: Option<String>,
        prefix: Option<String>,
        map: BTreeMap<String, String>,
        http: SharedHttp,
        token: TokenFn,
    ) -> Self {
        Self {
            address: address.into().trim_end_matches('/').to_owned(),
            mount: mount.unwrap_or_else(|| "secret".into()),
            prefix: prefix.unwrap_or_else(|| "seikika".into()),
            map,
            http,
            token,
        }
    }

    pub fn from_config(
        address: impl Into<String>,
        mount: Option<String>,
        prefix: Option<String>,
        map: BTreeMap<String, String>,
        http: SharedHttp,
    ) -> Self {
        Self::new(
            address,
            mount,
            prefix,
            map,
            http,
            Arc::new(|| {
                Ok(std::env::var("VAULT_TOKEN")
                    .or_else(|_| std::env::var("SEIKIKA_VAULT_TOKEN"))
                    .ok())
            }),
        )
    }

    fn path_for(&self, logical: &str) -> String {
        let id = mapped(&self.map, logical);
        if let Some(rest) = id.strip_prefix("vault:") {
            return rest.trim_start_matches('/').to_owned();
        }
        if id.contains('/') {
            return id.to_owned();
        }
        format!("{}/{id}", self.prefix.trim_matches('/'))
    }
}

impl SecretBackend for VaultBackend {
    fn name(&self) -> &'static str {
        "vault"
    }

    fn get(&self, logical: &str) -> Result<Option<SecretBytes>, SecretError> {
        let Some(token) = (self.token)()? else {
            return Err(SecretError::backend(
                "vault",
                "no VAULT_TOKEN / SEIKIKA_VAULT_TOKEN",
            ));
        };
        let path = self.path_for(logical);
        let url = format!("{}/v1/{}/data/{path}", self.address, self.mount);
        let resp = self.http.get(&url, &[("X-Vault-Token", &token)])?;
        match resp.status {
            200 => {
                let v: serde_json::Value = serde_json::from_str(&resp.body)
                    .map_err(|e| SecretError::backend("vault", e.to_string()))?;
                // Prefer data.data.value, then data.data[logical], then single-string field.
                let data = v
                    .pointer("/data/data")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let text = data
                    .get("value")
                    .and_then(|x| x.as_str())
                    .or_else(|| data.get(logical).and_then(|x| x.as_str()))
                    .or_else(|| {
                        data.as_object().and_then(|o| {
                            if o.len() == 1 {
                                o.values().next().and_then(|x| x.as_str())
                            } else {
                                None
                            }
                        })
                    })
                    .ok_or_else(|| SecretError::backend("vault", "no string value in KV data"))?;
                Ok(Some(SecretBytes::new(text.to_owned())))
            }
            404 => Ok(None),
            403 => Err(SecretError::backend("vault", "access denied (403)")),
            other => Err(SecretError::backend(
                "vault",
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
    fn kv_v2_value_field() {
        let url = "http://vault.test/v1/secret/data/seikika/jira-app-token";
        let mut gets = HashMap::new();
        gets.insert(
            url.into(),
            HttpResponse {
                status: 200,
                body: r#"{"data":{"data":{"value":"jira-tok"}}}"#.into(),
            },
        );
        let http = Arc::new(FakeTransport {
            gets,
            posts: HashMap::new(),
        });
        let b = VaultBackend::new(
            "http://vault.test",
            None,
            None,
            BTreeMap::new(),
            http,
            Arc::new(|| Ok(Some("s.token".into()))),
        );
        assert_eq!(
            b.get("jira-app-token").unwrap().unwrap().expose(),
            "jira-tok"
        );
    }
}
