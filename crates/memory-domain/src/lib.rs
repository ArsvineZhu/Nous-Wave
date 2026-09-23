//! Durable cognition semantics: Memory, revisions, topology objects, and use.

use chrono::{DateTime, Utc};
use nous_core::{
    AnchorId, CognitiveRef, EntityRef, EpistemicClass, Error, MemoryId, MemoryRevisionId,
    OccurrenceId, Result, SourceRegionId, SubjectId, TagId,
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
    pub head_revision: i64,
    pub memory_id: MemoryId,
    pub subject_id: SubjectId,
    pub memory_class: MemoryClass,
    pub current_revision_id: MemoryRevisionId,
    pub created_at: DateTime<Utc>,
    pub status: MemoryStatus,
    pub accessibility_mode: AccessibilityMode,
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
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revision_lifecycle: RevisionLifecycle,
    pub revision_intent: Option<RevisionIntent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionLifecycle {
    Current,
    Superseded,
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
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub epistemic_class: EpistemicClass,
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
        )?;
        if self
            .tag_proposals
            .iter()
            .any(|tag| match tag {TagProposal::Existing{..}=>false,TagProposal::New{label,..}=>label.trim().is_empty()||label.len()>256})
        {
            return Err(Error::Invalid("Tag proposal label is invalid".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="kind",rename_all="snake_case")]
pub enum TagProposal {
    Existing{tag_id:TagId},
    New{label:String,description:Option<String>,kind_hint:Option<String>},
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
    #[serde(default)]
    pub tag_order_provenance: Option<serde_json::Value>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub epistemic_class: EpistemicClass,
}

impl ExplicitMemoryInput {
    pub fn validate(&self) -> Result<()> {
        validate_memory_fields(
            &self.semantic_role,
            &self.representation_text,
            self.valid_from,
            self.valid_to,
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
        Ok(())
    }
}

fn validate_memory_fields(
    semantic_role: &str,
    representation_text: &str,
    valid_from: Option<DateTime<Utc>>,
    valid_to: Option<DateTime<Utc>>,
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
    #[serde(default)]
    pub topology: Option<TopologyConsolidationProposal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TopologyConsolidationProposal {
    #[serde(default)]
    pub tags: Vec<TopologyTagProposal>,
    #[serde(default)]
    pub anchors: Vec<TopologyAnchorProposal>,
    #[serde(default)]
    pub associations: Vec<TopologyAssociationProposal>,
    #[serde(default)]
    pub revisions: Vec<TopologyRevisionProposal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyTagProposal {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub kind_hint: Option<String>,
    #[serde(default)]
    pub tag_id: Option<TagId>,
    #[serde(default)]
    pub attach_to: Vec<MemoryRevisionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyAnchorProposal {
    #[serde(default)]
    pub label: Option<String>,
    pub description: String,
    #[serde(default)]
    pub supports: Vec<TopologyAnchorSupport>,
    #[serde(default)]
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyAnchorSupport {
    pub reference: CognitiveRef,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyAssociationProposal {
    pub from: CognitiveRef,
    pub to: CognitiveRef,
    pub association_kind: String,
    pub polarity: AssociationPolarity,
    pub support_class: AssociationSupportClass,
    pub support_value: f64,
    #[serde(default)]
    pub occurrence_id: Option<OccurrenceId>,
    #[serde(default)]
    pub memory_revision_id: Option<MemoryRevisionId>,
    #[serde(default)]
    pub bridge_hint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyRevisionProposal {
    pub expected_head_revision: i64,
    pub intent: RevisionIntent,
    pub entity_refs: Vec<EntityRef>,
    pub memory_id: MemoryId,
    pub representation_text: String,
    #[serde(default)]
    pub semantic_role: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    pub evidence: Vec<MemoryRevisionEvidence>,
    #[serde(default)]
    pub valid_from: Option<DateTime<Utc>>,
    #[serde(default)]
    pub valid_to: Option<DateTime<Utc>>,
    pub epistemic_class: EpistemicClass,
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
            tag_order_provenance: None,
            valid_from: None,
            valid_to: None,
            epistemic_class: EpistemicClass::Reported,
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn consolidation_target_uses_public_snake_case_wire_values() {
        let target: ConsolidationTarget = serde_json::from_str("\"integrative\"").expect("target");
        assert!(matches!(target, ConsolidationTarget::Integrative));
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum RevisionIntent {Correct,Rephrase,Reinterpret,Revoke}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize,Default)]
#[serde(rename_all="snake_case")]
pub enum AccessibilityMode {#[default] Auto,Normal,Deep,Explicit}
#[derive(Debug,Clone,Copy,PartialEq,Eq,Serialize,Deserialize)]
#[serde(rename_all="snake_case")]
pub enum AccessibilityLevel {Normal,Deep,Explicit}
#[derive(Debug,Clone,Serialize,Deserialize,Default)]
pub struct TemporalEvidence {
    pub occurred_min:Option<DateTime<Utc>>,
    pub occurred_max:Option<DateTime<Utc>>,
    pub observed_min:Option<DateTime<Utc>>,
    pub observed_max:Option<DateTime<Utc>>,
}
