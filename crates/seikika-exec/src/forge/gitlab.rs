//! GitLab, gitlab.com or self-managed: merge requests from push options (no API token; git's own
//! credentials decide who opens it) or through the API with the forge's token. The token is sent
//! as `PRIVATE-TOKEN`. GitLab CI is a CI host (`crate::ci::gitlab`).

use super::{ForgeDriver, Proposal};
use crate::executor::git_said;
use reqwest::RequestBuilder;
use seikika_model::forge::{Kind, Resolved};
use serde_json::{Value, json};

/// GitLab.com and self-managed GitLab.
pub struct Gitlab;

/// A project path in a URL path segment: `acme/pay` is `acme%2Fpay`.
pub(crate) fn encode(project: &str) -> String {
    project
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// GitLab's reason, from `{"message": …}` or `{"error": …}`, else the body.
pub(crate) fn reason(text: &str) -> String {
    let v: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    match v.get("message").or_else(|| v.get("error")) {
        Some(Value::String(s)) => s.clone(),
        Some(m) => m.to_string(),
        None => text.trim().to_owned(),
    }
}

/// The merge request GitLab names after a push with `merge_request.create`.
fn merge_request_url(said: &str) -> Option<String> {
    said.lines()
        .map(|l| l.trim_start_matches("remote:").trim())
        .filter(|l| l.contains("/-/merge_requests/"))
        .find_map(|l| l.split_whitespace().find(|w| w.starts_with("http")))
        .map(str::to_owned)
}

impl ForgeDriver for Gitlab {
    fn kind(&self) -> Kind {
        Kind::Gitlab
    }

    fn auth(&self, req: RequestBuilder, token: String) -> RequestBuilder {
        req.header("private-token", token)
    }

    fn username(&self, user: &Value) -> Result<Option<String>, String> {
        if let Some(state) = user["state"].as_str().filter(|s| *s != "active") {
            return Err(format!("the token's GitLab account is {state}"));
        }
        Ok(user["username"].as_str().map(str::to_owned))
    }

    /// Through the API, for a branch already pushed: `{forge, url, iid}`.
    fn open_review(&self, f: &Resolved, head: &str, base: &str, title: &str, body: &str) -> Value {
        let Ok(token) = std::env::var(&f.token) else {
            return json!({"forge": "gitlab", "error": format!(
                "set {} to open a merge request; the branch is pushed", f.token)});
        };
        let url = format!(
            "{}/projects/{}/merge_requests",
            f.api.trim_end_matches('/'),
            encode(&f.project)
        );
        let req = json!({"source_branch": head, "target_branch": base, "title": title,
            "description": body, "remove_source_branch": true});
        let post = || {
            crate::http::send(
                self.auth(crate::http::client().post(&url), token.clone())
                    .json(&req),
            )
        };
        // GitLab takes in a push before it knows the branch: until then it says the source branch
        // does not exist.
        let mut res = post();
        for _ in 0..20 {
            match &res {
                Ok((st, text)) if st.as_u16() == 400 && text.contains("source_branch") => {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    res = post();
                }
                _ => break,
            }
        }
        match res {
            Ok((st, text)) if st.is_success() => {
                let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                json!({"forge": "gitlab", "url": v["web_url"], "iid": v["iid"]})
            }
            Ok((st, text)) => json!({"forge": "gitlab",
                "error": format!("GitLab refused the merge request ({st}): {}", reason(&text))}),
            Err(e) => json!({"forge": "gitlab", "error": e}),
        }
    }

    /// One push that also opens the merge request (push options).
    fn propose(&self, _: &Resolved, p: &Proposal<'_>) -> Result<Value, String> {
        let said = git_said(
            p.repo,
            &[
                "push",
                "-o",
                "merge_request.create",
                "-o",
                &format!("merge_request.target={}", p.target),
                "-o",
                &format!("merge_request.title={}", p.title.replace('\n', " ")),
                "--",
                p.remote,
                p.branch,
            ],
        )?;
        Ok(match merge_request_url(&said) {
            Some(u) => json!({"forge": "gitlab", "url": u}),
            None => json!({"forge": "gitlab",
                "error": "GitLab opened no merge request (it says so on push); the branch is pushed"}),
        })
    }
}
