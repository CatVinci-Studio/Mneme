use serde::{Deserialize, Serialize};

pub type EntityKind = String;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub retriever: Option<String>,
    pub sync_remote: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPatch {
    pub provider: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub retriever: Option<String>,
    pub sync_remote: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigView {
    pub provider: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub has_key: bool,
    pub providers: Vec<String>,
    pub retriever: String,
    pub sync_remote: Option<String>,
    pub vault: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    pub id: String,
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub fetched_at: String,
    pub content_hash: String,
    pub word_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summarized: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceDetail {
    pub meta: SourceMeta,
    pub content: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AddSourceInput {
    pub url: Option<String>,
    pub text: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub kind: EntityKind,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateEntity {
    pub name: String,
    pub kind: EntityKind,
    #[serde(default)]
    pub why: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub tldr: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    #[serde(default)]
    pub candidate_entities: Vec<CandidateEntity>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractClaim {
    pub entity_name: String,
    pub text: String,
    pub span: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}
fn default_confidence() -> f64 {
    0.8
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractResult {
    #[serde(default)]
    pub entities: Vec<Entity>,
    #[serde(default)]
    pub claims: Vec<ExtractClaim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub span: String,
    pub source_id: String,
    pub char_start: usize,
    pub char_end: usize,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source_id: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactEntry {
    pub id: String,
    pub text: String,
    pub prov: Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub text: String,
    pub prov: Provenance,
    pub superseded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntry {
    pub target_slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPage {
    pub slug: String,
    pub title: String,
    pub kind: EntityKind,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    pub updated: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub facts: Vec<FactEntry>,
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
    #[serde(default)]
    pub contradictions: Vec<Contradiction>,
    #[serde(default)]
    pub links: Vec<LinkEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntitySummary {
    pub slug: String,
    pub title: String,
    pub kind: String,
    pub summary: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityDetail {
    pub page: EntityPage,
    pub backlinks: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub slug: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    pub slug: String,
    pub title: String,
    pub kind: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WikifyEntityReport {
    pub slug: String,
    pub ops: usize,
    pub result: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct WikifyReport {
    pub source_id: String,
    pub entities: Vec<WikifyEntityReport>,
}
#[derive(Debug, Clone, Serialize)]
pub struct AddSourceResult {
    pub source_id: String,
    pub meta: SourceMeta,
    pub report: WikifyReport,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResearchSource {
    pub slug: String,
    pub title: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct ResearchResult {
    pub answer: String,
    pub sources: Vec<ResearchSource>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintReport {
    pub entities: usize,
    pub sources: usize,
    pub facts: usize,
    pub orphans: Vec<String>,
    pub empty_facts: Vec<String>,
    pub contradictions: Vec<LintContradiction>,
}
#[derive(Debug, Clone, Serialize)]
pub struct LintContradiction {
    pub slug: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct RelatedFact {
    pub slug: String,
    pub fact_id: String,
    pub text: String,
}
#[derive(Debug, Clone)]
pub struct ReconcileInput {
    pub slug: String,
    pub title: String,
    pub kind: String,
    pub exists: bool,
    pub current_facts: Vec<(String, String)>,
    pub new_claims: Vec<Claim>,
    pub related_facts: Vec<RelatedFact>,
}

#[derive(Debug, Clone)]
pub enum PatchOp {
    CreatePage {
        entity: Entity,
        summary: String,
    },
    AppendFact {
        fact: String,
        claim: Claim,
    },
    UpdateFact {
        fact_id: String,
        new_fact: String,
        claim: Claim,
        target_slug: Option<String>,
    },
    SupersedeFact {
        fact_id: String,
        new_fact: String,
        claim: Claim,
        target_slug: Option<String>,
    },
    FlagContradiction {
        fact_id: String,
        conflicting: Claim,
        note: String,
        target_slug: Option<String>,
    },
    AddLink {
        target_slug: String,
        relation: Option<String>,
    },
}
