//! Typed view of the manifest (`apiVersion: toolbox.dev/v1alpha1`). Only built after the document
//! has passed the JSON Schema, so shapes are guaranteed; these types add names to the data.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub api_version: String,
    pub portal: Portal,
    #[serde(default)]
    pub policies: Map<String, Value>,
    #[serde(default)]
    pub connectors: Vec<Connector>,
    pub kinds: Vec<Kind>,
    #[serde(default)]
    pub scorecards: Vec<Scorecard>,
    /// Long-term focus programs that group scorecards and campaigns.
    #[serde(default)]
    pub tracks: Vec<Track>,
    /// Named groups of kinds, for readers: reports list the catalog group by group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kind_groups: Vec<KindGroup>,
    /// Time-bound goals over existing scorecards (D01M7Y11G0YZT23W26Z9D3XW68Y).
    #[serde(default)]
    pub campaigns: Vec<Campaign>,
    /// Accepted failures. Each one cites a decision and names one rule on one entity.
    #[serde(default)]
    pub waivers: Vec<Waiver>,
    /// Rubrics coding agents assess entities with (assessments are data, read by scorecards).
    #[serde(default)]
    pub rubrics: Vec<Rubric>,
    /// Upstream tools the portal needs (`toolbox doctor` checks them; D90).
    #[serde(default)]
    pub tools: BTreeMap<String, Tool>,
    pub authz: Option<Authz>,
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Federated actions: child portals' actions offered here, and how a parent reaches this one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub federation: Option<Federation>,
    /// The code hosts this portal reaches beyond plain git (`crate::forge`, docs/design/forges.md).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forges: Vec<Forge>,
    /// The CI systems this portal starts runs on, apart from the forges that host the code
    /// (`crate::ci`, docs/design/forges.md §4.7).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ci: Vec<CiHost>,
    pub ui: Option<Ui>,
    pub mcp: Option<Value>,
    #[serde(default)]
    pub tests: Vec<String>,
}

/// `federation:` (docs/design/federated-actions.md §5).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Federation {
    /// The kind whose entities are child portals (workspace `portal` records).
    pub portals: Option<String>,
    /// A parent's child portals by git repository, kept at their branch (`crate::federation`).
    #[serde(default)]
    pub children: Vec<GitChild>,
    /// The relation a caller needs on a child portal entity (default `can_run_actions`).
    pub permission: Option<String>,
    /// How a parent may dispatch this portal's actions: `{type: local}` or `{type: github-workflow, …}`.
    pub runner: Option<Value>,
    /// A parent's runners: `local.portals` (children a server may run in-process) and
    /// `github-workflow` (`repos`, `token`, `api`). The parent owns credentials, never the child.
    pub runners: Option<Value>,
    /// A parent's environment variable holding its approval signing key (64 hex characters).
    pub signing_key: Option<String>,
    /// Parent portals whose signed approvals this portal accepts on a dispatched run.
    #[serde(default)]
    pub accept_approvals_from: Vec<TrustedParent>,
}

/// One entry of `forges:`. Unset fields take the forge type's defaults (`crate::forge`).
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct Forge {
    pub url: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub api: Option<String>,
    /// The environment variable holding the API token.
    pub token: Option<String>,
    /// Project paths (globs) a parent may dispatch CI to; default: the children it lists.
    pub projects: Option<Vec<String>>,
    /// A checkout of this portal takes who you are from your own token on this forge, not from
    /// `--as` (`toolbox run --identity-token-file`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub identity: bool,
    /// The `ci:` host that runs `runner: {type: ci}` children on this forge, or `none`; default:
    /// the forge's own CI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci: Option<String>,
}

/// One entry of `ci:`: a CI system, named so forges and executors can point at it.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub struct CiHost {
    pub name: String,
    /// `github-actions`, `gitlab-ci` or `http` (an executor-protocol service).
    #[serde(rename = "type")]
    pub kind: String,
    /// The web URL (`github-actions`, `gitlab-ci`; default the public host), or the service's
    /// endpoint (`http`, required).
    pub url: Option<String>,
    pub api: Option<String>,
    /// The environment variable holding the token.
    pub token: Option<String>,
    /// Where a forge project's CI runs here; `{project}` is the project's path. Default
    /// `{project}`.
    pub project: Option<String>,
    /// Projects (globs, after mapping) a parent may start runs in; default any.
    pub projects: Option<Vec<String>>,
}

/// One entry of `federation.children`, with the defaults `crate::federation` fills in.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GitChild {
    pub git: String,
    pub branch: Option<String>,
    pub key: Option<String>,
}

