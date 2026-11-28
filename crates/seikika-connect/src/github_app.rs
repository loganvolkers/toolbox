//! GitHub App installation authentication for connectors.

use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// Optional GitHub App credentials. A configured installation requires all three fields.
#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Installation {
    app_id: Option<String>,
    private_key: Option<String>,
    installation_id: Option<String>,
}

impl Installation {
    /// Construct credentials read from a flat connector config.
    pub fn new(app_id: String, private_key: String, installation_id: String) -> Self {
        Self {
            app_id: Some(app_id),
            private_key: Some(private_key),
            installation_id: Some(installation_id),
        }
    }

    /// Mint an installation access token, or return `None` when no app is configured.
    pub async fn token(
        &self,
        http: &reqwest::Client,
        api: &str,
        version: &str,
    ) -> Result<Option<String>, String> {
        let (app_id, pem, installation) = match (
            self.app_id.as_deref(),
            self.private_key.as_deref(),
            self.installation_id.as_deref(),
        ) {
            (None, None, None) => return Ok(None),
            (Some(app_id), Some(pem), Some(installation))
                if !app_id.trim().is_empty()
                    && !pem.trim().is_empty()
                    && !installation.is_empty()
                    && installation.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                (app_id.trim(), pem, installation)
            }
            _ => return Err(
                "GitHub App authentication needs appId, privateKey and a numeric installationId"
                    .into(),
            ),
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let claims = json!({"iat": now.saturating_sub(60), "exp": now + 540, "iss": app_id});
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(pem.as_bytes())
            .map_err(|_| "GitHub App privateKey must be a valid RSA PEM key")?;
        let jwt = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &key,
        )
        .map_err(|_| "unable to sign the GitHub App JWT")?;
        let response = http
            .post(format!(
                "{api}/app/installations/{installation}/access_tokens"
            ))
            .bearer_auth(jwt)
            .header("accept", "application/vnd.github+json")
            .header("x-github-api-version", version)
            .header("user-agent", "seikika-github-app")
            .send()
            .await
            .map_err(|error| {
                format!("GitHub App installation token: {}", request_failure(&error))
            })?;
        if !response.status().is_success() {
            return Err(format!(
                "GitHub App installation token: HTTP {}",
                response.status().as_u16()
            ));
        }
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|_| "invalid GitHub App installation token response")?;
        body["token"]
            .as_str()
            .filter(|token| !token.is_empty())
            .map(|token| Some(token.to_owned()))
            .ok_or_else(|| "GitHub App installation response is missing an access token".to_owned())
    }
}

/// Classify a failed GitHub request without returning its URL, headers, or error text.
pub fn request_failure(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        return "timed out";
    }
    let mut tls = false;
    let mut cause = error.source();
    while let Some(current) = cause {
        let message = current.to_string().to_ascii_lowercase();
        if message.contains("certificate") || message.contains("unknownissuer") {
            return "TLS certificate verification failed";
        }
        if message.contains("dns error") || message.contains("failed to resolve") {
            return "DNS lookup failed";
        }
        tls |= message.contains("tls");
        cause = current.source();
    }
    if tls {
        "TLS connection failed"
    } else if error.is_connect() {
        "connection failed"
    } else {
        "request failed"
    }
}
