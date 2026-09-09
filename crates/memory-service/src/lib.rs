//! Application orchestration for the first Nous Wave production wave.

mod embedding;
mod support;

use chrono::{DateTime, Utc};
use nous_authority_store::AuthorityStore;
pub use nous_cognitive_runtime::{
    ResidentView, ResourceMaterial, ResourceQuery, ResourceQueryResult, ResourceResolver,
    ResourceUpsert, ResourceView, SessionView, UseFeedback, UseFeedbackEvent,
};
use nous_core::*;
use nous_memory_domain::*;
use nous_memory_retrieval::{
    AssociativeExpansion, BoundedWaveExpansion, CandidateRankInput, CandidateSemanticTrail,
    CandidateTopologyObservation, DenseGeneration, EpaResidualCueSensing, ResidualConfig,
    SeedFamily, SeedOrigin, SemanticCueSensing, TrailOrder, WaveGraphGeneration,
    WeightedCognitiveSeed, bounded_restart_field, observe_epa, rank_candidates,
    trail_topology_observation, wave_observability,
};
use nous_object_store::ObjectStore;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use uuid::Uuid;

pub use embedding::{MemoryFormationProvider, MemoryFormationRequest};
pub use nous_core::{CognitiveQuery, CognitiveQueryResult};
pub use nous_material::{AcceptedObservation, ObservationInput, ObservationMaterial};
use nous_serving::TextEmbeddingRequest;

#[derive(Clone)]
pub struct MemoryService {
    pub store: AuthorityStore,
    pub objects: ObjectStore,
    pub serving: nous_serving::ServingService,
    pub cognition: nous_cognitive_runtime::CognitiveRuntimeService,
    capabilities: Arc<Vec<CapabilityDescriptor>>,
    pub memory_formation_provider: Option<Arc<dyn MemoryFormationProvider>>,
    cue_sensing: Arc<dyn SemanticCueSensing>,
    expansion: Arc<dyn AssociativeExpansion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub api_version: u32,
    pub ready: bool,
    pub authority: String,
    pub capabilities: Vec<CapabilityStatus>,
    pub serving_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryView {
    pub object: MemoryObject,
    pub revision: MemoryRevision,
    pub evidence: Vec<MemoryRevisionEvidence>,
    pub entities: Vec<EntityRef>,
    pub tags: Vec<TagId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub memory: Option<MemoryView>,
    pub topology_changes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviseMemoryInput {
    #[serde(default)]
    pub subject: SubjectId,
    #[serde(default)]
    pub memory_id: MemoryId,
    pub representation_text: String,
    pub semantic_role: Option<String>,
    pub title: Option<String>,
    pub evidence: Vec<MemoryRevisionEvidence>,
    pub relation: MemoryRelation,
    pub occurred_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub epistemic_class: EpistemicClass,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub label: String,
    pub description: Option<String>,
    pub kind_hint: Option<String>,
    #[serde(default = "default_explicit_origin")]
    pub origin: String,
}

fn default_explicit_origin() -> String {
    "explicit".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnchorRequest {
    pub label: Option<String>,
    pub description: String,
    #[serde(default = "default_explicit_origin")]
    pub origin: String,
    #[serde(default)]
    pub supports: Vec<AnchorSupportInput>,
    #[serde(default)]
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorSupportInput {
    pub reference: CognitiveRef,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssociationRequest {
    pub from: CognitiveRef,
    pub to: CognitiveRef,
    pub association_kind: String,
    pub polarity: AssociationPolarity,
    pub support_class: AssociationSupportClass,
    pub support_value: f64,
    pub occurrence_id: Option<OccurrenceId>,
    pub memory_revision_id: Option<MemoryRevisionId>,
    #[serde(default)]
    pub bridge_hint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebindEntityRequest {
    pub mention_id: Uuid,
    pub entity_ref: Option<EntityRef>,
    pub binding_state: String,
    pub host_resolution_ref: Option<String>,
    pub reason: Option<String>,
}

mod memory;
mod memory_support;
mod query;
mod query_diagnostics;
mod query_evidence;
mod query_helpers;
mod query_projection;
mod runtime;
mod topology;
