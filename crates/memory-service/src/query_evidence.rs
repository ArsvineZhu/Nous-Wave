use super::support::*;
use super::*;
use sqlx::Row;
use std::collections::{HashMap, HashSet};

impl MemoryService {
    // Evidence projection keeps raw, derived, runtime and Resource result semantics together.
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn append_evidence_results(
        &self,
        query: &CognitiveQuery,
        lexical_ranks: &HashMap<CognitiveRef, usize>,
        resident_occurrences: Vec<OccurrenceId>,
        results: &mut Vec<CognitiveHit>,
        result_references: &mut HashSet<CognitiveRef>,
    ) -> Result<()> {
        let target_allows_derived = query.targets.is_empty()
            || query.targets.iter().any(|target| {
                matches!(
                    target,
                    QueryTarget::AnyRelevantCognition | QueryTarget::Evidence
                )
            });
        if target_allows_derived && results.len() < query.result_need.limit {
            for occurrence_id in resident_occurrences {
                let reference = CognitiveRef::Occurrence(occurrence_id);
                if result_references.contains(&reference) {
                    continue;
                }
                let Some(row) = sqlx::query("SELECT o.source_class,o.observed_at,a.content_hash FROM observation_occurrences o LEFT JOIN artifacts a ON a.artifact_id=o.artifact_id WHERE o.subject_id=$1 AND o.occurrence_id=$2")
        .bind(query.subject.0)
        .bind(occurrence_id.0)
        .fetch_optional(self.store.pool())
        .await
        .map_err(db)?
        else {
        continue;
        };
                let source_class: String = row.try_get("source_class").map_err(db)?;
                if (!query.constraints.source_classes_include.is_empty()
                    && !query
                        .constraints
                        .source_classes_include
                        .iter()
                        .any(|class| class.as_str() == source_class))
                    || query
                        .constraints
                        .source_classes_exclude
                        .iter()
                        .any(|class| class.as_str() == source_class)
                    || query
                        .constraints
                        .authority
                        .is_some_and(|authority| authority != AuthorityClass::Evidence)
                {
                    continue;
                }
                let representation = if let Some(hash) = row
                    .try_get::<Option<String>, _>("content_hash")
                    .map_err(db)?
                {
                    self.objects
                        .get(&hash)
                        .await
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                } else {
                    None
                };
                results.push(CognitiveHit {
                    reference: reference.clone(),
                    revision: None,
                    semantic_role: Some("observation".into()),
                    memory_class: None,
                    representation,
                    authority: AuthorityClass::Evidence,
                    freshness: FreshnessDescriptor {
                        observed_at: Some(row.try_get("observed_at").map_err(db)?),
                        valid_from: None,
                        valid_to: None,
                    },
                    entity_refs: Vec::new(),
                    evidence: if query.result_need.need_evidence {
                        vec![EvidenceHandle {
                            reference: reference.clone(),
                            support_role: "runtime_observation".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    match_evidence: MatchEvidence {
                        families: vec![EvidenceFamily::Runtime],
                        base_rank_score: 1.0,
                        field_contact: 0.0,
                        structural_score: 0.0,
                        topology_innovation: 0.0,
                        wave_observability: 0.0,
                        direct_seed_evidence: 0.0,
                        final_score: 1.0,
                        variants: Vec::new(),
                        explanation: Some("resident observation".into()),
                    },
                    materialization: if query.result_need.need_materialization_handles {
                        vec![MaterializationHandle {
                            reference: reference.clone(),
                            level: "evidence_occurrence".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    supersession_state: None,
                });
                result_references.insert(reference);
                if results.len() >= query.result_need.limit {
                    break;
                }
            }
        }
        if target_allows_derived && results.len() < query.result_need.limit {
            let mut derived_candidates = lexical_ranks
                .iter()
                .filter_map(|(reference, rank)| match reference {
                    CognitiveRef::DerivedRepresentation(id) => Some((*rank, *id)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            derived_candidates.sort_by_key(|(rank, id)| (*rank, id.0));
            for (rank, id) in derived_candidates
                .into_iter()
                .take(query.result_need.limit.saturating_sub(results.len()))
            {
                let Some(row) = sqlx::query("SELECT representation_kind,payload_text,source_region_id FROM derived_representations WHERE subject_id=$1 AND derived_representation_id=$2")
        .bind(query.subject.0)
        .bind(id.0)
        .fetch_optional(self.store.pool())
        .await
        .map_err(db)? else {
        continue;
        };
                let source_region = SourceRegionId(row.try_get("source_region_id").map_err(db)?);
                let reference = CognitiveRef::DerivedRepresentation(id);
                if result_references.contains(&reference) {
                    continue;
                }
                if query
                    .constraints
                    .authority
                    .is_some_and(|authority| authority != AuthorityClass::Interpretation)
                {
                    continue;
                }
                results.push(CognitiveHit {
                    reference: reference.clone(),
                    revision: None,
                    semantic_role: Some(row.try_get("representation_kind").map_err(db)?),
                    memory_class: None,
                    representation: row.try_get("payload_text").map_err(db)?,
                    authority: AuthorityClass::Interpretation,
                    freshness: FreshnessDescriptor {
                        observed_at: None,
                        valid_from: None,
                        valid_to: None,
                    },
                    entity_refs: Vec::new(),
                    evidence: if query.result_need.need_evidence {
                        vec![EvidenceHandle {
                            reference: CognitiveRef::SourceRegion(source_region),
                            support_role: "interpretation".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    match_evidence: MatchEvidence {
                        families: vec![EvidenceFamily::Lexical],
                        base_rank_score: 1.0 / (60.0 + rank as f64),
                        field_contact: 0.0,
                        structural_score: 0.0,
                        topology_innovation: 0.0,
                        wave_observability: 0.0,
                        direct_seed_evidence: 0.0,
                        final_score: 1.0 / (60.0 + rank as f64),
                        variants: Vec::new(),
                        explanation: Some("persisted derived textual surrogate".into()),
                    },
                    materialization: if query.result_need.need_materialization_handles {
                        vec![MaterializationHandle {
                            reference: CognitiveRef::SourceRegion(source_region),
                            level: "evidence_region".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    supersession_state: None,
                });
                result_references.insert(reference);
            }
        }
        if target_allows_derived && results.len() < query.result_need.limit {
            let mut occurrence_candidates = lexical_ranks
                .iter()
                .filter_map(|(reference, rank)| match reference {
                    CognitiveRef::Occurrence(id) => Some((*rank, *id)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            occurrence_candidates.sort_by_key(|(rank, id)| (*rank, id.0));
            for (rank, occurrence_id) in occurrence_candidates
                .into_iter()
                .take(query.result_need.limit.saturating_sub(results.len()))
            {
                let reference = CognitiveRef::Occurrence(occurrence_id);
                if result_references.contains(&reference) {
                    continue;
                }
                let Some(row) = sqlx::query("SELECT o.source_class,o.observed_at,a.content_hash FROM observation_occurrences o JOIN artifacts a ON a.artifact_id=o.artifact_id WHERE o.subject_id=$1 AND o.occurrence_id=$2")
        .bind(query.subject.0)
        .bind(occurrence_id.0)
        .fetch_optional(self.store.pool())
        .await
        .map_err(db)? else {
        continue;
        };
                let source_class: String = row.try_get("source_class").map_err(db)?;
                if query
                    .constraints
                    .authority
                    .is_some_and(|authority| authority != AuthorityClass::Evidence)
                {
                    continue;
                }
                if (!query.constraints.source_classes_include.is_empty()
                    && !query
                        .constraints
                        .source_classes_include
                        .iter()
                        .any(|class| class.as_str() == source_class))
                    || query
                        .constraints
                        .source_classes_exclude
                        .iter()
                        .any(|class| class.as_str() == source_class)
                {
                    continue;
                }
                let hash: String = row.try_get("content_hash").map_err(db)?;
                let Ok(bytes) = self.objects.get(&hash).await else {
                    continue;
                };
                let Ok(text) = String::from_utf8(bytes) else {
                    continue;
                };
                results.push(CognitiveHit {
                    reference: reference.clone(),
                    revision: None,
                    semantic_role: Some("observation".into()),
                    memory_class: None,
                    representation: Some(text),
                    authority: AuthorityClass::Evidence,
                    freshness: FreshnessDescriptor {
                        observed_at: Some(row.try_get("observed_at").map_err(db)?),
                        valid_from: None,
                        valid_to: None,
                    },
                    entity_refs: Vec::new(),
                    evidence: if query.result_need.need_evidence {
                        vec![EvidenceHandle {
                            reference: CognitiveRef::Occurrence(occurrence_id),
                            support_role: "direct".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    match_evidence: MatchEvidence {
                        families: vec![EvidenceFamily::Lexical],
                        base_rank_score: 1.0 / (60.0 + rank as f64),
                        field_contact: 0.0,
                        structural_score: 0.0,
                        topology_innovation: 0.0,
                        wave_observability: 0.0,
                        direct_seed_evidence: 0.0,
                        final_score: 1.0 / (60.0 + rank as f64),
                        variants: Vec::new(),
                        explanation: Some("raw observed textual evidence".into()),
                    },
                    materialization: if query.result_need.need_materialization_handles {
                        vec![MaterializationHandle {
                            reference: CognitiveRef::Occurrence(occurrence_id),
                            level: "evidence_occurrence".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    supersession_state: None,
                });
                result_references.insert(reference);
            }
        }
        if query
            .targets
            .iter()
            .any(|target| matches!(target, QueryTarget::Resource))
            && results.len() < query.result_need.limit
        {
            for resource in self.cognition.list_resources(query.subject).await? {
                let descriptor = resource.descriptor;
                if query
                    .constraints
                    .authority
                    .is_some_and(|authority| authority != AuthorityClass::ResourceDescriptor)
                {
                    continue;
                }
                let reference = CognitiveRef::Resource(descriptor.resource_ref.clone());
                if result_references.contains(&reference) {
                    continue;
                }
                results.push(CognitiveHit {
                    reference: reference.clone(),
                    revision: None,
                    semantic_role: Some("resource_descriptor".into()),
                    memory_class: None,
                    representation: descriptor.display_label.clone(),
                    authority: AuthorityClass::ResourceDescriptor,
                    freshness: FreshnessDescriptor {
                        observed_at: Some(descriptor.updated_at),
                        valid_from: None,
                        valid_to: None,
                    },
                    entity_refs: Vec::new(),
                    evidence: if query.result_need.need_evidence {
                        vec![EvidenceHandle {
                            reference: reference.clone(),
                            support_role: "resource_awareness".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    match_evidence: MatchEvidence {
                        families: vec![EvidenceFamily::Resource],
                        base_rank_score: 1.0,
                        field_contact: 0.0,
                        structural_score: 0.0,
                        topology_innovation: 0.0,
                        wave_observability: 0.0,
                        direct_seed_evidence: 0.0,
                        final_score: 1.0,
                        variants: Vec::new(),
                        explanation: Some("resource awareness descriptor".into()),
                    },
                    materialization: Vec::new(),
                    supersession_state: None,
                });
                result_references.insert(reference);
                if results.len() >= query.result_need.limit {
                    break;
                }
            }
        }
        Ok(())
    }
}
