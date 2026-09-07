//! Application orchestration for the first Nous Wave production wave.

mod embedding;
mod support;

use chrono::{DateTime, Utc};
use futures::Stream;
use nous_core::*;
use nous_material::*;
use nous_memory_domain::*;
use nous_memory_retrieval::{
    CandidateRankInput, CandidateSemanticTrail, CandidateTopologyObservation, DenseGeneration,
    EpaBasisGeneration, ExactPostings, LexicalDocument, LexicalGeneration, ResidualConfig,
    ServingPublisher, ServingSnapshot, SourceSeed, TrailOrder, VectorRecord, WaveConfig,
    WaveEdgeEvidence, WaveGraphGeneration, WaveNode, WaveNodeKind, bounded_restart_field,
    build_epa_basis, observe_epa, propagate, rank_candidates, residual_pyramid_with_search,
    trail_topology_observation, wave_observability,
};
use nous_memory_store::MemoryStore;
use nous_object_store::ObjectStore;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Arc, RwLock},
};
use uuid::Uuid;

pub use embedding::{
    MemoryFormationProvider, MemoryFormationRequest, TextEmbeddingOutput, TextEmbeddingProvider,
    TextEmbeddingRequest,
};
pub use nous_core::{CognitiveQuery, CognitiveQueryResult};
pub use nous_material::{AcceptedObservation, ObservationInput, ObservationMaterial};

#[derive(Clone)]
pub struct LocalRuntime {
    pub store: MemoryStore,
    pub objects: ObjectStore,
    pub publisher: ServingPublisher,
    pub max_upload_bytes: u64,
    pub resident_limit: usize,
    capabilities: Arc<Vec<CapabilityDescriptor>>,
    resource_resolvers: Arc<RwLock<HashMap<String, Arc<dyn ResourceResolver>>>>,
    text_embedding_provider: Option<Arc<dyn TextEmbeddingProvider>>,
    memory_formation_provider: Option<Arc<dyn MemoryFormationProvider>>,
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
pub struct CreateSubject {
    pub subject_id: Option<SubjectId>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Default for CreateSubject {
    fn default() -> Self {
        Self {
            subject_id: None,
            metadata: serde_json::json!({}),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectView {
    pub subject_id: SubjectId,
    pub created_at: DateTime<Utc>,
    pub state_revision: i64,
    pub status: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionView {
    pub session_id: SessionId,
    pub subject_id: SubjectId,
    pub opened_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub last_meaningful_use_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub state_revision: i64,
    pub resident: Vec<ResidentView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidentView {
    pub reference: CognitiveRef,
    pub entry_reason: String,
    pub state: String,
    pub entered_at: DateTime<Utc>,
    pub last_meaningful_use_at: Option<DateTime<Utc>>,
    pub hold_until: Option<DateTime<Utc>>,
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
pub struct UseFeedback {
    #[serde(default)]
    pub subject: SubjectId,
    pub session_id: Option<SessionId>,
    pub consumer: Option<String>,
    pub events: Vec<UseFeedbackEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseFeedbackEvent {
    pub reference: CognitiveRef,
    pub use_kind: UseKind,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpsert {
    pub resource_ref: ResourceRef,
    pub display_label: Option<String>,
    pub authority_class: String,
    #[serde(default)]
    pub coverage: serde_json::Value,
    #[serde(default)]
    pub query_dimensions: serde_json::Value,
    #[serde(default)]
    pub modalities: serde_json::Value,
    #[serde(default)]
    pub freshness_policy: serde_json::Value,
    pub access_cost_class: String,
    pub resolver_key: String,
    pub readiness: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceView {
    pub descriptor: ResourceDescriptor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMetadata {
    pub media_type: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[serde(default = "default_source_class")]
    pub source_class: SourceClass,
    pub external_object_ref: Option<ObjectRef>,
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(default = "Utc::now")]
    pub observed_at: DateTime<Utc>,
    pub conversation_ref: Option<String>,
    pub actor_entity_ref: Option<EntityRef>,
}

fn default_source_class() -> SourceClass {
    SourceClass::File
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionStatus {
    pub generation: u64,
    pub wave_nodes: usize,
    pub wave_edges: usize,
    pub lexical_ready: bool,
    pub dense_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuery {
    pub dimensions: serde_json::Value,
    pub synopsis_only: bool,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQueryResult {
    pub resource: ResourceRef,
    pub current_authority: bool,
    pub records: Vec<serde_json::Value>,
    pub evidence: Vec<EvidenceHandle>,
    pub degraded: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMaterial {
    pub handle: String,
    pub media_type: String,
    pub bytes: Vec<u8>,
}

#[async_trait::async_trait]
pub trait ResourceResolver: Send + Sync {
    async fn describe(&self, resource: &ResourceRef) -> Result<ResourceDescriptor>;
    async fn query(
        &self,
        resource: &ResourceRef,
        request: ResourceQuery,
    ) -> Result<ResourceQueryResult>;
    async fn materialize(&self, handle: &str) -> Result<ResourceMaterial>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationClaim {
    pub derivation_id: DerivationId,
    pub subject_id: SubjectId,
    pub source_region_id: SourceRegionId,
    pub representation_kind: String,
    pub producer_signature_id: Uuid,
    pub attempt_id: Uuid,
    pub attempt_no: i32,
    pub lease_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExtractionRequest {
    pub subject: SubjectId,
    pub source_region: SourceRegionId,
    pub media_type: String,
    pub input: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExtractionOutput {
    pub representation_kind: RepresentationKind,
    pub text: String,
    pub producer: ProducerSignature,
    pub derived_regions: Vec<DerivedRegion>,
    pub warnings: Vec<String>,
    pub coverage: serde_json::Value,
}

#[async_trait::async_trait]
pub trait DocumentExtractionProvider: Send + Sync {
    async fn extract(&self, request: DocumentExtractionRequest)
    -> Result<DocumentExtractionOutput>;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationRunResult {
    pub claimed: usize,
    pub succeeded: usize,
    pub failed: usize,
}

mod derivation;
mod material;
mod memory;
mod projection;
mod query;
mod runtime;
mod topology;