/// A parent whose approvals a child accepts, pinned by its signing key.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TrustedParent {
    pub portal: String,
    /// `ed25519:<64 hex>`.
    pub key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Portal {
    pub name: String,
    pub title: Option<String>,
    pub owner: String,
    pub toolbox: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Connector {
    pub name: String,
    pub package: String,
    pub runtime: String,
    pub config: Option<Value>,
    pub records: Vec<String>,
    #[serde(default)]
    pub refresh: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub freshness: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Kind {
    pub name: String,
    /// The name readers see; reports fall back to the kind name split into words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// What the kind is, for people and agents.
    pub description: Option<String>,
    pub identity_from: Vec<String>,
    pub fields: BTreeMap<String, Field>,
    #[serde(default)]
    pub relations: BTreeMap<String, Relation>,
    pub sources: Sources,
    #[serde(default)]
    pub write_back: BTreeMap<String, String>,
    #[serde(default)]
    pub map: Vec<MapRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Field {
    #[serde(rename = "type")]
    pub ty: String,
    pub values: Option<Vec<String>>,
    pub to: Option<String>,
    pub items: Option<String>,
    pub facet: Option<String>,
    pub computed: Option<String>,
    pub authority: Option<Authority>,
    /// What the field means, for people and agents.
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Authority {
    #[serde(rename = "type")]
    pub ty: String,
    pub claim: String,
    pub rules: String,
    pub path: String,
    pub team_field: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relation {
    pub to: String,
    #[serde(default)]
    pub many: bool,
    /// `report` (default) or `ignore`: whether dropped edges (no such target) are a diagnostic.
    pub unresolved: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sources {
    pub default: Vec<String>,
    #[serde(default)]
    pub fields: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapRule {
    pub from: String,
    pub source: Option<String>,
    pub id: String,
    pub set: String,
    #[serde(default)]
    pub relations: BTreeMap<String, String>,
}

impl MapRule {
    /// `(connector, record kind)` from `from: <connector>/<record kind>`.
    pub fn from_parts(&self) -> (&str, &str) {
        self.from
            .split_once('/')
            .unwrap_or((self.from.as_str(), ""))
    }
    /// The source label fragments from this rule carry (default `connector:<connector>`).
    pub fn source_label(&self) -> String {
        self.source
            .clone()
            .unwrap_or_else(|| format!("connector:{}", self.from_parts().0))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scorecard {
    pub name: String,
    /// A heading for people and agents (reports show it next to the name).
    pub title: Option<String>,
    /// What the standard is and why: shown by reports and `describe`.
    pub description: Option<String>,
    pub kind: String,
    pub owner: String,
    /// Optional track this scorecard belongs to (`tracks[].name`).
    pub track: Option<String>,
    pub levels: Vec<String>,
    pub applies_to: Option<String>,
    /// Why the standard matters: the outcome it protects, for readers deciding whether to care.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// Further reading: the decision, design doc or outside source behind it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
    /// A comparative scorecard: the subject against its peers, rule by rule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare: Option<Compare>,
    pub rules: Vec<Rule>,
    /// `{newEntitiesAtLeast: <level>}`: ratchet policy for entities new since the baseline.
    pub ratchet: Option<Value>,
    pub facets: Option<Value>,
}

/// Who a comparative scorecard compares: one subject, scored beside each peer.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Compare {
    pub subject: String,
    pub peers: Vec<String>,
}

/// A long-term focus program (Soundcheck "track").
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Track {
    pub name: String,
    pub title: Option<String>,
    pub owner: String,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

/// A titled URL: where to read more about a standard, rule or track.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Link {
    pub title: String,
    pub url: String,
}

/// Kinds that readers think of together (the docs, the code, the CI), listed as one group.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KindGroup {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub kinds: Vec<String>,
}

/// One scorecard goal inside a campaign.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignInclude {
    pub scorecard: String,
    pub at_least: Option<String>,
    #[serde(default)]
    pub rules: Vec<String>,
}

/// A checkpoint on the way to a campaign's `targetPercent`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignMilestone {
    pub name: String,
    pub target_percent: u8,
}

/// A time-bound goal over existing scorecards (Soundcheck "campaign" / Cortex "initiative").
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Campaign {
    pub name: String,
    pub title: Option<String>,
    pub track: Option<String>,
    pub owner: String,
    pub description: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub target_percent: Option<u8>,
    #[serde(default)]
    pub draft: bool,
    /// Optional lifecycle override (`draft` | `scheduled` | `active` | `ended`).
    pub status: Option<String>,
    #[serde(default)]
    pub milestones: Vec<CampaignMilestone>,
    pub includes: Vec<CampaignInclude>,
}

impl Campaign {
    /// Fleet goal percent (default 100).
    pub fn target_percent(&self) -> u8 {
        self.target_percent.unwrap_or(100)
    }
}

/// An upstream tool: the version required and how mise installs it.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub version: String,
    /// The mise tool id (kept; `install.mise` is the same).
    pub mise: Option<String>,
    pub version_args: Option<Vec<String>>,
    /// The tool's id for each installer `toolbox tools install` can run: `mise`, `aqua`, `pypi`.
    pub install: Option<serde_json::Map<String, Value>>,
}

/// A rubric: the questions an agent answers when it assesses an entity of `kind`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rubric {
    pub name: String,
    pub kind: String,
    pub description: Option<String>,
    /// The entity field that versions what is assessed (default `hash`): an assessment whose
    /// `subjectHash` differs from it is stale.
    pub subject: Option<String>,
    pub criteria: Vec<Criterion>,
}

impl Rubric {
    /// The field assessments are versioned by (`subject`, default `hash`).
    pub fn subject(&self) -> &str {
        self.subject.as_deref().unwrap_or("hash")
    }
}

/// One question of a rubric, with optional guidance on how to answer it.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Criterion {
    pub name: String,
    pub question: String,
    pub guidance: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub name: String,
    /// A short human name; reports spell out `name` when it is missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub level: String,
    pub check: String,
    /// CEL evaluated when the rule fails: what makes it fail (usually a list of items).
    pub evidence: Option<String>,
    /// CEL evaluated when the rule fails: a number the ratchet compares instead of the evidence
    /// count (lines in a file, occurrences of a pattern). Larger is worse.
    pub measure: Option<String>,
    /// CEL evaluated for every in-scope entity: the number a comparison compares.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// `lower` or `higher`: which way `value` improves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub better: Option<String>,
    /// The unit of `value`, for reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Relative difference within which two values are at parity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<f64>,
    pub fix: Option<String>,
    /// Why the rule matters, beyond what it checks and how to fix it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    /// Further reading behind the rule.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
    /// Connector whose records this rule reads. When that connector is skipped, a missing
    /// value is an expected `unknown` and the ratchet does not refuse `--ci` for it.
    #[serde(default, rename = "dataFrom")]
    pub data_from: Option<String>,
}

/// An accepted failure. The rule stays visible as `waived` and counts as a pass for the level.
/// Removing the waiver makes a still-failing rule a regression. Closing the gap, so the rule
/// passes without the waiver, is a new decision, not an edit of the one this cites.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Waiver {
    pub entity: String,
    pub scorecard: String,
    pub rule: String,
    /// A decision file id (`docs/decisions/<id>.md`).
    pub decision: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Authz {
    pub model: String,
    #[serde(default)]
    pub tests: Vec<String>,
    pub tuples: Vec<TupleDerivation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TupleDerivation {
    Static {
        #[serde(rename = "static")]
        tuples: Vec<Value>,
    },
    Kind {
        kind: String,
        emit: String,
    },
}

fn day2() -> String {
    "day2".into()
}

fn can_run_actions() -> String {
    "can_run_actions".into()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Action {
    pub name: String,
    pub kind: String,
    #[serde(default = "day2")]
    pub scope: String,
    pub description: String,
    pub inputs: Value,
    #[serde(default = "can_run_actions")]
    pub permission: String,
    pub approval: Option<String>,
    #[serde(default)]
    pub context: BTreeMap<String, ContextLookup>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    pub executor: Value,
    pub expect: Option<Expect>,
    pub unverified: Option<bool>,
    pub annotations: Option<Value>,
    pub mcp: Option<Value>,
    pub ui: Option<ActionUi>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextLookup {
    pub kind: String,
    #[serde(rename = "where")]
    pub where_: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Expect {
    pub postcondition: String,
    #[serde(default)]
    pub refresh: Vec<Refresh>,
    pub within: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Refresh {
    pub connector: String,
    pub kind: String,
    pub selector: Option<String>,
    pub ids: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActionUi {
    pub bundle: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ui {
    #[serde(default)]
    pub bundles: Vec<UiBundle>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiBundle {
    pub name: String,
    pub path: String,
    pub entry: String,
    pub framework: Option<String>,
    pub mounts: Vec<Value>,
}
