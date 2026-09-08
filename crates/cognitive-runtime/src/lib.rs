//! Model-independent Session continuity, consumer context and query orchestration.

mod sessions;
mod types;
mod use_feedback;
mod resources;
mod query;
mod working_set;
pub use working_set::*;
pub use query::{CognitiveContributor, QueryPlan, WorkCycle};

pub use types::*;
pub use resources::*;

use chrono::{DateTime, Utc};
use nous_authority_store::AuthorityStore;
use nous_core::*;
use std::{collections::{HashMap, HashSet}, sync::{Arc, RwLock}};
use uuid::Uuid;

#[derive(Clone)]
pub struct CognitiveRuntimeService {
    pub store: AuthorityStore,
    pub resident_limit: usize,
    resource_resolvers: Arc<RwLock<HashMap<String, Arc<dyn ResourceResolver>>>>,
}

impl CognitiveRuntimeService {
    pub fn new(store: AuthorityStore, resident_limit: usize) -> Result<Self> {
        if resident_limit == 0 { return Err(Error::Invalid("resident_limit must be positive".into())); }
        Ok(Self { store, resident_limit, resource_resolvers: Arc::new(RwLock::new(HashMap::new())) })
    }

    pub fn with_resource_resolver(self, key: impl Into<String>, resolver: Arc<dyn ResourceResolver>) -> Self {
        self.resource_resolvers.write().expect("resource resolver composition lock").insert(key.into(), resolver);
        self
    }

    pub async fn require_subject(&self, subject: SubjectId) -> Result<()> {
        self.store.require_subject(subject).await
    }

    pub async fn reference_in_subject(&self, subject: SubjectId, reference: &CognitiveRef) -> Result<bool> {
        self.store.reference_in_subject(subject, reference).await
    }
}
