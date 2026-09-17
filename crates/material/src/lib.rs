//! Evidence and material semantics.

use chrono::{DateTime, Utc};
use nous_core::{
    ArtifactId, CapabilityOperation, DerivedRegionId, DerivedRepresentationId, EntityRef, Error,
    ObjectRef, OccurrenceId, ProducerSignature, RepresentationKind, ResourceRef, Result,
    SourceClass, SourceRegionId, SubjectId,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use nous_core::{AuthorityClass, Modality};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_id: ArtifactId,
    pub subject_id: SubjectId,
    pub content_hash: String,
    pub byte_length: u64,
    pub media_type: String,
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

impl Artifact {
    pub fn validate(&self) -> Result<()> {
        if self.content_hash.len() != 64
            || !self
                .content_hash
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(Error::Invalid(
                "artifact content hash must be BLAKE3 hex".into(),
            ));
        }
        if self.media_type.trim().is_empty() || self.storage_key.trim().is_empty() {
            return Err(Error::Invalid(
                "artifact media_type/storage_key is required".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationOccurrence {
    pub occurrence_id: OccurrenceId,
    pub subject_id: SubjectId,
    pub artifact_id: Option<ArtifactId>,
    pub source_class: SourceClass,
    pub external_object_ref: Option<ObjectRef>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
    pub conversation_ref: Option<String>,
    pub actor_entity_ref: Option<EntityRef>,
    pub context: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl ObservationOccurrence {
    pub fn validate(&self) -> Result<()> {
        if self.observed_at < DateTime::<Utc>::UNIX_EPOCH {
            return Err(Error::Invalid("observed_at is before Unix epoch".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRegion {
    pub source_region_id: SourceRegionId,
    pub subject_id: SubjectId,
    pub artifact_id: ArtifactId,
    pub coordinate_kind: String,
    pub coordinate: serde_json::Value,
    pub coordinate_hash: String,
    pub parent_source_region_id: Option<SourceRegionId>,
    pub created_at: DateTime<Utc>,
}

impl SourceRegion {
    pub fn validate(&self) -> Result<()> {
        const KINDS: &[&str] = &[
            "whole_artifact",
            "byte_range",
            "text_span",
            "pdf_region",
            "image_bbox",
            "media_time",
            "json_pointer",
            "structural_path",
        ];
        if !KINDS.contains(&self.coordinate_kind.as_str()) {
            return Err(Error::Invalid("unsupported source coordinate kind".into()));
        }
        if self.coordinate_hash.is_empty() {
            return Err(Error::Invalid("source coordinate hash is required".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedRepresentation {
    pub derived_representation_id: DerivedRepresentationId,
    pub subject_id: SubjectId,
    pub source_region_id: SourceRegionId,
    pub representation_kind: RepresentationKind,
    pub producer: ProducerSignature,
    pub revision: i32,
    pub payload_text: Option<String>,
    pub payload_artifact_id: Option<ArtifactId>,
    pub quality: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub supersedes: Option<DerivedRepresentationId>,
}

impl DerivedRepresentation {
    pub fn validate(&self) -> Result<()> {
        if self.payload_text.is_none() && self.payload_artifact_id.is_none() {
            return Err(Error::Invalid(
                "derived representation needs text or an artifact payload".into(),
            ));
        }
        if self.revision < 1 {
            return Err(Error::Invalid(
                "derived representation revision starts at 1".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedRegion {
    pub derived_region_id: DerivedRegionId,
    pub subject_id: SubjectId,
    pub derived_representation_id: DerivedRepresentationId,
    pub coordinate_kind: String,
    pub coordinate: serde_json::Value,
    pub coordinate_hash: String,
    pub parent_derived_region_id: Option<DerivedRegionId>,
    pub created_at: DateTime<Utc>,
}

impl DerivedRegion {
    pub fn validate(&self) -> Result<()> {
        const KINDS: &[&str] = &["text_span", "segment", "bbox-in-derived", "structural_path"];
        if !KINDS.contains(&self.coordinate_kind.as_str()) || self.coordinate_hash.trim().is_empty()
        {
            return Err(Error::Invalid("invalid derived coordinate".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageNeed {
    pub coverage_need_id: Uuid,
    pub subject_id: SubjectId,
    pub source_region_id: SourceRegionId,
    pub representation_kind: RepresentationKind,
    pub capability_operation: CapabilityOperation,
    pub requirement: String,
    pub state: String,
    pub current_representation_id: Option<DerivedRepresentationId>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationRecord {
    pub derivation_id: nous_core::DerivationId,
    pub derivation_key: String,
    pub subject_id: SubjectId,
    pub source_region_id: SourceRegionId,
    pub representation_kind: RepresentationKind,
    pub producer: ProducerSignature,
    pub state: String,
    pub successful_representation_id: Option<DerivedRepresentationId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationAttempt {
    pub attempt_id: Uuid,
    pub derivation_id: nous_core::DerivationId,
    pub attempt_no: i32,
    pub state: String,
    pub lease_owner: Option<String>,
    pub lease_until: Option<DateTime<Utc>>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub problem_code: Option<String>,
    pub problem_detail: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObservationMaterial {
    InlineText { text: String, media_type: String },
    ArtifactRef { artifact_id: ArtifactId },
    ExternalObjectRef { object_ref: ObjectRef },
    StructuredJson { value: serde_json::Value },
    ResourceAvailability { resource: ResourceRef },
}

impl ObservationMaterial {
    pub fn modality(&self) -> Modality {
        match self {
            Self::InlineText { .. } => Modality::Text,
            Self::ArtifactRef { .. } => Modality::Binary,
            Self::ExternalObjectRef { .. } => Modality::Structured,
            Self::StructuredJson { .. } | Self::ResourceAvailability { .. } => Modality::Structured,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccurrenceDescriptor {
    pub source_class: SourceClass,
    pub external_object_ref: Option<ObjectRef>,
    pub occurred_at: Option<DateTime<Utc>>,
    #[serde(default = "Utc::now")]
    pub observed_at: DateTime<Utc>,
    pub conversation_ref: Option<String>,
    pub actor_entity_ref: Option<EntityRef>,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDirective {
    #[serde(default = "default_true")]
    pub admit: bool,
    pub hold_until: Option<DateTime<Utc>>,
}

impl Default for RuntimeDirective {
    fn default() -> Self {
        Self {
            admit: true,
            hold_until: None,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedEntityMention {
    pub surface: String,
    pub entity_ref: Option<EntityRef>,
    pub semantic_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationInput {
    #[serde(default)]
    pub subject: SubjectId,
    pub session: Option<nous_core::SessionId>,
    pub occurrence: OccurrenceDescriptor,
    pub material: ObservationMaterial,
    #[serde(default)]
    pub entities: Vec<ResolvedEntityMention>,
    #[serde(default)]
    pub runtime: RuntimeDirective,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedObservation {
    pub artifact: Option<Artifact>,
    pub occurrence: ObservationOccurrence,
    pub source_region: Option<SourceRegion>,
    #[serde(default)]
    pub mention_ids: Vec<Uuid>,
    pub resident: bool,
    pub memory_revisions: Vec<nous_core::MemoryRevisionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolObservation {
    pub subject: SubjectId,
    pub session: Option<nous_core::SessionId>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
    pub tool_ref: Option<ObjectRef>,
    pub result: serde_json::Value,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_coordinates_are_not_parser_chunk_ordinals() {
        let region = SourceRegion {
            source_region_id: SourceRegionId::new(),
            subject_id: SubjectId::new(),
            artifact_id: ArtifactId::new(),
            coordinate_kind: "text_span".into(),
            coordinate: serde_json::json!({"encoding":"utf-8","start":0,"end":4}),
            coordinate_hash: "stable-coordinate".into(),
            parent_source_region_id: None,
            created_at: Utc::now(),
        };
        assert!(region.validate().is_ok());
        assert!(
            SourceRegion {
                coordinate_kind: "chunk_12".into(),
                ..region
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn formation_directive_uses_public_snake_case_wire_values() {
        let directive: FormationDirective =
            serde_json::from_str("\"consider_specific\"").expect("directive");
        assert!(matches!(directive, FormationDirective::ConsiderSpecific));
    }
}
