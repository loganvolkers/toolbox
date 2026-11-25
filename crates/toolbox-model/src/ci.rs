//! CI hosts (docs/design/forges.md §4.7): the systems a portal starts runs on, declared apart from
//! the forges that host the code. A forge's projects build on the forge's own CI unless the
//! forge's entry names a `ci:` host, or `none`. An executor may name a host too, for its endpoint
//! and token. A child portal names neither: which CI runs it, and with whose token, is the
//! parent's to say.

use crate::forge::{self, Kind, Resolved, path_glob};
use crate::manifest::{CiHost, Forge, Manifest};

/// `forges[].ci: none`: this forge's children run no CI for a parent.
pub const NONE: &str = "none";

/// A CI host type: the driver that starts, follows and reads back its runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiType {
    GithubActions,
    GitlabCi,
    /// A service that speaks the executor protocol over HTTP and bridges to any CI.
    Http,
}

impl CiType {
    /// The type a manifest's `type:` names.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "github-actions" => Some(Self::GithubActions),
            "gitlab-ci" => Some(Self::GitlabCi),
            "http" => Some(Self::Http),
            _ => None,
        }
    }

    /// The name `parse` accepts.
    pub fn name(self) -> &'static str {
        match self {
            Self::GithubActions => "github-actions",
            Self::GitlabCi => "gitlab-ci",
            Self::Http => "http",
        }
    }

    /// A forge's own CI.
    pub fn of_forge(kind: Kind) -> Self {
        match kind {
            Kind::Github => Self::GithubActions,
            Kind::Gitlab => Self::GitlabCi,
        }
    }

    /// The forge whose API this CI is part of, if any.
    pub fn forge(self) -> Option<Kind> {
        match self {
            Self::GithubActions => Some(Kind::Github),
            Self::GitlabCi => Some(Kind::Gitlab),
            Self::Http => None,
        }
    }
}

/// Every host a portal reaches: the forges its code is on and the CI hosts it starts runs on.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hosts {
    pub forges: Vec<Forge>,
    pub ci: Vec<CiHost>,
}

impl Hosts {
    /// A manifest's `forges:` and `ci:`.
    pub fn of(m: &Manifest) -> Self {
        Self {
            forges: m.forges.clone(),
            ci: m.ci.clone(),
        }
    }

    /// The `ci:` host called `name`.
    pub fn host(&self, name: &str) -> Option<&CiHost> {
        self.ci.iter().find(|h| h.name == name)
    }
}

/// Where one project's runs start, with the endpoint and token to start them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The `ci:` host's name; `None` for a forge's own CI.
    pub host: Option<String>,
    pub ty: CiType,
    /// The API base (`github-actions`, `gitlab-ci`) or the service's endpoint (`http`).
    pub api: String,
    /// The environment variable holding the token; `None` for a service that takes none.
    pub token: Option<String>,
    /// The project on the CI host.
    pub project: String,
    /// `None`: any project.
    pub projects: Option<Vec<String>>,
}

impl Target {
    /// Whether the host's `projects` admits this project.
    pub fn admits(&self) -> bool {
        self.projects
            .as_ref()
            .is_none_or(|ps| ps.iter().any(|p| path_glob(p, &self.project)))
    }
}

/// Where a forge project's CI runs on host `h`: its `project` template with `{project}` filled in.
pub fn map(h: &CiHost, project: &str) -> String {
    h.project
        .as_deref()
        .unwrap_or("{project}")
        .replace("{project}", project)
}

/// Host `h`'s endpoint and token for `project`, already mapped.
pub fn target(h: &CiHost, project: &str) -> Result<Target, String> {
    let ty =
        CiType::parse(&h.kind).ok_or_else(|| format!("ci {}: unknown type {}", h.name, h.kind))?;
    let (api, token) = match ty.forge() {
        Some(k) => {
            let url = h
                .url
                .clone()
                .or_else(|| forge::known_host(k).map(str::to_owned))
                .unwrap_or_default();
            (
                h.api.clone().unwrap_or_else(|| forge::default_api(k, &url)),
                Some(
                    h.token
                        .clone()
                        .unwrap_or_else(|| forge::default_token(k).into()),
                ),
            )
        }
        None => (
            h.url
                .clone()
                .ok_or_else(|| format!("ci {}: an http CI host needs url", h.name))?,
            h.token.clone(),
        ),
    };
    Ok(Target {
        host: Some(h.name.clone()),
        ty,
        api,
        token,
        project: project.into(),
        projects: h.projects.clone(),
    })
}

