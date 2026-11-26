//! AWS Secrets Manager backend (GetSecretValue over HTTP; auth via injectable headers).

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::backends::{SecretBackend, mapped};
use crate::bytes::SecretBytes;
use crate::error::SecretError;
use crate::http::SharedHttp;

type AuthHeaders = Arc<dyn Fn() -> Result<Vec<(String, String)>, SecretError> + Send + Sync>;

/// Calls `secretsmanager.GetSecretValue`. Signing is delegated to `auth_headers` so CI can use
/// fakes/LocalStack without the AWS SDK; production sets headers from env session creds helpers.
pub struct AwsSecretsManager {
    region: String,
    endpoint: String,
    map: BTreeMap<String, String>,
    http: SharedHttp,
    auth_headers: AuthHeaders,
}

impl AwsSecretsManager {
    pub fn new(
        region: impl Into<String>,
        endpoint: Option<String>,
        map: BTreeMap<String, String>,
        http: SharedHttp,
        auth_headers: AuthHeaders,
    ) -> Self {
        let region = region.into();
        let endpoint =
            endpoint.unwrap_or_else(|| format!("https://secretsmanager.{region}.amazonaws.com"));
        Self {
            region,
            endpoint,
            map,
            http,
            auth_headers,
        }
    }

    /// Production: optional `SEIKIKA_AWS_SECRETS_BEARER` for proxies/LocalStack; otherwise empty
    /// signed-headers placeholder (operators terminate TLS auth at a sidecar, or set endpoint + bearer).
    pub fn from_config(
        region: Option<String>,
        endpoint: Option<String>,
        map: BTreeMap<String, String>,
        http: SharedHttp,
    ) -> Self {
        let region = region.unwrap_or_else(|| "us-east-1".into());
        Self::new(
            region,
            endpoint,
            map,
            http,
            Arc::new(|| {
                let mut h = Vec::new();
                if let Ok(t) = std::env::var("SEIKIKA_AWS_SECRETS_BEARER")
                    && !t.is_empty()
                {
                    h.push(("Authorization".into(), format!("Bearer {t}")));
                }
                Ok(h)
            }),
        )
    }

    fn secret_id(&self, logical: &str) -> String {
        let id = mapped(&self.map, logical);
        id.strip_prefix("aws-sm:").unwrap_or(id).to_owned()
    }
}

impl SecretBackend for AwsSecretsManager {
    fn name(&self) -> &'static str {
        "aws_secrets_manager"
    }

    fn get(&self, logical: &str) -> Result<Option<SecretBytes>, SecretError> {
        let secret_id = self.secret_id(logical);
        let body = serde_json::json!({ "SecretId": secret_id }).to_string();
        let mut headers = vec![
            (
                "Content-Type".to_owned(),
                "application/x-amz-json-1.1".to_owned(),
            ),
            (
                "X-Amz-Target".to_owned(),
                "secretsmanager.GetSecretValue".to_owned(),
            ),
            ("X-Amz-Region".to_owned(), self.region.clone()),
        ];
        headers.extend((self.auth_headers)()?);
        let header_refs: Vec<(&str, &str)> = headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let resp = self
            .http
            .post(self.endpoint.trim_end_matches('/'), &header_refs, &body)?;
        match resp.status {
            200 => {
                let v: serde_json::Value = serde_json::from_str(&resp.body)
                    .map_err(|e| SecretError::backend("aws", e.to_string()))?;
                if let Some(s) = v.get("SecretString").and_then(|x| x.as_str()) {
                    return Ok(Some(SecretBytes::new(s.to_owned())));
                }
                if let Some(b64) = v.get("SecretBinary").and_then(|x| x.as_str()) {
                    let bytes = crate::b64::decode(b64)?;
                    let text = String::from_utf8(bytes)
                        .map_err(|e| SecretError::backend("aws", e.to_string()))?;
                    return Ok(Some(SecretBytes::new(text)));
                }
                Err(SecretError::backend("aws", "no SecretString/SecretBinary"))
            }
            400 | 404 => {
                // ResourceNotFoundException is typically HTTP 400 with __type.
                if resp.body.contains("ResourceNotFoundException") || resp.status == 404 {
                    Ok(None)
                } else if resp.body.contains("AccessDenied") {
                    Err(SecretError::backend("aws", "access denied"))
                } else {
                    Err(SecretError::backend(
                        "aws",
                        format!(
                            "unexpected status {}: {}",
                            resp.status,
                            &resp.body[..resp.body.len().min(80)]
                        ),
                    ))
                }
            }
            403 => Err(SecretError::backend("aws", "access denied (403)")),
            other => Err(SecretError::backend(
                "aws",
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
    fn get_secret_value() {
        let mut posts = HashMap::new();
        posts.insert(
            "http://sm.local".into(),
            HttpResponse {
                status: 200,
                body: r#"{"SecretString":"aws-secret"}"#.into(),
            },
        );
        let http = Arc::new(FakeTransport {
            gets: HashMap::new(),
            posts,
        });
        let b = AwsSecretsManager::new(
            "us-east-1",
            Some("http://sm.local".into()),
            BTreeMap::new(),
            http,
            Arc::new(|| Ok(vec![])),
        );
        assert_eq!(
            b.get("github-app-private-key").unwrap().unwrap().expose(),
            "aws-secret"
        );
    }

    #[test]
    fn not_found() {
        let mut posts = HashMap::new();
        posts.insert(
            "http://sm.local".into(),
            HttpResponse {
                status: 400,
                body: r#"{"__type":"ResourceNotFoundException"}"#.into(),
            },
        );
        let http = Arc::new(FakeTransport {
            gets: HashMap::new(),
            posts,
        });
        let b = AwsSecretsManager::new(
            "us-east-1",
            Some("http://sm.local".into()),
            BTreeMap::new(),
            http,
            Arc::new(|| Ok(vec![])),
        );
        assert!(b.get("missing").unwrap().is_none());
    }
}
