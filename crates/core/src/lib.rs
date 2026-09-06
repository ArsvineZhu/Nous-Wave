use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const API_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubjectId(pub Uuid);

impl Default for SubjectId {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub subject_id: SubjectId,
    pub label: Option<String>,
    pub metadata: serde_json::Value,
    pub revision: i64,
    pub seed_revision: Uuid,
    pub memory_enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedRevision {
    pub revision_id: Uuid,
    pub subject_id: SubjectId,
    pub artifact_id: Uuid,
    pub authored_by: String,
    pub parent_revision: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Readiness {
    Ready,
    Degraded,
    Unavailable,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStatus {
    pub capability_id: String,
    pub status: Readiness,
    pub reason: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    Infrastructure(String),
}

pub type Result<T> = std::result::Result<T, Error>;
