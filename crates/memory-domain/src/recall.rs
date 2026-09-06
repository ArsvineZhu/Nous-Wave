use chrono::{DateTime, Utc};
use nous_material::{EpistemicClass, TemporalRange};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Memory,
    Entity,
    Source,
    Artifact,
    Relation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeRef {
    pub kind: NodeKind,
    pub id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecallEffort {
    Light,
    Normal,
    Deep,
    Maximum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecallObjective {
    Current,
    Historical,
    Evolution,
    Transition,
    EpisodeReconstruction,
    Explanation,
    Comparison,
    Verification,
    AssociativeRecollection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecallCues {
    pub text: Option<String>,
    #[serde(default)]
    pub entities: Vec<Uuid>,
    #[serde(default)]
    pub references: Vec<NodeRef>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecallConstraints {
    pub scope: Option<String>,
    #[serde(default)]
    pub epistemic: Vec<EpistemicClass>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemporalPerspective {
    pub occurred: Option<TemporalRange>,
    pub observed: Option<TemporalRange>,
    pub known_by: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultNeed {
    pub limit: usize,
    pub include_evidence: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallIntent {
    pub api_version: u32,
    pub cycle_id: Option<Uuid>,
    #[serde(default)]
    pub target: Vec<NodeRef>,
    pub objective: RecallObjective,
    #[serde(default)]
    pub temporal: TemporalPerspective,
    #[serde(default)]
    pub cues: RecallCues,
    #[serde(default)]
    pub constraints: RecallConstraints,
    pub result_need: ResultNeed,
    pub effort: RecallEffort,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecallEffortTrace {
    pub recall_rounds: usize,
    pub candidate_channels: Vec<String>,
    pub index_queries: usize,
    pub candidates_examined: usize,
    pub repeated_candidates: usize,
    pub temporal_expansions: usize,
    pub association_nodes_activated: usize,
    pub edges_visited: usize,
    pub max_graph_depth: usize,
    pub reranker_calls: usize,
    pub evidence_inspections: usize,
    pub model_assisted_calls: usize,
    pub wall_time_ms: f64,
    pub reused_candidates: usize,
}
