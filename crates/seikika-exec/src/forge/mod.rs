//! Forges (docs/design/forges.md): where a portal's repositories live, and what Seikika asks of
//! them: open a review for a pushed branch, and say whose a token is. One [`ForgeDriver`] per
//! `forge::Kind`; the manifest side (`forges:`, resolving a URL to a forge) is
//! `seikika_model::forge`. CI is not a forge capability: `crate::ci` starts runs on the CI host
//! a forge's entry names, or on the forge's own CI.

pub mod github;
pub mod gitlab;

use crate::executor::git;
use reqwest::RequestBuilder;
use seikika_model::forge::{Kind, Resolved};
use serde_json::Value;
use std::path::Path;

/// A branch to push and propose for review.
pub struct Proposal<'a> {
    /// The repository the branch is pushed from.
    pub repo: &'a Path,
    pub remote: &'a str,
    pub branch: &'a str,
    /// The branch it would merge into.
    pub target: &'a str,
    pub title: &'a str,
    pub body: &'a str,
}

/// What Seikika asks of one forge type.
pub trait ForgeDriver: Sync {
    fn kind(&self) -> Kind;

    /// `req` with `token` in the header this forge reads it from.
    fn auth(&self, req: RequestBuilder, token: String) -> RequestBuilder;

    /// The username in this forge's `GET /user`, or why the account can't act.
    fn username(&self, user: &Value) -> Result<Option<String>, String>;

    /// Open a review for `head`, already pushed, on `f`'s project, with its token: `{forge, url,
    /// …}`, or `{forge, error}` when it can't (the branch stays pushed, and the run `proposed`).
    fn open_review(&self, f: &Resolved, head: &str, base: &str, title: &str, body: &str) -> Value;

    /// Push a proposed branch and open its review.
    fn propose(&self, f: &Resolved, p: &Proposal<'_>) -> Result<Value, String> {
        git(p.repo, &["push", "-q", p.remote, p.branch])?;
        Ok(self.open_review(f, p.branch, p.target, p.title, p.body))
    }
}

/// The driver for a forge type.
pub fn driver(kind: Kind) -> &'static dyn ForgeDriver {
    match kind {
        Kind::Github => &github::Github,
        Kind::Gitlab => &gitlab::Gitlab,
    }
}

/// Open a pull or merge request for `head`, already pushed, on `f`'s project, with its token.
pub fn open_review(f: &Resolved, head: &str, base: &str, title: &str, body: &str) -> Value {
    driver(f.kind).open_review(f, head, base, title, body)
}

/// Who owns `token` on a forge, as its username: GitLab's `GET /user` `username` (an active
/// account only), GitHub's `login`. The token goes to `api` and nowhere else.
pub fn whoami(kind: Kind, api: &str, token: &str) -> Result<String, String> {
    let d = driver(kind);
    let url = format!("{}/user", api.trim_end_matches('/'));
    let (st, text) =
        crate::http::send(d.auth(crate::http::client().get(url), token.trim().to_owned()))?;
    if !st.is_success() {
        return Err(format!("{} did not accept the token ({st})", kind.name()));
    }
    let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    d.username(&v)?
        .ok_or_else(|| format!("{} named no user for the token", kind.name()))
}
