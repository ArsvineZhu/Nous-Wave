//! Immutable material/evidence registration and explicitly invoked derivation.

mod derivation;
mod materialization;
mod observation;
mod types;
pub use materialization::{ByteRange, MaterializeRequest, MaterializedEvidence};
pub use types::*;

use chrono::{DateTime, Utc};
use futures::Stream;
use nous_authority_store::{AuthorityStore, ProjectionInvalidation};
use nous_cognitive_runtime::CognitiveRuntimeService;
use nous_core::*;
use nous_material::*;
use nous_object_store::ObjectStore;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct MaterialService {
    pub store: AuthorityStore,
    pub objects: ObjectStore,
    pub cognition: CognitiveRuntimeService,
    pub max_upload_bytes: u64,
}

impl MaterialService {
    pub fn new(
        store: AuthorityStore,
        objects: ObjectStore,
        cognition: CognitiveRuntimeService,
        max_upload_bytes: u64,
    ) -> Result<Self> {
        if max_upload_bytes == 0 {
            return Err(Error::Invalid("max_upload_bytes must be positive".into()));
        }
        Ok(Self {
            store,
            objects,
            cognition,
            max_upload_bytes,
        })
    }
}
