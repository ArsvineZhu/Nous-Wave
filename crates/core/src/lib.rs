//! Shared semantic primitives for Nous Wave.
//!
//! This crate deliberately contains no SQL, HTTP, model-provider, vector-index,
//! or object-store mechanics. It is the vocabulary shared by Authority,
//! runtime, projections, and the host-facing API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

pub const API_VERSION: u32 = 1;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl From<$name> for Uuid {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

uuid_id!(SubjectId);
uuid_id!(MemoryId);
uuid_id!(MemoryRevisionId);
uuid_id!(ArtifactId);
uuid_id!(OccurrenceId);
uuid_id!(SourceRegionId);
uuid_id!(DerivedRepresentationId);
uuid_id!(DerivedRegionId);
uuid_id!(TagId);
uuid_id!(AnchorId);
uuid_id!(SessionId);
uuid_id!(DerivationId);
uuid_id!(ServingGenerationId);
uuid_id!(UseEventId);

/// A Host-owned identity. The string is opaque to Nous except for exact
/// equality and its namespace/type prefix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityRef(String);

impl EntityRef {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_opaque_ref(&value, "entity")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for EntityRef {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

/// A Host-owned resource identity. Nous stores awareness and uses a resolver;
/// it does not make resource contents its Authority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResourceRef(String);

impl ResourceRef {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_opaque_ref(&value, "resource")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ResourceRef {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

/// An opaque reference to an external object owned by a Host system.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObjectRef(String);

impl ObjectRef {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        validate_opaque_ref(&value, "object")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ObjectRef {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

fn validate_opaque_ref(value: &str, expected_prefix: &str) -> Result<()> {
    let parts: Vec<_> = value.split(':').collect();
    if value.is_empty()
        || value.len() > 1024
        || parts.len() < 2
        || parts[0] != expected_prefix
        || value.chars().any(char::is_whitespace)
        || parts.iter().any(|part| part.is_empty())
    {
        return Err(Error::Invalid(format!(
            "invalid {expected_prefix} reference"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum CognitiveRef {
    Memory(MemoryId),
    MemoryRevision(MemoryRevisionId),
    Artifact(ArtifactId),
    SourceRegion(SourceRegionId),
    DerivedRepresentation(DerivedRepresentationId),
    DerivedRegion(DerivedRegionId),
    Entity(EntityRef),
    Tag(TagId),
    Anchor(AnchorId),
    Resource(ResourceRef),
    ExternalObject(ObjectRef),
    Occurrence(OccurrenceId),
    Session(SessionId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityClass {
    SubjectCognition,
    ExternalCurrentAuthority,
    Evidence,
    Interpretation,
    ResourceDescriptor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicClass {
    Observed,
    Reported,
    Derived,
    Inferred,
    Narrative,
    Simulated,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceClass {
    Message,
    Tool,
    File,
    Web,
    HostEvent,
    Simulation,
    Generated,
    Import,
    Other(String),
}

impl SourceClass {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Message => "message",
            Self::Tool => "tool",
            Self::File => "file",
            Self::Web => "web",
            Self::HostEvent => "host_event",
            Self::Simulation => "simulation",
            Self::Generated => "generated",
            Self::Import => "import",
            Self::Other(value) => value,
        }
    }
}

impl From<String> for SourceClass {
    fn from(value: String) -> Self {
        match value.as_str() {
            "message" => Self::Message,
            "tool" => Self::Tool,
            "file" => Self::File,
            "web" => Self::Web,
            "host_event" => Self::HostEvent,
            "simulation" => Self::Simulation,
            "generated" => Self::Generated,
            "import" => Self::Import,
            _ => Self::Other(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityOperation {
    TextEmbedding,
    TextRerank,
    TextInterpretation,
    MemoryFormationText,
    MemoryConsolidationText,
    ImageInterpretation,
    ImageEmbedding,
    SpeechTranscription,
    DocumentExtraction,
}

impl CapabilityOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TextEmbedding => "text.embedding",
            Self::TextRerank => "text.rerank",
            Self::TextInterpretation => "text.interpretation",
            Self::MemoryFormationText => "memory.formation.text",
            Self::MemoryConsolidationText => "memory.consolidation.text",
            Self::ImageInterpretation => "image.interpretation",
            Self::ImageEmbedding => "image.embedding",
            Self::SpeechTranscription => "speech.transcription",
            Self::DocumentExtraction => "document.extraction",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Text,
    Image,
    Audio,
    Video,
    Structured,
    Binary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepresentationKind {
    ExtractedText,
    Ocr,
    Transcript,
    ImageDescription,
    SceneDescription,
    Summary,
    Embedding,
    ResourceSynopsis,
    Other,
}

impl RepresentationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExtractedText => "extracted_text",
            Self::Ocr => "ocr",
            Self::Transcript => "transcript",
            Self::ImageDescription => "image_description",
            Self::SceneDescription => "scene_description",
            Self::Summary => "summary",
            Self::Embedding => "embedding",
            Self::ResourceSynopsis => "resource_synopsis",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RequirementStrength {
    Required,
    Preferred,
    #[default]
    Optional,
    Forbidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityReadiness {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityLimits {
    pub max_input_bytes: Option<u64>,
    pub max_output_bytes: Option<u64>,
    pub timeout_ms: Option<u64>,
}

impl Default for CapabilityLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: None,
            max_output_bytes: None,
            timeout_ms: Some(30_000),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub operation: CapabilityOperation,
    pub input_modalities: Vec<Modality>,
    pub output_kind: RepresentationKind,
    pub producer: ProducerSignature,
    pub limits: CapabilityLimits,
    pub readiness: CapabilityReadiness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    pub operation: CapabilityOperation,
    pub strength: RequirementStrength,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerSignature {
    pub signature_hash: String,
    pub provider_class: String,
    pub operation: CapabilityOperation,
    pub implementation: String,
    pub model_identity: Option<String>,
    pub model_revision: Option<String>,
    pub preprocessing_identity: String,
    pub preprocessing_revision: String,
    pub config_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingSpaceSignature {
    pub space_hash: String,
    pub model_identity: String,
    pub weights_revision: String,
    pub task: String,
    pub input_representation: String,
    pub preprocessing_identity: String,
    pub preprocessing_revision: String,
    pub dimension: u32,
    pub normalization: String,
    pub output_semantics: String,
}

impl EmbeddingSpaceSignature {
    pub fn compatible_with(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeInterval {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

impl TimeInterval {
    pub fn validate(&self) -> Result<()> {
        if let (Some(start), Some(end)) = (self.start, self.end)
            && start > end
        {
            return Err(Error::Invalid("time interval starts after its end".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessDescriptor {
    pub observed_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceHandle {
    pub reference: CognitiveRef,
    pub support_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceSummary {
    pub source_count: usize,
    pub producer_signatures: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializationHandle {
    pub reference: CognitiveRef,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextContribution {
    pub reference: CognitiveRef,
    pub semantic_role: String,
    pub text: Option<String>,
    pub authority: AuthorityClass,
    pub freshness: FreshnessDescriptor,
    pub evidence: Vec<EvidenceHandle>,
    pub provenance: ProvenanceSummary,
    pub priority: f32,
    pub estimated_tokens: Option<u32>,
    pub materialization: Option<MaterializationHandle>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SituationDescriptor {
    pub consumer: Option<String>,
    #[serde(default)]
    pub current_refs: Vec<CognitiveRef>,
    #[serde(default)]
    pub current_objects: Vec<ObjectRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QueryTarget {
    AnyRelevantCognition,
    Memory,
    Evidence,
    EntityNeighborhood { entity_ref: EntityRef },
    AnchorNeighborhood { anchor: AnchorId },
    Resource,
    Exact { reference: CognitiveRef },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCue {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCue {
    pub entity_ref: EntityRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectCue {
    pub object_ref: ObjectRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactCue {
    pub artifact: ArtifactId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRegionCue {
    pub region: CognitiveRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCue {
    pub tag: TagId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorCue {
    pub anchor: AnchorId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalCue {
    pub interval: TimeInterval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationCue {
    pub from: CognitiveRef,
    pub to: CognitiveRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleCue {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCue {
    pub resource: ResourceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Cue {
    Text(TextCue),
    Entity(EntityCue),
    Object(ObjectCue),
    Artifact(ArtifactCue),
    MediaRegion(MediaRegionCue),
    Tag(TagCue),
    Anchor(AnchorCue),
    Temporal(TemporalCue),
    Relation(RelationCue),
    Example(ExampleCue),
    Resource(ResourceCue),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryConstraints {
    #[serde(default)]
    pub source_classes_include: Vec<SourceClass>,
    #[serde(default)]
    pub source_classes_exclude: Vec<SourceClass>,
    #[serde(default)]
    pub memory_classes_include: Vec<String>,
    #[serde(default)]
    pub memory_classes_exclude: Vec<String>,
    #[serde(default)]
    pub entity_requirements: Vec<EntityRef>,
    pub occurred: Option<TimeInterval>,
    pub observed: Option<TimeInterval>,
    pub valid: Option<TimeInterval>,
    #[serde(default)]
    pub include_suppressed: bool,
    pub authority: Option<AuthorityClass>,
    #[serde(default)]
    pub modalities: Vec<Modality>,
    #[serde(default)]
    pub evidence_classes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExplorationIntent {
    #[default]
    None,
    BoundedAssociative,
    AroundTag,
    AroundAnchor,
    ExplainAssociation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CurrentAuthorityNeed {
    #[default]
    None,
    Prefer,
    Required,
    HistoricalOrStale,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceIntent {
    #[serde(default)]
    pub current_authority: CurrentAuthorityNeed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultNeed {
    #[serde(default = "default_result_limit")]
    pub limit: usize,
    #[serde(default = "default_true")]
    pub need_evidence: bool,
    #[serde(default)]
    pub need_materialization_handles: bool,
}

fn default_result_limit() -> usize {
    12
}

fn default_true() -> bool {
    true
}

impl Default for ResultNeed {
    fn default() -> Self {
        Self {
            limit: default_result_limit(),
            need_evidence: true,
            need_materialization_handles: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveEffort {
    Light,
    #[default]
    Normal,
    Deep,
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    #[serde(default)]
    pub text_embedding: RequirementStrength,
    #[serde(default)]
    pub text_rerank: RequirementStrength,
    #[serde(default)]
    pub multimodal_interpretation: RequirementStrength,
    #[serde(default)]
    pub residual_sensing: RequirementStrength,
}

impl Default for CapabilityPolicy {
    fn default() -> Self {
        Self {
            text_embedding: RequirementStrength::Optional,
            text_rerank: RequirementStrength::Optional,
            multimodal_interpretation: RequirementStrength::Optional,
            residual_sensing: RequirementStrength::Optional,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticsRequest {
    None,
    #[default]
    Summary,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveQuery {
    pub api_version: u32,
    #[serde(default)]
    pub subject: SubjectId,
    pub session: Option<SessionId>,
    #[serde(default)]
    pub situation: SituationDescriptor,
    #[serde(default)]
    pub targets: Vec<QueryTarget>,
    #[serde(default)]
    pub cues: Vec<Cue>,
    #[serde(default)]
    pub constraints: QueryConstraints,
    #[serde(default)]
    pub exploration: ExplorationIntent,
    #[serde(default)]
    pub resources: ResourceIntent,
    #[serde(default)]
    pub result_need: ResultNeed,
    #[serde(default)]
    pub effort: CognitiveEffort,
    #[serde(default)]
    pub capabilities: CapabilityPolicy,
    #[serde(default)]
    pub diagnostics: DiagnosticsRequest,
}

impl CognitiveQuery {
    pub fn validate(&self) -> Result<()> {
        if self.api_version != API_VERSION {
            return Err(Error::Invalid(format!(
                "unsupported cognitive query api_version {}",
                self.api_version
            )));
        }
        if self.result_need.limit == 0 || self.result_need.limit > 2048 {
            return Err(Error::Invalid(
                "result_need.limit must be between 1 and 2048".into(),
            ));
        }
        if self.cues.len() > 256 || self.targets.len() > 128 {
            return Err(Error::Invalid("query cue/target bound exceeded".into()));
        }
        if self.capabilities.text_embedding == RequirementStrength::Forbidden
            && self.capabilities.residual_sensing == RequirementStrength::Required
        {
            return Err(Error::Invalid(
                "required residual_sensing conflicts with forbidden text_embedding".into(),
            ));
        }
        if let Some(interval) = self.constraints.occurred {
            interval.validate()?;
        }
        if let Some(interval) = self.constraints.observed {
            interval.validate()?;
        }
        if let Some(interval) = self.constraints.valid {
            interval.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryStatus {
    Complete,
    Degraded,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Degradation {
    pub code: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryGenerationTrace {
    pub lexical: Option<ServingGenerationId>,
    #[serde(default)]
    pub dense: Vec<ServingGenerationId>,
    pub wave: Option<ServingGenerationId>,
    pub epa_basis: Option<ServingGenerationId>,
    pub postings: Option<ServingGenerationId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceFamily {
    Exact,
    Runtime,
    Entity,
    Lexical,
    SemanticDense,
    TagDirect,
    AnchorDirect,
    Temporal,
    WaveField,
    Resource,
    LanguageRerank,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchEvidence {
    #[serde(default)]
    pub families: Vec<EvidenceFamily>,
    pub base_rank_score: f64,
    pub field_contact: f64,
    pub structural_score: f64,
    pub topology_innovation: f64,
    pub wave_observability: f64,
    pub direct_seed_evidence: f64,
    pub final_score: f64,
    #[serde(default)]
    pub variants: Vec<String>,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveHit {
    pub reference: CognitiveRef,
    pub revision: Option<MemoryRevisionId>,
    pub semantic_role: Option<String>,
    pub memory_class: Option<String>,
    pub representation: Option<String>,
    pub authority: AuthorityClass,
    pub freshness: FreshnessDescriptor,
    #[serde(default)]
    pub entity_refs: Vec<EntityRef>,
    #[serde(default)]
    pub evidence: Vec<EvidenceHandle>,
    pub match_evidence: MatchEvidence,
    #[serde(default)]
    pub materialization: Vec<MaterializationHandle>,
    pub supersession_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceActionSuggestion {
    pub resource: ResourceRef,
    pub action: String,
    pub reason: String,
    #[serde(default)]
    pub current_authority: bool,
    #[serde(default)]
    pub records: Vec<serde_json::Value>,
    #[serde(default)]
    pub evidence: Vec<EvidenceHandle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryDiagnostics {
    pub candidate_counts: std::collections::BTreeMap<String, usize>,
    pub lane_status: std::collections::BTreeMap<String, String>,
    pub wave_observability: Option<f64>,
    pub trace: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveQueryResult {
    pub query_id: Uuid,
    pub generation: QueryGenerationTrace,
    pub status: QueryStatus,
    pub results: Vec<CognitiveHit>,
    #[serde(default)]
    pub resource_actions: Vec<ResourceActionSuggestion>,
    #[serde(default)]
    pub degradation: Vec<Degradation>,
    pub diagnostics: Option<QueryDiagnostics>,
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

impl fmt::Display for CognitiveRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory(id) => write!(f, "memory:{}", id.0),
            Self::MemoryRevision(id) => write!(f, "memory_revision:{}", id.0),
            Self::Artifact(id) => write!(f, "artifact:{}", id.0),
            Self::SourceRegion(id) => write!(f, "source_region:{}", id.0),
            Self::DerivedRepresentation(id) => write!(f, "derived_representation:{}", id.0),
            Self::DerivedRegion(id) => write!(f, "derived_region:{}", id.0),
            Self::Entity(id) => f.write_str(id.as_str()),
            Self::Tag(id) => write!(f, "tag:{}", id.0),
            Self::Anchor(id) => write!(f, "anchor:{}", id.0),
            Self::Resource(id) => f.write_str(id.as_str()),
            Self::ExternalObject(id) => f.write_str(id.as_str()),
            Self::Occurrence(id) => write!(f, "occurrence:{}", id.0),
            Self::Session(id) => write!(f, "session:{}", id.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_refs_require_their_owner_namespace() {
        assert!(EntityRef::new("entity:host-person:alice").is_ok());
        assert!(EntityRef::new("name:alice").is_err());
        assert!(ResourceRef::new("resource:schedule:primary").is_ok());
        assert!(ObjectRef::new("object:messaging:message:1").is_ok());
    }

    #[test]
    fn embedding_spaces_are_exactly_compatible() {
        let base = EmbeddingSpaceSignature {
            space_hash: "a".into(),
            model_identity: "model".into(),
            weights_revision: "1".into(),
            task: "retrieval".into(),
            input_representation: "text".into(),
            preprocessing_identity: "l2".into(),
            preprocessing_revision: "1".into(),
            dimension: 3,
            normalization: "l2".into(),
            output_semantics: "dense_similarity".into(),
        };
        assert!(base.compatible_with(&base));
        let mut changed = base.clone();
        changed.dimension = 4;
        assert!(!base.compatible_with(&changed));
    }
}
