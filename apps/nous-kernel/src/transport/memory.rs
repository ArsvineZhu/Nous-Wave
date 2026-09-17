use super::*;
use nous_authority_store::database_error as db;
use nous_core::{CognitiveRef, EntityRef, MemoryId, MemoryRevisionId, Result, SubjectId, TagId};
use nous_memory_domain::{EvidenceRef, ExplicitMemoryInput, MemoryRevisionEvidence};
use nous_memory_service::{MemoryView, ReviseMemoryInput};
use uuid::Uuid;

fn view(input: MemoryView) -> p::Memory {
    let object = input.object;
    let r = input.revision;
    p::Memory {
        memory_id: object.memory_id.0.to_string(),
        revision_id: r.memory_revision_id.0.to_string(),
        subject_id: object.subject_id.0.to_string(),
        memory_class: enum_name(object.memory_class),
        status: enum_name(object.status),
        etag: format!("h:{}", object.head_revision),
        semantic_role: r.semantic_role,
        text: r.representation_text,
        title: r.title,
        evidence: input
            .evidence
            .into_iter()
            .map(|e| p::Evidence {
                reference: Some(to_ref(e.evidence.cognitive_ref())),
                support_role: enum_name(e.support_role),
            })
            .collect(),
        tags: input.tags.into_iter().map(|t| t.0.to_string()).collect(),
        occurred_at: r.occurred_at.map(timestamp),
        observed_at: Some(timestamp(r.observed_at)),
        valid: Some(p::TimeInterval {
            start: r.valid_from.map(timestamp),
            end: r.valid_to.map(timestamp),
        }),
        epistemic_class: enum_name(r.epistemic_class),
        confidence: r.confidence,
        revision_no: r.revision_no,
    }
}
fn evidence(input: Vec<p::Evidence>) -> Result<Vec<MemoryRevisionEvidence>> {
    input
        .into_iter()
        .enumerate()
        .map(|(index, e)| {
            let evidence = match from_ref(required(e.reference, "evidence.reference")?)? {
                CognitiveRef::Occurrence(occurrence_id) => {
                    EvidenceRef::Occurrence { occurrence_id }
                }
                CognitiveRef::SourceRegion(source_region_id) => {
                    EvidenceRef::SourceRegion { source_region_id }
                }
                CognitiveRef::DerivedRepresentation(derived_representation_id) => {
                    EvidenceRef::DerivedRepresentation {
                        derived_representation_id,
                    }
                }
                CognitiveRef::DerivedRegion(derived_region_id) => {
                    EvidenceRef::DerivedRegion { derived_region_id }
                }
                _ => return Err(Error::Invalid("unsupported evidence reference".into())),
            };
            Ok(MemoryRevisionEvidence {
                evidence_no: index as i32,
                evidence,
                support_role: enum_value(&e.support_role)?,
                weight: None,
            })
        })
        .collect()
}
fn etag(value: &str) -> Result<i64> {
    value
        .strip_prefix("h:")
        .and_then(|v| v.parse().ok())
        .filter(|v| *v >= 0)
        .ok_or_else(|| Error::Invalid("expected_etag is required".into()))
}
impl KernelService {
    pub(super) async fn consolidate_memory(
        &self,
        input: p::ConsolidateMemoryRequest,
    ) -> Result<p::Memory> {
        let subject = SubjectId(id(&input.subject_id)?);
        let result = self
            .0
            .require_memory()?
            .consolidate(
                subject,
                nous_memory_domain::ConsolidationRequest {
                    subject,
                    source_memories: input
                        .source_revision_ids
                        .iter()
                        .map(|r| Ok(MemoryRevisionId(id(r)?)))
                        .collect::<Result<_>>()?,
                    target: enum_value(&input.target)?,
                    capability: nous_core::CapabilityRequirement {
                        operation: nous_core::CapabilityOperation::MemoryConsolidationText,
                        strength: nous_core::RequirementStrength::Optional,
                    },
                    representation_text: Some(input.text),
                    semantic_role: Some(input.semantic_role),
                    topology: None,
                },
            )
            .await?;
        Ok(view(required(result.memory, "consolidated Memory")?))
    }
    pub(super) async fn get_memory(&self, input: p::ObjectRequest) -> Result<p::Memory> {
        Ok(view(
            self.0
                .require_memory()?
                .memory(
                    SubjectId(id(&input.subject_id)?),
                    MemoryId(id(&input.id)?),
                    None,
                )
                .await?,
        ))
    }
    pub(super) async fn get_memory_revision(&self, input: p::ObjectRequest) -> Result<p::Memory> {
        Ok(view(
            self.0
                .require_memory()?
                .revision(
                    SubjectId(id(&input.subject_id)?),
                    MemoryRevisionId(id(&input.id)?),
                )
                .await?,
        ))
    }
    pub(super) async fn form_memory(&self, input: p::FormMemoryRequest) -> Result<p::Memory> {
        let m = required(input.input, "input")?;
        let valid = m.valid.unwrap_or_default();
        Ok(view(
            self.0
                .require_memory()?
                .form_memory(ExplicitMemoryInput {
                    subject: SubjectId(id(&input.subject_id)?),
                    memory_class: enum_value(&m.memory_class)?,
                    semantic_role: m.semantic_role,
                    representation_text: m.text,
                    title: m.title,
                    evidence: evidence(m.evidence)?,
                    entity_refs: input
                        .entity_refs
                        .into_iter()
                        .map(EntityRef::new)
                        .collect::<Result<_>>()?,
                    tags: m
                        .tags
                        .iter()
                        .map(|t| Ok(TagId(id(t)?)))
                        .collect::<Result<_>>()?,
                    tag_order_provenance: None,
                    occurred_at: time(m.occurred_at)?,
                    observed_at: required(time(m.observed_at)?, "observed_at")?,
                    valid_from: time(valid.start)?,
                    valid_to: time(valid.end)?,
                    epistemic_class: enum_value(&m.epistemic_class)?,
                    confidence: m.confidence,
                })
                .await?,
        ))
    }
    pub(super) async fn revise_memory(&self, input: p::ReviseMemoryRequest) -> Result<p::Memory> {
        let m = required(input.input, "input")?;
        let valid = m.valid.unwrap_or_default();
        Ok(view(
            self.0
                .require_memory()?
                .revise_memory(ReviseMemoryInput {
                    subject: SubjectId(id(&input.subject_id)?),
                    memory_id: MemoryId(id(&input.memory_id)?),
                    expected_head_revision: etag(&input.expected_etag)?,
                    representation_text: m.text,
                    semantic_role: Some(m.semantic_role),
                    title: m.title,
                    evidence: evidence(m.evidence)?,
                    relation: enum_value(input.relation.as_deref().unwrap_or("supersedes"))?,
                    occurred_at: time(m.occurred_at)?,
                    valid_from: time(valid.start)?,
                    valid_to: time(valid.end)?,
                    epistemic_class: enum_value(&m.epistemic_class)?,
                    confidence: m.confidence,
                })
                .await?,
        ))
    }
    pub(super) async fn suppress_memory(
        &self,
        input: p::MemoryMutationRequest,
    ) -> Result<p::Memory> {
        Ok(view(
            self.0
                .require_memory()?
                .suppress(
                    SubjectId(id(&input.subject_id)?),
                    MemoryId(id(&input.memory_id)?),
                    etag(&input.expected_etag)?,
                )
                .await?,
        ))
    }
    pub(super) async fn restore_memory(
        &self,
        input: p::MemoryMutationRequest,
    ) -> Result<p::Memory> {
        Ok(view(
            self.0
                .require_memory()?
                .restore(
                    SubjectId(id(&input.subject_id)?),
                    MemoryId(id(&input.memory_id)?),
                    etag(&input.expected_etag)?,
                )
                .await?,
        ))
    }
    pub(super) async fn purge_memory(&self, input: p::MemoryMutationRequest) -> Result<()> {
        self.0
            .require_memory()?
            .purge_memory(
                SubjectId(id(&input.subject_id)?),
                MemoryId(id(&input.memory_id)?),
                etag(&input.expected_etag)?,
            )
            .await
    }
    pub(super) async fn list_memories(
        &self,
        input: p::ListRequest,
    ) -> Result<p::ListMemoriesResponse> {
        let subject = SubjectId(id(&input.subject_id)?);
        self.0.store.require_subject(subject).await?;
        let scope = format!("memories:{}:{}", input.subject_id, input.status);
        let (limit, last) = page(input.page, &scope)?;
        let mut ids=sqlx::query_scalar::<_,Uuid>("SELECT memory_id FROM memory_objects WHERE subject_id=$1 AND ($2::uuid IS NULL OR memory_id>$2) AND ($3='' OR status=$3) ORDER BY memory_id LIMIT $4")
            .bind(subject.0).bind(last).bind(input.status).bind(limit+1).fetch_all(self.0.store.pool()).await.map_err(db)?;
        let more = ids.len() > limit as usize;
        ids.truncate(limit as usize);
        let next_page_token = if more {
            next_token(&scope, *ids.last().expect("nonempty page"))
        } else {
            String::new()
        };
        let mut items = Vec::new();
        for id in ids {
            items.push(view(
                self.0
                    .require_memory()?
                    .memory(subject, MemoryId(id), None)
                    .await?,
            ));
        }
        Ok(p::ListMemoriesResponse {
            items,
            next_page_token,
        })
    }
    pub(super) async fn list_memory_revisions(
        &self,
        input: p::MemoryHistoryRequest,
    ) -> Result<p::ListMemoriesResponse> {
        let subject = SubjectId(id(&input.subject_id)?);
        let memory = MemoryId(id(&input.memory_id)?);
        self.0
            .require_memory()?
            .memory(subject, memory, None)
            .await?;
        let scope = format!("history:{}:{}", input.subject_id, input.memory_id);
        let (limit, last) = page(input.page, &scope)?;
        let mut ids=sqlx::query_scalar::<_,Uuid>("SELECT memory_revision_id FROM memory_revisions WHERE subject_id=$1 AND memory_id=$2 AND ($3::uuid IS NULL OR memory_revision_id>$3) ORDER BY memory_revision_id LIMIT $4")
            .bind(subject.0).bind(memory.0).bind(last).bind(limit+1).fetch_all(self.0.store.pool()).await.map_err(db)?;
        let more = ids.len() > limit as usize;
        ids.truncate(limit as usize);
        let next_page_token = if more {
            next_token(&scope, *ids.last().expect("nonempty page"))
        } else {
            String::new()
        };
        let mut items = Vec::new();
        for id in ids {
            items.push(view(
                self.0
                    .require_memory()?
                    .memory(subject, memory, Some(MemoryRevisionId(id)))
                    .await?,
            ));
        }
        Ok(p::ListMemoriesResponse {
            items,
            next_page_token,
        })
    }
}
