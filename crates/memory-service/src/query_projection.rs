use super::query_helpers::exact_hit;
use super::support::*;
use super::*;
use nous_memory_retrieval::RankedCandidate;
use sqlx::Row;
use std::collections::HashSet;

impl MemoryService {
    #[expect(
        clippy::too_many_lines,
        reason = "projection resolves exact refs and ranked memory views as one bounded result assembly"
    )]
    pub(crate) async fn project_memory_results(
        &self,
        query: &CognitiveQuery,
        ranked: Vec<RankedCandidate>,
        views: &[(
            CognitiveRef,
            MemoryId,
            MemoryRevisionId,
            sqlx::postgres::PgRow,
        )],
        materialize_evidence: bool,
    ) -> Result<(Vec<CognitiveHit>, HashSet<CognitiveRef>)> {
        let mut results = Vec::new();
        let mut result_references = HashSet::new();
        for target in &query.targets {
            let QueryTarget::Exact { reference } = target else {
                continue;
            };
            if !self.reference_in_subject(query.subject, reference).await? {
                return Err(Error::Invalid(
                    "exact query reference is outside Subject".into(),
                ));
            }
            if matches!(reference, CognitiveRef::Memory(_)) {
                continue;
            }
            if let CognitiveRef::MemoryRevision(revision)=reference {
                let memory=self.revision(query.subject,*revision).await?;
                if memory.object.status!=MemoryStatus::Active&&!query.constraints.include_suppressed{continue;}
            }

            let exact_authority = match reference {
                CognitiveRef::Resource(_) => AuthorityClass::ResourceDescriptor,
                CognitiveRef::DerivedRepresentation(_) | CognitiveRef::DerivedRegion(_) => {
                    AuthorityClass::Interpretation
                }
                CognitiveRef::Artifact(_)
                | CognitiveRef::SourceRegion(_)
                | CognitiveRef::Occurrence(_)
                | CognitiveRef::ExternalObject(_) => AuthorityClass::Evidence,
                CognitiveRef::Session(_) => AuthorityClass::Evidence,
                CognitiveRef::MemoryRevision(_)
                | CognitiveRef::Entity(_)
                | CognitiveRef::Tag(_)
                | CognitiveRef::Anchor(_) => AuthorityClass::SubjectCognition,
                CognitiveRef::Memory(_) => AuthorityClass::SubjectCognition,
            };
            if query
                .constraints
                .authority
                .is_some_and(|authority| authority != exact_authority)
            {
                continue;
            }
            let Some(hit) = exact_hit(
                reference,
                query.result_need.need_evidence,
                query.result_need.need_materialization_handles,
            ) else {
                continue;
            };
            result_references.insert(reference.clone());
            results.push(hit);
            if results.len() >= query.result_need.limit {
                break;
            }
        }
        for candidate in ranked
            .into_iter()
            .take(query.result_need.limit.saturating_sub(results.len()))
        {
            if result_references.contains(&candidate.reference) {
                continue;
            }
            let Some((_, memory_id, revision, row)) = views
                .iter()
                .find(|(reference, _, _, _)| *reference == candidate.reference)
            else {
                continue;
            };
            let memory_view = self
                .memory(subject_id_from_row(row)?, *memory_id, Some(*revision))
                .await?;
            let evidence = if query.result_need.need_evidence {
                memory_view
                    .evidence
                    .into_iter()
                    .map(|item| EvidenceHandle {
                        reference: item.evidence.cognitive_ref(),
                        support_role: format!("{:?}", item.support_role).to_lowercase(),
                    })
                    .collect()
            } else {
                Vec::new()
            };
            results.push(CognitiveHit {
                reference: candidate.reference.clone(),
                revision: Some(*revision),
                semantic_role: row.try_get("semantic_role").map_err(db)?,
                memory_class: row.try_get("memory_class").map_err(db)?,
                representation: Some(row.try_get("representation_text").map_err(db)?),
                authority: AuthorityClass::SubjectCognition,
                freshness: FreshnessDescriptor {
                    observed_at: memory_view.temporal_evidence.observed_max,
                    valid_from: row.try_get("valid_from").map_err(db)?,
                    valid_to: row.try_get("valid_to").map_err(db)?,
                },
                entity_refs: memory_view.entities,
                evidence,
                match_evidence: MatchEvidence {
                    families: candidate.families,
                    base_rank_score: candidate.base_rank_score,
                    field_contact: candidate.field_contact,
                    structural_score: candidate.structural_score,
                    topology_innovation: candidate.topology_innovation,
                    wave_observability: candidate.wave_observability,
                    direct_seed_evidence: candidate.direct_seed_evidence,
                    final_score: candidate.final_score,
                    variants: candidate.variants,
                    explanation: Some("candidate evidence retained by family".into()),
                },
                materialization: if query.result_need.need_materialization_handles
                    && materialize_evidence
                {
                    vec![MaterializationHandle {
                        reference: CognitiveRef::MemoryRevision(*revision),
                        level: "memory_revision".into(),
                    }]
                } else {
                    Vec::new()
                },
                revision_lifecycle: row.try_get("revision_lifecycle").map_err(db)?,
            });
            result_references.insert(candidate.reference);
        }
        Ok((results, result_references))
    }
}
