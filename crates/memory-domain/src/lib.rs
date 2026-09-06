use chrono::{DateTime, Utc};
use nous_core::SubjectId;
use nous_material::{Classification, TemporalRange};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod learning;
pub mod recall;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemoryKind {
    Episode,
    Episodic,
    Semantic,
    Procedural,
    Conceptual,
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryObject {
    pub object_id: Uuid,
    pub subject_id: SubjectId,
    pub kind: MemoryKind,
    pub scope: String,
    pub current_revision: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRevision {
    pub revision_id: Uuid,
    pub object_id: Uuid,
    pub title: String,
    pub representation_artifacts: Vec<Uuid>,
    pub source_refs: Vec<Uuid>,
    pub derivation_refs: Vec<Uuid>,
    pub classification: Classification,
    pub occurred: Option<TemporalRange>,
    pub observed_at: Option<DateTime<Utc>>,
    pub recorded_at: DateTime<Utc>,
    pub content_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RevisionRelation {
    Correction,
    WorldEvolution,
    Consolidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionParent {
    pub revision_id: Uuid,
    pub parent_revision_id: Uuid,
    pub relation: RevisionRelation,
    pub reason: String,
    pub supporting_sources: Vec<Uuid>,
}
