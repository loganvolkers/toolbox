//! Minimal HTTP transport for remote secret backends (mockable in tests).

use crate::error::SecretError;
use std::collections::HashMap;
use std::sync::Arc;

/// One HTTP response used by remote backends.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

/// Pluggable HTTP GET/POST so unit tests can fake Vault/AWS/GCP/Azure without live cloud.
pub trait HttpTransport: Send + Sync {
    fn get(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, SecretError>;
    fn post(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> Result<HttpResponse, SecretError>;
}

/// Drive an async future from sync `SecretBackend::get` on a dedicated thread so we never
/// nest `block_on` inside a current-thread Tokio runtime (tests, serve).
#[cfg(feature = "remote")]
pub(crate) fn block_on_async<F, T>(fut: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("secrets runtime")
            .block_on(fut)
    })
    .join()
    .expect("secrets http thread")
}

/// reqwest-backed transport used in production.
#[cfg(feature = "remote")]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

#[cfg(feature = "remote")]
impl ReqwestTransport {
    pub fn new() -> Result<Self, SecretError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| SecretError::backend("http", e.to_string()))?;
        Ok(Self { client })
    }
}

#[cfg(feature = "remote")]
impl Default for ReqwestTransport {
    fn default() -> Self {
        Self::new().expect("reqwest client")
    }
}

#[cfg(feature = "remote")]
impl HttpTransport for ReqwestTransport {
    fn get(&self, url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse, SecretError> {
        let client = self.client.clone();
        let url = url.to_owned();
        let headers: Vec<(String, String)> = headers
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        block_on_async(async move {
            let mut req = client.get(&url);
            for (k, v) in &headers {
                req = req.header(k, v);
            }
            let resp = req
                .send()
                .await
                .map_err(|e| SecretError::backend("http", e.to_string()))?;
            let status = resp.status().as_u16();
            let body = resp
                .text()
                .await
                .map_err(|e| SecretError::backend("http", e.to_string()))?;
            Ok(HttpResponse { status, body })
        })
    }

    fn post(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> Result<HttpResponse, SecretError> {
        let client = self.client.clone();
        let url = url.to_owned();
        let body = body.to_owned();
        let headers: Vec<(String, String)> = headers
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        block_on_async(async move {
            let mut req = client.post(&url).body(body);
            for (k, v) in &headers {
                req = req.header(k, v);
            }
            let resp = req
                .send()
                .await
                .map_err(|e| SecretError::backend("http", e.to_string()))?;
            let status = resp.status().as_u16();
            let body = resp
                .text()
                .await
                .map_err(|e| SecretError::backend("http", e.to_string()))?;
            Ok(HttpResponse { status, body })
        })
    }
}

/// In-memory fake for tests: map method+url → response.
#[derive(Default)]
pub struct FakeTransport {
    pub gets: HashMap<String, HttpResponse>,
    pub posts: HashMap<String, HttpResponse>,
}

impl HttpTransport for FakeTransport {
    fn get(&self, url: &str, _headers: &[(&str, &str)]) -> Result<HttpResponse, SecretError> {
        self.gets
            .get(url)
            .cloned()
            .ok_or_else(|| SecretError::backend("fake-http", format!("no GET stub for {url}")))
    }

    fn post(
        &self,
        url: &str,
        _headers: &[(&str, &str)],
        _body: &str,
    ) -> Result<HttpResponse, SecretError> {
        self.posts
            .get(url)
            .cloned()
            .ok_or_else(|| SecretError::backend("fake-http", format!("no POST stub for {url}")))
    }
}

pub type SharedHttp = Arc<dyn HttpTransport>;

#[cfg(feature = "remote")]
pub fn default_http() -> Result<SharedHttp, SecretError> {
    Ok(Arc::new(ReqwestTransport::new()?))
}
