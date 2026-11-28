//! GitHub App installation authentication for connectors.

use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// GitHub App credentials with an optional explicit installation ID.
#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Installation {
    app_id: Option<String>,
    private_key: Option<String>,
    installation_id: Option<String>,
}

impl Installation {
    /// Construct credentials for a known installation.
    pub fn new(app_id: String, private_key: String, installation_id: String) -> Self {
        Self::with_optional_installation_id(app_id, private_key, Some(installation_id))
    }

    /// Construct credentials read from a flat connector config.
    pub fn with_optional_installation_id(
        app_id: String,
        private_key: String,
        installation_id: Option<String>,
    ) -> Self {
        Self {
            app_id: Some(app_id),
            private_key: Some(private_key),
            installation_id,
        }
    }

    /// Mint an installation access token, or return `None` when no app is configured.
    pub async fn token(
        &self,
        http: &reqwest::Client,
        api: &str,
        version: &str,
    ) -> Result<Option<String>, String> {
        self.token_with_org(http, api, version, None).await
    }

    /// Mint a token using the configured ID or the App's installation on `organization`.
    pub async fn token_for_org(
        &self,
        http: &reqwest::Client,
        api: &str,
        version: &str,
        organization: &str,
    ) -> Result<Option<String>, String> {
        self.token_with_org(http, api, version, Some(organization))
            .await
    }

    async fn token_with_org(
        &self,
        http: &reqwest::Client,
        api: &str,
        version: &str,
        organization: Option<&str>,
    ) -> Result<Option<String>, String> {
        let Some(jwt) = self.jwt()? else {
            return Ok(None);
        };
        let api = api.trim_end_matches('/');
        let installation = match (self.installation_id.as_deref(), organization) {
            (Some(id), _) if !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()) => {
                id.to_owned()
            }
            (Some(_), _) => return Err("GitHub App installationId must be numeric".into()),
            (None, Some(org)) => discover_installation(http, api, version, &jwt, org).await?,
            (None, None) => {
                return Err(
                    "GitHub App needs an installationId or an organization for discovery".into(),
                );
            }
        };
        let response = app_request(
            http.post(format!(
                "{api}/app/installations/{installation}/access_tokens"
            )),
            &jwt,
            version,
            "GitHub App installation token",
        )
        .await?;
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

    fn jwt(&self) -> Result<Option<String>, String> {
        let (app_id, pem) = match (self.app_id.as_deref(), self.private_key.as_deref()) {
            (None, None) if self.installation_id.is_none() => return Ok(None),
            (Some(id), Some(pem)) if !id.trim().is_empty() && !pem.trim().is_empty() => {
                (id.trim(), pem)
            }
            _ => return Err("GitHub App authentication needs appId and privateKey".into()),
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let claims = json!({"iat": now.saturating_sub(60), "exp": now + 540, "iss": app_id});
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(pem.as_bytes())
            .map_err(|_| "GitHub App privateKey must be a valid RSA PEM key")?;
        jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &key,
        )
        .map(Some)
        .map_err(|_| "unable to sign the GitHub App JWT".into())
    }
}

async fn discover_installation(
    http: &reqwest::Client,
    api: &str,
    version: &str,
    jwt: &str,
    organization: &str,
) -> Result<String, String> {
    if matches!(organization, "" | "." | "..")
        || !organization
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err("GitHub App needs a valid organization name for installation discovery".into());
    }
    let response = app_request(
        http.get(format!("{api}/orgs/{organization}/installation")),
        jwt,
        version,
        "GitHub App organization installation lookup",
    )
    .await?;
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|_| "invalid GitHub App organization installation response")?;
    body["id"]
        .as_u64()
        .filter(|id| *id > 0)
        .map(|id| id.to_string())
        .ok_or_else(|| "GitHub App organization installation response is missing an ID".to_owned())
}

async fn app_request(
    request: reqwest::RequestBuilder,
    jwt: &str,
    version: &str,
    stage: &str,
) -> Result<reqwest::Response, String> {
    let response = request
        .bearer_auth(jwt)
        .header("accept", "application/vnd.github+json")
        .header("x-github-api-version", version)
        .header("user-agent", "seikika-github-app")
        .send()
        .await
        .map_err(|error| format!("{stage}: {}", request_failure(&error)))?;
    if !response.status().is_success() {
        return Err(format!("{stage}: HTTP {}", response.status().as_u16()));
    }
    Ok(response)
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
