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
    pub ui: Option<Ui>,
    pub mcp: Option<Value>,
    #[serde(default)]
    pub tests: Vec<String>,
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
    pub rules: Vec<Rule>,
    /// `{newEntitiesAtLeast: <level>}`: ratchet policy for entities new since the baseline.
    pub ratchet: Option<Value>,
    pub facets: Option<Value>,
}

/// A long-term focus program (Soundcheck "track").
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Track {
    pub name: String,
    pub title: Option<String>,
    pub owner: String,
    pub description: Option<String>,
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
    pub mise: Option<String>,
    pub version_args: Option<Vec<String>>,
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
    pub level: String,
    pub check: String,
    /// CEL evaluated when the rule fails: what makes it fail (usually a list of items).
    pub evidence: Option<String>,
    /// CEL evaluated when the rule fails: a number the ratchet compares instead of the evidence
    /// count (lines in a file, occurrences of a pattern). Larger is worse.
    pub measure: Option<String>,
    pub fix: Option<String>,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Action {
    pub name: String,
    pub kind: String,
    pub scope: String,
    pub description: String,
    pub inputs: Value,
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
