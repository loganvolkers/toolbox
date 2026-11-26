//! GitHub and GitHub Enterprise Server: pull requests through the REST API with the forge's
//! token. GitHub Actions is a CI host (`crate::ci::github`).

use super::ForgeDriver;
use reqwest::RequestBuilder;
use seikika_model::forge::{Kind, Resolved};
use serde_json::{Value, json};

/// GitHub and GitHub Enterprise Server.
pub struct Github;

impl ForgeDriver for Github {
    fn kind(&self) -> Kind {
        Kind::Github
    }

    fn auth(&self, req: RequestBuilder, token: String) -> RequestBuilder {
        req.bearer_auth(token)
            .header("accept", "application/vnd.github+json")
    }

    fn username(&self, user: &Value) -> Result<Option<String>, String> {
        Ok(user["login"].as_str().map(str::to_owned))
    }

    fn open_review(&self, f: &Resolved, head: &str, base: &str, title: &str, body: &str) -> Value {
        let Ok(token) = std::env::var(&f.token) else {
            return json!({"forge": "github", "error": format!(
                "set {} to open a pull request; the branch is pushed", f.token)});
        };
        let url = format!("{}/repos/{}/pulls", f.api.trim_end_matches('/'), f.project);
        let req = json!({"title": title, "head": head, "base": base, "body": body});
        match crate::http::send(self.auth(crate::http::client().post(url), token).json(&req)) {
            Ok((st, text)) => {
                let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                if st.is_success() {
                    json!({"forge": "github", "url": v["html_url"], "number": v["number"]})
                } else {
                    json!({"forge": "github",
                        "error": format!("GitHub refused the pull request ({st}): {}", v["message"])})
                }
            }
            Err(e) => json!({"forge": "github", "error": e}),
        }
    }
}
