//! Authentication (docs/v1-spec.md §9; authorization.md §4.9). In order:
//!
//! 1. **Identity-aware proxy JWT** (`iap-google`: `x-goog-iap-jwt-assertion`;
//!    `cloudflare-access`: `cf-access-jwt-assertion`), signature, issuer and audience checked.
//! 2. **Bearer token**: a personal access token (`tbx_…`), or (mode `oidc`) a JWT from the
//!    configured issuer.
//! 3. **Dev mode** only: the `x-toolbox-user` header is trusted.
//!
//! The result is a subject (an email or a login); mapping it to a `user:` entity happens in the
//! API layer, where the catalog is.

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{DecodingKey, Validation, decode, decode_header};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Dev,
    IapGoogle,
    CloudflareAccess,
    Oidc,
}

impl Mode {
    pub fn parse(s: &str) -> Result<Mode, String> {
        match s {
            "dev" => Ok(Mode::Dev),
            "iap-google" => Ok(Mode::IapGoogle),
            "cloudflare-access" => Ok(Mode::CloudflareAccess),
            "oidc" => Ok(Mode::Oidc),
            other => Err(format!(
                "unknown auth mode {other} (dev, iap-google, cloudflare-access, oidc)"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub mode: Mode,
    /// Where the signing keys are: `https://…`, `http://…` or `file://…` (JWKS JSON).
    pub jwks: Option<String>,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    /// The claim that names the user (default `email`).
    pub subject_claim: String,
    /// Personal access tokens known without a database (`TOOLBOX_PATS=alice=tbx_…`), as
    /// sha256(token) hex → subject.
    pub static_pats: HashMap<String, String>,
}

impl AuthConfig {
    /// Defaults per mode, overridable with TOOLBOX_JWKS_URL, TOOLBOX_ISSUER, TOOLBOX_AUDIENCE,
    /// TOOLBOX_SUBJECT_CLAIM and TOOLBOX_PATS.
    pub fn from_env(mode: Mode) -> AuthConfig {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let (jwks, issuer) = match mode {
            Mode::IapGoogle => (
                Some("https://www.gstatic.com/iap/verify/public_key-jwk".to_owned()),
                Some("https://cloud.google.com/iap".to_owned()),
            ),
            Mode::CloudflareAccess => {
                let team = env("TOOLBOX_CF_TEAM_DOMAIN");
                (
                    team.as_ref().map(|t| format!("{t}/cdn-cgi/access/certs")),
                    team,
                )
            }
            _ => (None, None),
        };
        let static_pats = env("TOOLBOX_PATS")
            .map(|v| {
                v.split(',')
                    .filter_map(|p| {
                        p.split_once('=')
                            .map(|(user, tok)| (pat_hash(tok.trim()), user.trim().to_owned()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        AuthConfig {
            mode,
            jwks: env("TOOLBOX_JWKS_URL").or(jwks),
            issuer: env("TOOLBOX_ISSUER").or(issuer),
            audience: env("TOOLBOX_AUDIENCE"),
            subject_claim: env("TOOLBOX_SUBJECT_CLAIM").unwrap_or_else(|| "email".into()),
            static_pats,
        }
    }
}

/// Who is calling, before catalog mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// An email or a login.
    pub subject: String,
    /// `iap`, `oidc`, `pat` or `dev`.
    pub method: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// No credentials were presented.
    Missing,
    /// Credentials were presented and rejected.
    Invalid(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Missing => write!(f, "no credentials"),
            AuthError::Invalid(m) => write!(f, "invalid credentials: {m}"),
        }
    }
}

/// sha256 hex of a personal access token (only hashes are stored).
pub fn pat_hash(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A new personal access token: `tbx_` + 48 hex digits from the OS random source.
pub fn new_pat() -> std::io::Result<String> {
    use std::io::Read;
    let mut bytes = [0u8; 24];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(format!(
        "tbx_{}",
        bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
    ))
}

/// Looks up a PAT hash; implemented over Postgres by the server.
#[async_trait::async_trait]
pub trait PatLookup: Send + Sync {
    async fn subject_for(&self, hash: &str) -> Option<String>;
}

pub struct Authenticator {
    pub config: AuthConfig,
    http: reqwest::Client,
    keys: RwLock<Option<(Instant, JwkSet)>>,
    pats: Option<Box<dyn PatLookup>>,
}

impl Authenticator {
    pub fn new(config: AuthConfig, pats: Option<Box<dyn PatLookup>>) -> Self {
        Authenticator {
            config,
            http: reqwest::Client::new(),
            keys: RwLock::new(None),
            pats,
        }
    }

    async fn jwks(&self) -> Result<JwkSet, String> {
        if let Some((at, set)) = self.keys.read().unwrap().as_ref()
            && at.elapsed() < Duration::from_secs(600)
        {
            return Ok(set.clone());
        }
        let url = self
            .config
            .jwks
            .as_deref()
            .ok_or("no JWKS configured (TOOLBOX_JWKS_URL)")?;
        let text = if let Some(path) = url.strip_prefix("file://") {
            std::fs::read_to_string(path).map_err(|e| format!("JWKS {path}: {e}"))?
        } else {
            self.http
                .get(url)
                .send()
                .await
                .and_then(|r| r.error_for_status())
                .map_err(|e| format!("JWKS {url}: {e}"))?
                .text()
                .await
                .map_err(|e| e.to_string())?
        };
        let set: JwkSet = serde_json::from_str(&text).map_err(|e| format!("JWKS {url}: {e}"))?;
        *self.keys.write().unwrap() = Some((Instant::now(), set.clone()));
        Ok(set)
    }

    /// Verify a JWT against the configured keys, issuer and audience; return the subject claim.
    pub async fn verify_jwt(&self, token: &str) -> Result<String, String> {
        let header = decode_header(token).map_err(|e| format!("malformed JWT: {e}"))?;
        let set = self.jwks().await?;
        let jwk = match &header.kid {
            Some(kid) => set
                .find(kid)
                .ok_or_else(|| format!("unknown key id {kid}"))?,
            None => set.keys.first().ok_or("empty JWKS")?,
        };
        let key = DecodingKey::from_jwk(jwk).map_err(|e| format!("bad JWK: {e}"))?;
        let mut v = Validation::new(header.alg);
        match &self.config.audience {
            Some(aud) => v.set_audience(&[aud]),
            None => v.validate_aud = false,
        }
        if let Some(iss) = &self.config.issuer {
            v.set_issuer(&[iss]);
        }
        let data = decode::<Value>(token, &key, &v).map_err(|e| format!("JWT rejected: {e}"))?;
        data.claims
            .get(&self.config.subject_claim)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| format!("JWT has no `{}` claim", self.config.subject_claim))
    }

    /// Authenticate a request, given a case-insensitive header lookup.
    pub async fn authenticate(
        &self,
        header: impl Fn(&str) -> Option<String>,
    ) -> Result<Identity, AuthError> {
        let proxy_header = match self.config.mode {
            Mode::IapGoogle => Some("x-goog-iap-jwt-assertion"),
            Mode::CloudflareAccess => Some("cf-access-jwt-assertion"),
            _ => None,
        };
        if let Some(h) = proxy_header
            && let Some(token) = header(h)
        {
            return self
                .verify_jwt(&token)
                .await
                .map(|subject| Identity {
                    subject,
                    method: "iap",
                })
                .map_err(AuthError::Invalid);
        }
        if let Some(bearer) =
            header("authorization").and_then(|v| v.strip_prefix("Bearer ").map(str::to_owned))
        {
            if bearer.starts_with("tbx_") {
                let hash = pat_hash(&bearer);
                if let Some(s) = self.config.static_pats.get(&hash) {
                    return Ok(Identity {
                        subject: s.clone(),
                        method: "pat",
                    });
                }
                if let Some(p) = &self.pats
                    && let Some(s) = p.subject_for(&hash).await
                {
                    return Ok(Identity {
                        subject: s,
                        method: "pat",
                    });
                }
                return Err(AuthError::Invalid(
                    "unknown or revoked personal access token".into(),
                ));
            }
            if self.config.mode == Mode::Oidc {
                return self
                    .verify_jwt(&bearer)
                    .await
                    .map(|subject| Identity {
                        subject,
                        method: "oidc",
                    })
                    .map_err(AuthError::Invalid);
            }
            return Err(AuthError::Invalid(
                "bearer JWTs are accepted only in oidc mode".into(),
            ));
        }
        if self.config.mode == Mode::Dev
            && let Some(u) = header("x-toolbox-user")
        {
            return Ok(Identity {
                subject: u,
                method: "dev",
            });
        }
        Err(AuthError::Missing)
    }
}
