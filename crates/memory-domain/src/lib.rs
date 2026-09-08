//! Durable cognition semantics: Memory, revisions, topology objects, and use.

use chrono::{DateTime, Utc};
use nous_core::{
    AnchorId, CognitiveRef, EntityRef, EpistemicClass, Error, MemoryId, MemoryRevisionId,
    OccurrenceId, Result, SessionId, SourceRegionId, SubjectId, TagId,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryClass {
    Specific,
    Integrative,
    Procedural,
}

impl MemoryClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Specific => "specific",
            Self::Integrative => "integrative",
            Self::Procedural => "procedural",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryObject {
    pub memory_id: MemoryId,
    pub subject_id: SubjectId,
    pub memory_class: MemoryClass,
    pub current_revision_id: MemoryRevisionId,
    pub created_at: DateTime<Utc>,
    pub status: MemoryStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStatus {
    Active,
    Suppressed,
    Purging,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRevision {
    pub memory_revision_id: MemoryRevisionId,
    pub memory_id: MemoryId,
    pub subject_id: SubjectId,
    pub revision_no: i32,
    pub parent_revision_id: Option<MemoryRevisionId>,
    pub semantic_role: String,
    pub title: Option<String>,
    pub representation_text: String,
    pub attributes: serde_json::Value,
    pub epistemic_class: EpistemicClass,
    pub confidence: Option<f64>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub supersession_state: SupersessionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupersessionState {
    Current,
    Superseded,
    Contradicted,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvidenceRef {
    Occurrence {
        occurrence_id: OccurrenceId,
    },
    SourceRegion {
        source_region_id: SourceRegionId,
    },
    DerivedRepresentation {
        derived_representation_id: nous_core::DerivedRepresentationId,
    },
    DerivedRegion {
        derived_region_id: nous_core::DerivedRegionId,
    },
}

impl EvidenceRef {
    pub fn cognitive_ref(&self) -> CognitiveRef {
        match self {
            Self::Occurrence { occurrence_id } => CognitiveRef::Occurrence(*occurrence_id),
            Self::SourceRegion { source_region_id } => {
                CognitiveRef::SourceRegion(*source_region_id)
            }
            Self::DerivedRepresentation {
                derived_representation_id,
            } => CognitiveRef::DerivedRepresentation(*derived_representation_id),
            Self::DerivedRegion { derived_region_id } => {
                CognitiveRef::DerivedRegion(*derived_region_id)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportRole {
    Direct,
    Corroborating,
    Interpretation,
    Contradiction,
    Contextual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRevisionEvidence {
    pub evidence_no: i32,
    pub evidence: EvidenceRef,
    pub support_role: SupportRole,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRelation {
    Supersedes,
    Contradicts,
    Integrates,
    DerivedFrom,
    Proceduralizes,
}

impl MemoryRelation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supersedes => "supersedes",
            Self::Contradicts => "contradicts",
            Self::Integrates => "integrates",
            Self::DerivedFrom => "derived_from",
            Self::Proceduralizes => "proceduralizes",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRevisionRelation {
    pub from_revision_id: MemoryRevisionId,
    pub to_revision_id: MemoryRevisionId,
    pub relation: MemoryRelation,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub tag_id: TagId,
    pub subject_id: SubjectId,
    pub current_revision_id: Uuid,
    pub status: TopologyStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRevision {
    pub tag_revision_id: Uuid,
    pub tag_id: TagId,
    pub revision_no: i32,
    pub label: String,
    pub description: Option<String>,
    pub kind_hint: Option<String>,
    pub origin: String,
    pub producer_signature_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRevisionTag {
    pub memory_revision_id: MemoryRevisionId,
    pub tag_id: TagId,
    pub role: String,
    pub ordinal: Option<i32>,
    pub order_provenance: Option<serde_json::Value>,
    pub provenance: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    pub anchor_id: AnchorId,
    pub subject_id: SubjectId,
    pub current_revision_id: Uuid,
    pub status: TopologyStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorRevision {
    pub anchor_revision_id: Uuid,
    pub anchor_id: AnchorId,
    pub revision_no: i32,
    pub label: Option<String>,
    pub description: String,
    pub origin: String,
    pub producer_signature_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyStatus {
    Active,
    Superseded,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorSupport {
    pub anchor_revision_id: Uuid,
    pub support_ref_kind: String,
    pub support_ref: String,
    pub support_role: String,
    pub provenance: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationEvidence {
    pub association_evidence_id: Uuid,
    pub subject_id: SubjectId,
    pub from_ref_kind: String,
    pub from_ref: String,
    pub to_ref_kind: String,
    pub to_ref: String,
    pub association_kind: String,
    pub polarity: AssociationPolarity,
    pub support_class: AssociationSupportClass,
    pub support_value: f64,
    pub occurrence_id: Option<OccurrenceId>,
    pub memory_revision_id: Option<MemoryRevisionId>,
    pub producer_signature_id: Option<Uuid>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssociationPolarity {
    Positive,
    Negative,
}

impl AssociationPolarity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssociationSupportClass {
    HostExplicit,
    MemoryEvidence,
    Consolidation,
    MeaningfulUse,
    DerivedStructure,
}

impl AssociationSupportClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HostExplicit => "host_explicit",
            Self::MemoryEvidence => "memory_evidence",
            Self::Consolidation => "consolidation",
            Self::MeaningfulUse => "meaningful_use",
            Self::DerivedStructure => "derived_structure",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFormationProposal {
    pub memory_class: MemoryClass,
    pub semantic_role: String,
    pub representation_text: String,
    pub title: Option<String>,
    pub evidence: Vec<MemoryRevisionEvidence>,
    pub entity_refs: Vec<EntityRef>,
    pub tag_proposals: Vec<TagProposal>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub epistemic_class: EpistemicClass,
    pub confidence: Option<f64>,
}

impl MemoryFormationProposal {
    pub fn validate(&self) -> Result<()> {
        if self.memory_class != MemoryClass::Specific {
            return Err(Error::Invalid(
                "provider-assisted formation only creates Specific Memory".into(),
            ));
        }
        validate_memory_fields(
            &self.semantic_role,
            &self.representation_text,
            self.valid_from,
            self.valid_to,
            self.confidence,
        )?;
        if self
            .tag_proposals
            .iter()
            .any(|tag| tag.label.trim().is_empty() || tag.label.len() > 256)
        {
            return Err(Error::Invalid("Tag proposal label is invalid".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagProposal {
    pub label: String,
    pub description: Option<String>,
    pub kind_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplicitMemoryInput {
    #[serde(default)]
    pub subject: SubjectId,
    pub memory_class: MemoryClass,
    pub semantic_role: String,
    pub representation_text: String,
    pub title: Option<String>,
    #[serde(default)]
    pub evidence: Vec<MemoryRevisionEvidence>,
    #[serde(default)]
    pub entity_refs: Vec<EntityRef>,
    #[serde(default)]
    pub tags: Vec<TagId>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub epistemic_class: EpistemicClass,
    pub confidence: Option<f64>,
}

impl ExplicitMemoryInput {
    pub fn validate(&self) -> Result<()> {
        validate_memory_fields(
            &self.semantic_role,
            &self.representation_text,
            self.valid_from,
            self.valid_to,
            self.confidence,
        )?;
        if self.evidence.is_empty() {
            return Err(Error::Invalid(
                "a Memory must cite at least one evidence ref".into(),
            ));
        }
        let mut evidence_numbers = std::collections::HashSet::new();
        if self.evidence.iter().any(|evidence| {
            evidence.evidence_no < 0 || !evidence_numbers.insert(evidence.evidence_no)
        }) {
            return Err(Error::Invalid(
                "evidence_no values must be unique and non-negative".into(),
            ));
        }
        if self.evidence.iter().any(|evidence| {
            evidence
                .weight
                .is_some_and(|weight| !weight.is_finite() || weight < 0.0)
        }) {
            return Err(Error::Invalid(
                "evidence weight must be finite and non-negative".into(),
            ));
        }
        Ok(())
    }
}

fn validate_memory_fields(
    semantic_role: &str,
    representation_text: &str,
    valid_from: Option<DateTime<Utc>>,
    valid_to: Option<DateTime<Utc>>,
    confidence: Option<f64>,
) -> Result<()> {
    if semantic_role.trim().is_empty() || semantic_role.len() > 128 {
        return Err(Error::Invalid(
            "semantic_role is required and bounded".into(),
        ));
    }
    if representation_text.trim().is_empty() || representation_text.len() > 1_000_000 {
        return Err(Error::Invalid(
            "representation_text is required and bounded".into(),
        ));
    }
    if let (Some(start), Some(end)) = (valid_from, valid_to)
        && start > end
    {
        return Err(Error::Invalid("valid_to precedes valid_from".into()));
    }
    if confidence.is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value)) {
        return Err(Error::Invalid("confidence must be within [0,1]".into()));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsolidationTarget {
    Integrative,
    Procedural,
    TopologyOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationRequest {
    #[serde(default)]
    pub subject: SubjectId,
    pub source_memories: Vec<MemoryRevisionId>,
    pub target: ConsolidationTarget,
    pub capability: nous_core::CapabilityRequirement,
    pub representation_text: Option<String>,
    pub semantic_role: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_memory_requires_evidence() {
        let input = ExplicitMemoryInput {
            subject: SubjectId::new(),
            memory_class: MemoryClass::Specific,
            semantic_role: "fact".into(),
            representation_text: "a bounded fact".into(),
            title: None,
            evidence: Vec::new(),
            entity_refs: Vec::new(),
            tags: Vec::new(),
            occurred_at: None,
            observed_at: Utc::now(),
            valid_from: None,
            valid_to: None,
            epistemic_class: EpistemicClass::Reported,
            confidence: None,
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn consolidation_target_uses_public_snake_case_wire_values() {
        let target: ConsolidationTarget = serde_json::from_str("\"integrative\"").expect("target");
        assert!(matches!(target, ConsolidationTarget::Integrative));
    }
}
