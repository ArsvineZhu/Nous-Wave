//! Model-independent Session continuity, consumer context and query orchestration.

mod checkpoints;
mod query;
pub use checkpoints::*;
mod resources;
mod sessions;
mod types;
mod use_feedback;
mod working_set;
pub use query::{CognitiveContributor, QueryPlan, WorkCycle};
pub use working_set::*;

pub use resources::*;
pub use types::*;

use chrono::{DateTime, Utc};
use nous_authority_store::AuthorityStore;
use nous_core::*;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone)]
pub struct CognitiveRuntimeService {
    pub store: AuthorityStore,
    pub resident_limit: usize,
}

impl CognitiveRuntimeService {
    pub fn new(store: AuthorityStore, resident_limit: usize) -> Result<Self> {
        if resident_limit == 0 {
            return Err(Error::Invalid("resident_limit must be positive".into()));
        }
        Ok(Self {
            store,
            resident_limit,
        })
    }

    pub async fn require_subject(&self, subject: SubjectId) -> Result<()> {
        self.store.require_subject(subject).await
    }

    pub async fn reference_in_subject(
        &self,
        subject: SubjectId,
        reference: &CognitiveRef,
    ) -> Result<bool> {
        self.store.reference_in_subject(subject, reference).await
    }
}
