use crate::NousRuntime;
use nous_core::*;
use nous_material::*;
use nous_memory_domain::*;
use nous_memory_service::MemoryFormationRequest;
use std::collections::HashSet;

impl NousRuntime {
    pub async fn observe(&self, input: ObservationInput) -> Result<AcceptedObservation> {
        let mut accepted = self.material.record_observation(input.clone()).await?;
        if matches!(input.formation, FormationDirective::None) { return Ok(accepted); }
        let mut memory_revisions = Vec::new();
        let occurrence_id = accepted.occurrence.occurrence_id;
        let memory = self.require_memory()?;
        if matches!(input.formation, FormationDirective::ExplicitSpecific) {
            let text = match &input.material {
                ObservationMaterial::InlineText { text, .. } => text.clone(),
                _ => {
                    return Err(Error::Invalid(
                        "explicit formation requires inline text or a supplied representation"
                            .into(),
                    ));
                }
            };
            let evidence = vec![MemoryRevisionEvidence {
                evidence_no: 0,
                evidence: EvidenceRef::Occurrence { occurrence_id },
                support_role: SupportRole::Direct,
                weight: Some(1.0),
            }];
            let view = memory
                .form_memory(ExplicitMemoryInput {
                    subject: input.subject,
                    memory_class: MemoryClass::Specific,
                    semantic_role: "episode".into(),
                    representation_text: text,
                    title: None,
                    evidence,
                    entity_refs: input
                        .entities
                        .iter()
                        .filter_map(|mention| mention.entity_ref.clone())
                        .collect(),
                    tags: Vec::new(),
                    occurred_at: input.occurrence.occurred_at,
                    observed_at: input.occurrence.observed_at,
                    valid_from: None,
                    valid_to: None,
                    epistemic_class: EpistemicClass::Reported,
                    confidence: None,
                })
                .await?;
            memory_revisions.push(view.revision.memory_revision_id);
        } else if matches!(input.formation, FormationDirective::ConsiderSpecific)
            && let Some(provider) = &memory.memory_formation_provider
        {
            let text = match &input.material {
                ObservationMaterial::InlineText { text, .. } => Some(text.clone()),
                ObservationMaterial::ArtifactRef { artifact_id } => {
                    let artifact = self.material.artifact(input.subject, *artifact_id).await?;
                    self.material.objects
                        .get(&artifact.content_hash)
                        .await
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                }
                _ => None,
            };
            if let Some(text) = text.filter(|text| !text.trim().is_empty()) {
                let proposals = provider
                    .propose(MemoryFormationRequest {
                        subject: input.subject,
                        occurrence: occurrence_id,
                        source_class: input.occurrence.source_class.clone(),
                        text,
                        entity_refs: input
                            .entities
                            .iter()
                            .filter_map(|mention| mention.entity_ref.clone())
                            .collect(),
                        occurred_at: input.occurrence.occurred_at,
                        observed_at: input.occurrence.observed_at,
                    })
                    .await?;
                if proposals.len() > 32 {
                    return Err(Error::Invalid(
                        "memory formation proposal count exceeds the request bound".into(),
                    ));
                }
                let allowed_entities = input
                    .entities
                    .iter()
                    .filter_map(|mention| mention.entity_ref.clone())
                    .collect::<Vec<_>>();
                let mut proposal_keys = HashSet::new();
                for proposal in proposals {
                    let normalized = proposal
                        .representation_text
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .to_lowercase();
                    let evidence_key = serde_json::to_string(&proposal.evidence)
                        .map_err(|error| Error::Invalid(error.to_string()))?;
                    if !proposal_keys.insert(format!("{normalized}\0{evidence_key}")) {
                        continue;
                    }
                    let view = memory
                        .commit_formation_proposal(input.subject, proposal, &allowed_entities)
                        .await?;
                    memory_revisions.push(view.revision.memory_revision_id);
                }
            }
        }
        if let Some(session) = input.session.filter(|_| input.runtime.admit) {
            for revision in &memory_revisions {
                self.cognition.admit(session, CognitiveRef::MemoryRevision(*revision), "formed", input.runtime.hold_until).await?;
            }
            self.cognition.evict_if_needed(session).await?;
        }
        accepted.memory_revisions = memory_revisions;
        Ok(accepted)
    }
}