/// Where a project on forge `f` runs its CI: the host the forge's entry names, its own CI, or
/// `Ok(None)` for `ci: none`.
pub fn for_forge(hosts: &[CiHost], f: &Resolved) -> Result<Option<Target>, String> {
    match f.ci.as_deref() {
        None => Ok(Some(Target {
            host: None,
            ty: CiType::of_forge(f.kind),
            api: f.api.clone(),
            token: Some(f.token.clone()),
            project: f.project.clone(),
            projects: f.projects.clone(),
        })),
        Some(NONE) => Ok(None),
        Some(name) => {
            let h = hosts.iter().find(|h| h.name == name).ok_or_else(|| {
                format!(
                    "forge {} runs CI on {name}, which ci: does not declare",
                    f.url
                )
            })?;
            target(h, &map(h, &f.project)).map(Some)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(name: &str, kind: &str) -> CiHost {
        CiHost {
            name: name.into(),
            kind: kind.into(),
            ..CiHost::default()
        }
    }

    fn on(ci: Option<&str>) -> Resolved {
        let fs = [Forge {
            url: "https://github.com".into(),
            ci: ci.map(str::to_owned),
            ..Forge::default()
        }];
        forge::resolve(&fs, "https://github.com/acme/web.git").unwrap()
    }

    #[test]
    fn a_forge_runs_its_own_ci_unless_its_entry_names_a_host() {
        let t = for_forge(&[], &on(None)).unwrap().unwrap();
        assert_eq!(
            (t.host, t.ty, t.api.as_str(), t.token.as_deref()),
            (
                None,
                CiType::GithubActions,
                "https://api.github.com",
                Some("GITHUB_TOKEN")
            )
        );
        assert_eq!(for_forge(&[], &on(Some(NONE))).unwrap(), None);
        let e = for_forge(&[], &on(Some("buildkite"))).unwrap_err();
        assert!(e.contains("ci: does not declare"), "{e}");
    }

    #[test]
    fn a_declared_host_maps_the_project_and_gates_it() {
        let hosts = [
            CiHost {
                url: Some("https://gitlab.acme.dev".into()),
                token: Some("ACME_GITLAB_TOKEN".into()),
                project: Some("mirrors/{project}".into()),
                projects: Some(vec!["mirrors/acme/*".into()]),
                ..host("mirror", "gitlab-ci")
            },
            CiHost {
                url: Some("https://bk.acme.dev/toolbox".into()),
                ..host("buildkite", "http")
            },
        ];
        let t = for_forge(&hosts, &on(Some("mirror"))).unwrap().unwrap();
        assert_eq!(
            (t.ty, t.api.as_str(), t.token.as_deref(), t.project.as_str()),
            (
                CiType::GitlabCi,
                "https://gitlab.acme.dev/api/v4",
                Some("ACME_GITLAB_TOKEN"),
                "mirrors/acme/web"
            )
        );
        assert!(t.admits());
        let t = for_forge(&hosts, &on(Some("buildkite"))).unwrap().unwrap();
        assert_eq!(
            (t.ty, t.api.as_str(), t.token, t.project.as_str()),
            (
                CiType::Http,
                "https://bk.acme.dev/toolbox",
                None,
                "acme/web"
            )
        );
        let gated = Target {
            project: "acme/web".into(),
            ..for_forge(&hosts, &on(Some("mirror"))).unwrap().unwrap()
        };
        assert!(!gated.admits());
        assert!(target(&host("bk", "http"), "x").is_err());
    }

    #[test]
    fn the_hybrid_example_builds_the_ghes_children_on_buildkite() {
        let dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/federated-actions/hybrid-org"
        );
        let src = crate::load::load(std::path::Path::new(dir)).unwrap();
        let m: Manifest = serde_json::from_value(src.document).unwrap();
        let routes: Vec<_> = m
            .federation
            .as_ref()
            .unwrap()
            .children
            .iter()
            .filter_map(|c| forge::resolve(&m.forges, &c.git))
            .map(|f| {
                let t = for_forge(&m.ci, &f).unwrap().unwrap();
                (f.project, t.ty, t.host, t.api)
            })
            .collect();
        let own = |p: &str, ty, api: &str| (p.to_owned(), ty, None, api.to_owned());
        assert_eq!(
            routes,
            [
                own("acme/web", CiType::GithubActions, "https://api.github.com"),
                (
                    "platform/ledger".into(),
                    CiType::Http,
                    Some("buildkite".into()),
                    "https://buildkite-bridge.acme.dev/toolbox".into()
                ),
                own("acme/mobile", CiType::GitlabCi, "https://gitlab.com/api/v4"),
                own(
                    "acme/platform/pay",
                    CiType::GitlabCi,
                    "https://gitlab.acme.dev/api/v4"
                ),
                own(
                    "acme/tools",
                    CiType::GitlabCi,
                    "https://gitlab.acme.dev/api/v4"
                ),
            ]
        );
    }
}
