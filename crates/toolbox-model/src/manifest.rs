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
    pub kind: String,
    pub owner: String,
    pub levels: Vec<String>,
    pub applies_to: Option<String>,
    pub rules: Vec<Rule>,
    pub facets: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub name: String,
    pub level: String,
    pub check: String,
    pub fix: Option<String>,
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
