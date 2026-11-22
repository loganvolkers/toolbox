//! Google Cloud Secret Manager backend (REST; ADC/WIF token via env).

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::backends::{SecretBackend, mapped};
use crate::bytes::SecretBytes;
use crate::error::SecretError;
use crate::http::SharedHttp;

const DEFAULT_API: &str = "https://secretmanager.googleapis.com";

type TokenFn = Arc<dyn Fn() -> Result<Option<String>, SecretError> + Send + Sync>;

/// Reads `projects/{project}/secrets/{id}/versions/latest`.
pub struct GcpSecretManager {
    project: String,
    endpoint: String,
    map: BTreeMap<String, String>,
    http: SharedHttp,
    token: TokenFn,
}

impl GcpSecretManager {
    pub fn new(
        project: impl Into<String>,
        endpoint: Option<String>,
        map: BTreeMap<String, String>,
        http: SharedHttp,
        token: TokenFn,
    ) -> Self {
        Self {
            project: project.into(),
            endpoint: endpoint.unwrap_or_else(|| DEFAULT_API.to_owned()),
            map,
            http,
            token,
        }
    }

    /// Production constructor: bearer from `GOOGLE_ACCESS_TOKEN` / `CLOUDSDK_AUTH_ACCESS_TOKEN`,
    /// else GCP metadata server (workload identity).
    pub fn from_config(
        project: impl Into<String>,
        endpoint: Option<String>,
        map: BTreeMap<String, String>,
        http: SharedHttp,
    ) -> Self {
        let http_for_meta = http.clone();
        Self::new(
            project,
            endpoint,
            map,
            http,
            Arc::new(move || access_token(&http_for_meta)),
        )
    }

    fn resource_path(&self, logical: &str) -> Result<String, SecretError> {
        let id = mapped(&self.map, logical);
        if let Some(rest) = id.strip_prefix("gcp-sm:") {
            return Ok(normalize_resource(rest));
        }
        if id.starts_with("projects/") {
            return Ok(normalize_resource(id));
        }
        Ok(format!(
            "projects/{}/secrets/{}/versions/latest",
            self.project, id
        ))
    }
}

fn normalize_resource(s: &str) -> String {
    if s.contains("/versions/") {
        s.trim_start_matches('/').to_owned()
    } else {
        format!("{}/versions/latest", s.trim_end_matches('/'))
    }
}

fn access_token(http: &SharedHttp) -> Result<Option<String>, SecretError> {
    if let Ok(t) = std::env::var("GOOGLE_ACCESS_TOKEN").or_else(|_| std::env::var("CLOUDSDK_AUTH_ACCESS_TOKEN"))
        && !t.is_empty()
    {
        return Ok(Some(t));
    }
    // Workload identity / GCE metadata.
    let meta = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";
    match http.get(meta, &[("Metadata-Flavor", "Google")]) {
        Ok(resp) if resp.status == 200 => {
            let v: serde_json::Value = serde_json::from_str(&resp.body)
                .map_err(|e| SecretError::backend("gcp", e.to_string()))?;
            Ok(v.get("access_token")
                .and_then(|x| x.as_str())
                .map(str::to_owned))
        }
        Ok(_) | Err(_) => Ok(None),
    }
}

impl SecretBackend for GcpSecretManager {
    fn name(&self) -> &'static str {
        "gcp_secret_manager"
    }

    fn get(&self, logical: &str) -> Result<Option<SecretBytes>, SecretError> {
        let Some(token) = (self.token)()? else {
            return Err(SecretError::backend(
                "gcp",
                "no access token (set GOOGLE_ACCESS_TOKEN or use workload identity)",
            ));
        };
        let resource = self.resource_path(logical)?;
        let url = format!(
            "{}/v1/{}:access",
            self.endpoint.trim_end_matches('/'),
            resource
        );
        let auth = format!("Bearer {token}");
        let resp = self.http.get(&url, &[("Authorization", &auth)])?;
        match resp.status {
            200 => {
                let v: serde_json::Value = serde_json::from_str(&resp.body)
                    .map_err(|e| SecretError::backend("gcp", e.to_string()))?;
                let b64 = v
                    .pointer("/payload/data")
                    .and_then(|x| x.as_str())
                    .ok_or_else(|| SecretError::backend("gcp", "missing payload.data"))?;
                let bytes = crate::b64::decode(b64)?;
                let text = String::from_utf8(bytes)
                    .map_err(|e| SecretError::backend("gcp", format!("secret is not utf-8: {e}")))?;
                Ok(Some(SecretBytes::new(text)))
            }
            404 => Ok(None),
            401 | 403 => Err(SecretError::backend(
                "gcp",
                format!("access denied ({})", resp.status),
            )),
            other => Err(SecretError::backend(
                "gcp",
                format!("unexpected status {other}: {}", truncate(&resp.body)),
            )),
        }
    }
}

fn truncate(s: &str) -> String {
    let t: String = s.chars().take(120).collect();
    if s.chars().count() > 120 {
        format!("{t}…")
    } else {
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{FakeTransport, HttpResponse};
    use std::collections::HashMap;

    #[test]
    fn happy_path_and_404() {
        let resource = "projects/p/secrets/toolbox-jira-token/versions/latest";
        let url = format!("https://sm.test/v1/{resource}:access");
        let mut gets = HashMap::new();
        // "tok" in base64
        gets.insert(
            url.clone(),
            HttpResponse {
                status: 200,
                body: r#"{"payload":{"data":"dG9r"}}"#.into(),
            },
        );
        let missing = "https://sm.test/v1/projects/p/secrets/missing/versions/latest:access";
        gets.insert(
            missing.to_string(),
            HttpResponse {
                status: 404,
                body: "{}".into(),
            },
        );
        let http: SharedHttp = Arc::new(FakeTransport {
            gets,
            posts: HashMap::new(),
        });
        let b = GcpSecretManager::new(
            "p",
            Some("https://sm.test".into()),
            BTreeMap::from([("jira-app-token".into(), "toolbox-jira-token".into())]),
            http.clone(),
            Arc::new(|| Ok(Some("ya29.test".into()))),
        );
        assert_eq!(b.get("jira-app-token").unwrap().unwrap().expose(), "tok");
        assert!(b.get("missing").unwrap().is_none());
    }

    #[test]
    fn forbidden_is_error() {
        let url =
            "https://sm.test/v1/projects/p/secrets/toolbox-github-app-pem/versions/latest:access";
        let mut gets = HashMap::new();
        gets.insert(
            url.into(),
            HttpResponse {
                status: 403,
                body: "denied".into(),
            },
        );
        let http: SharedHttp = Arc::new(FakeTransport {
            gets,
            posts: HashMap::new(),
        });
        let b = GcpSecretManager::new(
            "p",
            Some("https://sm.test".into()),
            BTreeMap::from([("github-app-private-key".into(), "toolbox-github-app-pem".into())]),
            http,
            Arc::new(|| Ok(Some("ya29.test".into()))),
        );
        assert!(b.get("github-app-private-key").is_err());
    }
}
