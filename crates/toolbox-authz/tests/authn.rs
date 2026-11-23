//! Authentication: IAP-style and OIDC JWTs (RS256 and ES256) against a JWKS file, issuer and
//! audience checks, expiry, PATs and dev headers.

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use toolbox_authz::authn::{AuthConfig, AuthError, Authenticator, Mode, pat_hash};

fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .display()
        .to_string()
}

fn config(mode: Mode) -> AuthConfig {
    AuthConfig {
        mode,
        jwks: Some(format!("file://{}", fixture("test-jwks.json"))),
        issuer: Some("https://cloud.google.com/iap".into()),
        audience: Some("/projects/1/global/backendServices/2".into()),
        subject_claim: "email".into(),
        static_pats: HashMap::from([(pat_hash("tbx_alice"), "alice".to_owned())]),
    }
}

fn token(alg: Algorithm, kid: &str, pem: &str, claims: serde_json::Value) -> String {
    let pem = std::fs::read(fixture(pem)).unwrap();
    let key = match alg {
        Algorithm::RS256 => EncodingKey::from_rsa_pem(&pem).unwrap(),
        _ => EncodingKey::from_ec_pem(&pem).unwrap(),
    };
    let mut h = Header::new(alg);
    h.kid = Some(kid.into());
    encode(&h, &claims, &key).unwrap()
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn headers<'a>(pairs: &'a [(&'a str, String)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |name| {
        pairs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    }
}

#[tokio::test]
async fn iap_jwt_es256_is_verified() {
    let a = Authenticator::new(config(Mode::IapGoogle), None);
    let claims = json!({"email": "alice@acme.example", "iss": "https://cloud.google.com/iap",
                        "aud": "/projects/1/global/backendServices/2", "exp": now() + 300});
    let t = token(Algorithm::ES256, "test-ec", "test-ec.pem", claims.clone());
    let id = a
        .authenticate(headers(&[("x-goog-iap-jwt-assertion", t)]))
        .await
        .unwrap();
    assert_eq!(
        (id.subject.as_str(), id.method),
        ("alice@acme.example", "iap")
    );

    let mut wrong_aud = claims.clone();
    wrong_aud["aud"] = json!("other");
    let t = token(Algorithm::ES256, "test-ec", "test-ec.pem", wrong_aud);
    assert!(matches!(
        a.authenticate(headers(&[("x-goog-iap-jwt-assertion", t)]))
            .await,
        Err(AuthError::Invalid(_))
    ));

    let mut expired = claims;
    expired["exp"] = json!(now() - 3600);
    let t = token(Algorithm::ES256, "test-ec", "test-ec.pem", expired);
    assert!(matches!(
        a.authenticate(headers(&[("x-goog-iap-jwt-assertion", t)]))
            .await,
        Err(AuthError::Invalid(_))
    ));
}

#[tokio::test]
async fn oidc_bearer_rs256_and_rejections() {
    let a = Authenticator::new(config(Mode::Oidc), None);
    let claims = json!({"email": "bob@acme.example", "iss": "https://cloud.google.com/iap",
                        "aud": "/projects/1/global/backendServices/2", "exp": now() + 300});
    let t = token(Algorithm::RS256, "test-rsa", "test-rsa.pem", claims);
    let id = a
        .authenticate(headers(&[("authorization", format!("Bearer {t}"))]))
        .await
        .unwrap();
    assert_eq!(
        (id.subject.as_str(), id.method),
        ("bob@acme.example", "oidc")
    );
    // Tampered signature.
    let bad = format!("{}x", &t[..t.len() - 1]);
    assert!(matches!(
        a.authenticate(headers(&[("authorization", format!("Bearer {bad}"))]))
            .await,
        Err(AuthError::Invalid(_))
    ));
    // Dev header is ignored outside dev mode.
    assert_eq!(
        a.authenticate(headers(&[("x-toolbox-user", "mallory".into())]))
            .await,
        Err(AuthError::Missing)
    );
}

#[tokio::test]
async fn pats_and_dev_headers() {
    let a = Authenticator::new(config(Mode::Dev), None);
    let id = a
        .authenticate(headers(&[("authorization", "Bearer tbx_alice".into())]))
        .await
        .unwrap();
    assert_eq!((id.subject.as_str(), id.method), ("alice", "pat"));
    assert!(matches!(
        a.authenticate(headers(&[("authorization", "Bearer tbx_nope".into())]))
            .await,
        Err(AuthError::Invalid(_))
    ));
    let id = a
        .authenticate(headers(&[("x-toolbox-user", "erin".into())]))
        .await
        .unwrap();
    assert_eq!(id.method, "dev");
    assert_eq!(a.authenticate(headers(&[])).await, Err(AuthError::Missing));
}

#[tokio::test]
async fn token_mode_accepts_pats_and_ignores_the_dev_header() {
    let a = Authenticator::new(config(Mode::Token), None);
    let id = a
        .authenticate(headers(&[("authorization", "Bearer tbx_alice".into())]))
        .await
        .unwrap();
    assert_eq!((id.subject.as_str(), id.method), ("alice", "pat"));
    assert_eq!(
        a.authenticate(headers(&[("x-toolbox-user", "mallory".into())]))
            .await,
        Err(AuthError::Missing)
    );
    assert!(matches!(
        a.authenticate(headers(&[(
            "authorization",
            "Bearer eyJhbGciOi.x.y".into()
        )]))
        .await,
        Err(AuthError::Invalid(_))
    ));
}
