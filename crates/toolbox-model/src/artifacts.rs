//! Executable connectors and executors from releases (D90): a connector's or an `in-image`
//! executor's `package:` of the form `github:owner/repo[/app]@version` names the release asset
//! `<app>-<target>.tar.xz` (`.zip` on Windows) at tag `v<version>`, the layout `dist` produces.
//! `toolbox install` downloads it, checks it against the portal's `toolbox.lock`, and unpacks it
//! into the cache, where [`Spec::installed`] finds it.

use std::path::PathBuf;

/// A released executable: repository, app (binary) name and version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub owner: String,
    pub repo: String,
    pub app: String,
    pub version: String,
}

/// Parse `github:owner/repo[/app]@version`; `None` for any other package.
pub fn parse(package: &str) -> Option<Spec> {
    let rest = package.strip_prefix("github:")?;
    let (path, version) = rest.rsplit_once('@')?;
    let mut parts = path.split('/');
    let (owner, repo) = (parts.next()?, parts.next()?);
    let app = parts.next().unwrap_or(repo);
    if owner.is_empty() || repo.is_empty() || version.is_empty() || parts.next().is_some() {
        return None;
    }
    Some(Spec {
        owner: owner.into(),
        repo: repo.into(),
        app: app.into(),
        version: version.trim_start_matches('v').into(),
    })
}

/// This platform's Rust target triple, as release assets are named.
pub fn target() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        _ => "unsupported",
    }
}

impl Spec {
    /// The archive's file name for `target`.
    pub fn asset(&self, target: &str) -> String {
        let ext = if target.contains("windows") {
            "zip"
        } else {
            "tar.xz"
        };
        format!("{}-{target}.{ext}", self.app)
    }

    /// The asset's URL under `base` (`https://github.com`, or `TOOLBOX_RELEASES_URL`).
    pub fn url(&self, base: &str, target: &str) -> String {
        format!(
            "{}/{}/{}/releases/download/v{}/{}",
            base.trim_end_matches('/'),
            self.owner,
            self.repo,
            self.version,
            self.asset(target)
        )
    }

    /// Where the unpacked binary lives in the cache.
    pub fn installed(&self) -> PathBuf {
        cache_dir()
            .join("connectors")
            .join(&self.owner)
            .join(&self.repo)
            .join(&self.version)
            .join(target())
            .join(format!("{}{}", self.app, std::env::consts::EXE_SUFFIX))
    }
}

/// `TOOLBOX_CACHE_DIR`, else `$XDG_CACHE_HOME/toolbox`, else `~/.cache/toolbox`.
pub fn cache_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("TOOLBOX_CACHE_DIR") {
        return d.into();
    }
    if let Some(d) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(d).join("toolbox");
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    PathBuf::from(home.unwrap_or_default()).join(".cache/toolbox")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_and_assets() {
        let s = parse("github:acme/toolbox-connectors/jira@v1.2.0").unwrap();
        assert_eq!((s.app.as_str(), s.version.as_str()), ("jira", "1.2.0"));
        assert_eq!(
            s.url("https://github.com", "x86_64-unknown-linux-gnu"),
            "https://github.com/acme/toolbox-connectors/releases/download/v1.2.0/jira-x86_64-unknown-linux-gnu.tar.xz"
        );
        assert_eq!(
            s.asset("x86_64-pc-windows-msvc"),
            "jira-x86_64-pc-windows-msvc.zip"
        );
        assert_eq!(parse("github:acme/jira@1.0").unwrap().app, "jira");
        for bad in [
            "builtin:github",
            "github:acme@1",
            "github:a/b/c/d@1",
            "github:a/b",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }
}
