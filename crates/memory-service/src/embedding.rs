use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nous_core::OccurrenceId;
use nous_core::{EntityRef, Result, SourceClass, SubjectId};
use nous_memory_domain::MemoryFormationProposal;
use serde::{Deserialize, Serialize};

/// A narrow proposal seam for explicit Host-selected memory formation. The
/// provider proposes domain values; the service validates and commits them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFormationRequest {
    pub subject: SubjectId,
    pub occurrence: OccurrenceId,
    pub source_class: SourceClass,
    pub text: String,
    pub entity_refs: Vec<EntityRef>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
}

#[async_trait]
pub trait MemoryFormationProvider: Send + Sync {
    async fn propose(
        &self,
        request: MemoryFormationRequest,
    ) -> Result<Vec<MemoryFormationProposal>>;
}
