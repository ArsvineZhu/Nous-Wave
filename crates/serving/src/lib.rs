//! Immutable on-disk serving generations shared by runtime and cognitive contributors.
mod artifacts;
mod build;
mod lifecycle;
mod provider;

pub use provider::*;
use nous_authority_store::{AuthorityStore, ServingRecord};
use nous_core::*;
use nous_memory_retrieval::*;
use nous_object_store::ObjectStore;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServingOptions {
    pub root: PathBuf,
    pub lexical: bool,
    pub dense: bool,
    pub topology: bool,
    pub memory_enabled: bool,
}

#[derive(Clone)]
pub struct ServingService {
    pub store: AuthorityStore,
    pub objects: ObjectStore,
    pub publisher: ServingPublisher,
    pub options: ServingOptions,
    pub embedding: Option<Arc<dyn TextEmbeddingProvider>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectionStatus {
    pub generations: BTreeMap<String, ServingGenerationId>,
    pub rebuilt: Vec<String>,
    pub reopened: Vec<String>,
    pub degradation: Vec<Degradation>,
}

impl ServingService {
    pub fn new(store: AuthorityStore, objects: ObjectStore, options: ServingOptions, embedding: Option<Arc<dyn TextEmbeddingProvider>>) -> Result<Self> {
        std::fs::create_dir_all(&options.root).map_err(artifacts::io)?;
        Ok(Self { store, objects, options, embedding, publisher: ServingPublisher::default() })
    }
}
