//! Integration tests against wiremock HTTP servers for each remote backend.

use std::collections::BTreeMap;
use std::sync::Arc;

use toolbox_secrets::backends::SecretBackend;
use toolbox_secrets::backends::aws::AwsSecretsManager;
use toolbox_secrets::backends::azure::AzureKeyVault;
use toolbox_secrets::backends::gcp::GcpSecretManager;
use toolbox_secrets::backends::vault::VaultBackend;
use toolbox_secrets::http::{HttpResponse, HttpTransport, SharedHttp};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Transport that forwards to real HTTP via reqwest (exercises wiremock).
struct LiveHttp;

impl HttpTransport for LiveHttp {
    fn get(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, toolbox_secrets::SecretError> {
        toolbox_secrets::http::ReqwestTransport::new()?.get(url, headers)
    }

    fn post(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> Result<HttpResponse, toolbox_secrets::SecretError> {
        toolbox_secrets::http::ReqwestTransport::new()?.post(url, headers, body)
    }
}

fn live() -> SharedHttp {
    Arc::new(LiveHttp)
}

#[tokio::test]
async fn gcp_wiremock_access() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/projects/demo/secrets/toolbox-github-app-pem/versions/latest:access",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"payload":{"data":"cGVtLWRhdGE="}}"#, // "pem-data"
            "application/json",
        ))
        .mount(&server)
        .await;

    let b = GcpSecretManager::new(
        "demo",
        Some(server.uri()),
        BTreeMap::from([("github-app-private-key".into(), "toolbox-github-app-pem".into())]),
        live(),
        Arc::new(|| Ok(Some("tok".into()))),
    );
    let v = tokio::task::spawn_blocking(move || b.get("github-app-private-key"))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(v.expose(), "pem-data");
}

#[tokio::test]
async fn vault_wiremock_kv() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/secret/data/toolbox/jira-app-token"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"data":{"data":{"value":"jira-from-vault"}}}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let b = VaultBackend::new(
        server.uri(),
        None,
        None,
        BTreeMap::new(),
        live(),
        Arc::new(|| Ok(Some("s.t".into()))),
    );
    let v = tokio::task::spawn_blocking(move || b.get("jira-app-token"))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(v.expose(), "jira-from-vault");
}

#[tokio::test]
async fn aws_wiremock_get_secret() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"SecretString":"aws-pem"}"#,
            "application/x-amz-json-1.1",
        ))
        .mount(&server)
        .await;

    let b = AwsSecretsManager::new(
        "us-east-1",
        Some(server.uri()),
        BTreeMap::from([("github-app-private-key".into(), "toolbox/github".into())]),
        live(),
        Arc::new(|| Ok(vec![])),
    );
    let v = tokio::task::spawn_blocking(move || b.get("github-app-private-key"))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(v.expose(), "aws-pem");
}

#[tokio::test]
async fn azure_wiremock_secret() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/secrets/toolbox-gcp-connector-sa"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"value":"{\"type\":\"service_account\"}"}"#,
            "application/json",
        ))
        .mount(&server)
        .await;

    let b = AzureKeyVault::new(
        server.uri(),
        None,
        BTreeMap::from([("gcp-sa-key".into(), "toolbox-gcp-connector-sa".into())]),
        live(),
        Arc::new(|| Ok(Some("aad".into()))),
    );
    let v = tokio::task::spawn_blocking(move || b.get("gcp-sa-key"))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(v.expose().contains("service_account"));
}
