//! Forges (docs/design/forges.md): the code hosts a portal reaches beyond plain git. A git URL is
//! on the forge whose `url` it starts with (`https://`), or whose host it names (`ssh://`,
//! `git@host:path`). `https://github.com` and `https://gitlab.com` need no declaration; any other
//! host is plain git until `forges:` names it. The project is the rest of the URL, without `.git`.

use crate::manifest::Forge;

/// A forge type: the driver that implements its capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Github,
    Gitlab,
}

impl Kind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "github" => Some(Kind::Github),
            "gitlab" => Some(Kind::Gitlab),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Github => "github",
            Kind::Gitlab => "gitlab",
        }
    }
}

/// A forge with its defaults filled in, and the project a git URL names on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub kind: Kind,
    pub url: String,
    pub api: String,
    /// The environment variable holding the API token.
    pub token: String,
    /// `None`: a parent may dispatch to the children it lists on this forge.
    pub projects: Option<Vec<String>>,
    /// `owner/repo`, or `group/subgroup/project`.
    pub project: String,
}

impl Resolved {
    /// Whether `projects` admits this project (always, when the forge lists none).
    pub fn admits(&self) -> bool {
        self.projects
            .as_ref()
            .is_none_or(|ps| ps.iter().any(|p| path_glob(p, &self.project)))
    }
}

/// The hosts known without a declaration.
const KNOWN: &[(&str, Kind)] = &[
    ("https://github.com", Kind::Github),
    ("https://gitlab.com", Kind::Gitlab),
];

/// A declared forge's type: its own, or a known host's.
pub fn kind_of(f: &Forge) -> Option<Kind> {
    match &f.kind {
        Some(k) => Kind::parse(k),
        None => KNOWN
            .iter()
            .find(|(u, _)| trim(&f.url) == *u)
            .map(|(_, k)| *k),
    }
}

fn trim(u: &str) -> &str {
    u.trim_end_matches('/')
}

/// The API base a forge has when it names none.
pub fn default_api(kind: Kind, url: &str) -> String {
    let url = trim(url);
    match kind {
        Kind::Github if url == "https://github.com" => "https://api.github.com".into(),
        Kind::Github => format!("{url}/api/v3"),
        Kind::Gitlab => format!("{url}/api/v4"),
    }
}

/// The token variable a forge reads when it names none.
pub fn default_token(kind: Kind) -> &'static str {
    match kind {
        Kind::Github => "GITHUB_TOKEN",
        Kind::Gitlab => "GITLAB_TOKEN",
    }
}

/// The host (`host[:port]`) and path of a git URL; `None` for a local path or `file://`.
fn split_git(git: &str) -> Option<(String, String, bool)> {
    if let Some((scheme, rest)) = git.split_once("://") {
        let (auth, path) = rest.split_once('/').unwrap_or((rest, ""));
        let host = auth.rsplit('@').next().unwrap_or(auth);
        return match scheme {
            "https" | "http" => Some((host.to_owned(), path.to_owned(), true)),
            "ssh" | "git" => Some((host.to_owned(), path.to_owned(), false)),
            _ => None,
        };
    }
    let (head, path) = git.split_once(':')?;
    if head.contains('/') || !head.contains('@') {
        return None;
    }
    let host = head.rsplit('@').next().unwrap_or(head);
    Some((host.to_owned(), path.to_owned(), false))
}

fn web_host_and_prefix(url: &str) -> Option<(String, String)> {
    let (_, rest) = trim(url).split_once("://")?;
    let (host, prefix) = rest.split_once('/').unwrap_or((rest, ""));
    Some((host.to_owned(), prefix.to_owned()))
}

fn project_from(path: &str, prefix: &str) -> Option<String> {
    let path = path.trim_matches('/');
    let path = if prefix.is_empty() {
        path
    } else {
        path.strip_prefix(prefix)?.trim_start_matches('/')
    };
    let p = path
        .strip_suffix(".git")
        .unwrap_or(path)
        .trim_end_matches('/');
    (!p.is_empty() && !p.split('/').any(|s| s.is_empty() || s == "." || s == ".."))
        .then(|| p.into())
}

