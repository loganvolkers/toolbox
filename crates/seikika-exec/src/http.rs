//! Blocking HTTP for the forge and CI drivers. The lifecycle is synchronous and may already run
//! inside a server's runtime, so each request gets a current-thread runtime on its own thread.

use reqwest::{RequestBuilder, StatusCode};

pub(crate) fn block<T: Send + 'static>(
    f: impl std::future::Future<Output = T> + Send + 'static,
) -> Result<T, String> {
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map(|rt| rt.block_on(f))
            .map_err(|e| e.to_string())
    })
    .join()
    .map_err(|_| "http thread panicked".to_string())?
}

/// Send `req` with Seikika's user agent: the status and the body as text.
pub(crate) fn send(req: RequestBuilder) -> Result<(StatusCode, String), String> {
    block(async move {
        let r = req
            .header("user-agent", "seikika")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = r.status();
        Ok((status, r.text().await.unwrap_or_default()))
    })?
}

pub(crate) fn client() -> reqwest::Client {
    reqwest::Client::new()
}