/// The forge a git URL is on, and the project it names there. Declared forges come first; a
/// declared URL longer than another wins.
pub fn resolve(forges: &[Forge], git: &str) -> Option<Resolved> {
    let (host, path, web) = split_git(git)?;
    let declared = forges.iter().filter_map(|f| Some((f, kind_of(f)?)));
    let known = KNOWN
        .iter()
        .filter(|(u, _)| !forges.iter().any(|f| trim(&f.url) == *u))
        .map(|(u, k)| {
            (
                Forge {
                    url: (*u).into(),
                    kind: None,
                    api: None,
                    token: None,
                    projects: None,
                },
                *k,
            )
        });
    let mut all: Vec<(Forge, Kind)> = declared.map(|(f, k)| (f.clone(), k)).chain(known).collect();
    all.sort_by_key(|(f, _)| std::cmp::Reverse(trim(&f.url).len()));
    for (f, kind) in all {
        let Some((fhost, prefix)) = web_host_and_prefix(&f.url) else {
            continue;
        };
        // An SSH URL names the host without the web port.
        let same_host = if web {
            fhost.eq_ignore_ascii_case(&host)
        } else {
            fhost
                .split(':')
                .next()
                .unwrap_or("")
                .eq_ignore_ascii_case(host.split(':').next().unwrap_or(""))
        };
        if !same_host {
            continue;
        }
        let Some(project) = project_from(&path, &prefix) else {
            continue;
        };
        return Some(Resolved {
            kind,
            api: f.api.clone().unwrap_or_else(|| default_api(kind, &f.url)),
            token: f
                .token
                .clone()
                .unwrap_or_else(|| default_token(kind).into()),
            projects: f.projects.clone(),
            url: trim(&f.url).into(),
            project,
        });
    }
    None
}

/// A project glob: `*` matches within one path segment, so `acme/*` matches `acme/pay` but not
/// `acme/platform/pay`.
pub fn path_glob(pattern: &str, path: &str) -> bool {
    let (ps, ss): (Vec<&str>, Vec<&str>) =
        (pattern.split('/').collect(), path.split('/').collect());
    ps.len() == ss.len() && ps.iter().zip(&ss).all(|(p, s)| segment(p, s))
}

fn segment(p: &str, s: &str) -> bool {
    match p.split_once('*') {
        None => p == s,
        Some((head, rest)) => {
            s.starts_with(head)
                && (head.len()..=s.len()).any(|i| s.is_char_boundary(i) && segment(rest, &s[i..]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forge(url: &str, kind: Option<&str>) -> Forge {
        Forge {
            url: url.into(),
            kind: kind.map(str::to_owned),
            api: None,
            token: None,
            projects: None,
        }
    }

    #[test]
    fn known_hosts_need_no_declaration() {
        let r = resolve(&[], "https://github.com/acme/web.git").unwrap();
        assert_eq!(
            (r.kind, r.api.as_str(), r.token.as_str(), r.project.as_str()),
            (
                Kind::Github,
                "https://api.github.com",
                "GITHUB_TOKEN",
                "acme/web"
            )
        );
        let r = resolve(&[], "git@gitlab.com:acme/platform/pay.git").unwrap();
        assert_eq!(
            (r.kind, r.api.as_str(), r.token.as_str(), r.project.as_str()),
            (
                Kind::Gitlab,
                "https://gitlab.com/api/v4",
                "GITLAB_TOKEN",
                "acme/platform/pay"
            )
        );
        assert!(resolve(&[], "/srv/git/tools.git").is_none());
        assert!(resolve(&[], "https://git.example/acme/x.git").is_none());
    }

    #[test]
    fn a_declared_forge_is_matched_by_url_or_ssh_host() {
        let fs = [
            forge("https://gitlab.acme.dev", Some("gitlab")),
            forge("http://127.0.0.1:8929", Some("gitlab")),
            Forge {
                token: Some("ACME_GHES".into()),
                ..forge("https://ghe.acme.dev", Some("github"))
            },
        ];
        let r = resolve(&fs, "ssh://git@gitlab.acme.dev:2222/acme/pay.git").unwrap();
        assert_eq!((r.kind, r.project.as_str()), (Kind::Gitlab, "acme/pay"));
        assert_eq!(r.api, "https://gitlab.acme.dev/api/v4");
        let r = resolve(&fs, "http://root:tok@127.0.0.1:8929/acme/a.git").unwrap();
        assert_eq!(r.project, "acme/a");
        assert!(resolve(&fs, "http://127.0.0.1:9999/acme/a.git").is_none());
        let r = resolve(&fs, "https://ghe.acme.dev/platform/ledger").unwrap();
        assert_eq!(
            (r.api.as_str(), r.token.as_str()),
            ("https://ghe.acme.dev/api/v3", "ACME_GHES")
        );
    }

    #[test]
    fn a_forge_under_a_path_and_a_declared_known_host() {
        let fs = [
            forge("https://code.acme.dev/gitlab", Some("gitlab")),
            Forge {
                token: Some("ORG_TOKEN".into()),
                ..forge("https://github.com/", None)
            },
        ];
        let r = resolve(&fs, "https://code.acme.dev/gitlab/acme/pay.git").unwrap();
        assert_eq!(r.project, "acme/pay");
        assert!(resolve(&fs, "https://code.acme.dev/other/acme/pay.git").is_none());
        assert_eq!(
            resolve(&fs, "https://github.com/a/b").unwrap().token,
            "ORG_TOKEN"
        );
        assert!(resolve(&fs, "https://github.com/a/../b").is_none());
    }

    #[test]
    fn the_hybrid_example_routes_each_child_to_its_own_forge() {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/federated-actions/hybrid-org"
        );
        let src = crate::load::load(std::path::Path::new(dir)).unwrap();
        let m: crate::manifest::Manifest = serde_json::from_value(src.document).unwrap();
        let routes: Vec<_> = m
            .federation
            .unwrap()
            .children
            .iter()
            .map(|c| {
                resolve(&m.forges, &c.git).map(|r| {
                    let admits = r.admits();
                    (r.kind, r.api, r.token, r.project, admits)
                })
            })
            .collect();
        let r = |k, api: &str, token: &str, project: &str, admits| {
            Some((
                k,
                api.to_owned(),
                token.to_owned(),
                project.to_owned(),
                admits,
            ))
        };
        assert_eq!(
            routes,
            [
                r(
                    Kind::Github,
                    "https://api.github.com",
                    "GITHUB_TOKEN",
                    "acme/web",
                    true
                ),
                r(
                    Kind::Github,
                    "https://ghe.acme.dev/api/v3",
                    "ACME_GHES_TOKEN",
                    "platform/ledger",
                    true
                ),
                r(
                    Kind::Gitlab,
                    "https://gitlab.com/api/v4",
                    "GITLAB_TOKEN",
                    "acme/mobile",
                    true
                ),
                r(
                    Kind::Gitlab,
                    "https://gitlab.acme.dev/api/v4",
                    "ACME_GITLAB_TOKEN",
                    "acme/platform/pay",
                    true
                ),
                r(
                    Kind::Gitlab,
                    "https://gitlab.acme.dev/api/v4",
                    "ACME_GITLAB_TOKEN",
                    "acme/tools",
                    true
                ),
                None,
            ]
        );
    }

    #[test]
    fn project_globs_stay_within_a_segment() {
        assert!(path_glob("acme/*", "acme/pay"));
        assert!(!path_glob("acme/*", "acme/platform/pay"));
        assert!(path_glob("acme/*/*", "acme/platform/pay"));
        assert!(path_glob("acme/pay-*", "acme/pay-svc"));
        assert!(!path_glob("acme/pay-*", "acme/web"));
    }
}
