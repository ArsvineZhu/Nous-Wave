use crate::{CognitiveRuntimeService, QueryPlan};
use nous_core::*;
use std::collections::{BTreeMap, HashSet};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait CognitiveContributor: Send + Sync {
    async fn contribute(
        &self,
        query: &CognitiveQuery,
        plan: &QueryPlan,
    ) -> Result<CognitiveQueryResult>;
}

impl CognitiveRuntimeService {
    pub async fn query(
        &self,
        query: CognitiveQuery,
        memory: Option<&dyn CognitiveContributor>,
    ) -> Result<CognitiveQueryResult> {
        let plan = QueryPlan::for_query(&query);
        self.query_with_plan(query, memory, plan).await
    }

    pub async fn query_with_plan(
        &self,
        query: CognitiveQuery,
        memory: Option<&dyn CognitiveContributor>,
        plan: QueryPlan,
    ) -> Result<CognitiveQueryResult> {
        query.validate()?;
        self.require_subject(query.subject).await?;
        if let Some(session) = query.session {
            self.require_session(query.subject, session).await?;
        }
        let mut result = if let Some(memory) = memory {
            memory.contribute(&query, &plan).await?
        } else {
            if query
                .targets
                .iter()
                .any(|target| matches!(target, QueryTarget::Memory))
            {
                return Err(Error::Unavailable("Memory MicroSystem is disabled".into()));
            }
            CognitiveQueryResult {
                query_id: Uuid::now_v7(),
                generation: QueryGenerationTrace::default(),
                status: QueryStatus::Complete,
                results: Vec::new(),
                resource_actions: Vec::new(),
                degradation: Vec::new(),
                diagnostics: None,
            }
        };
        let mut seen: HashSet<CognitiveRef> = result
            .results
            .iter()
            .map(|hit| hit.reference.clone())
            .collect();
        for reference in query.targets.iter().filter_map(|target| match target {
            QueryTarget::Exact { reference } => Some(reference),
            _ => None,
        }) {
            self.store
                .validate_reference(query.subject, reference)
                .await?;
            if seen.insert(reference.clone()) {
                result.results.push(reference_hit(
                    reference.clone(),
                    EvidenceFamily::Exact,
                    &query,
                ));
            }
        }
        let runtime_allowed = query.targets.is_empty()
            || query.targets.iter().any(|target| {
                matches!(
                    target,
                    QueryTarget::AnyRelevantCognition | QueryTarget::Evidence
                )
            });
        if runtime_allowed && let Some(session) = query.session {
            for resident in self.session(query.subject, session).await?.resident {
                if seen.insert(resident.reference.clone()) {
                    result.results.push(reference_hit(
                        resident.reference,
                        EvidenceFamily::Runtime,
                        &query,
                    ));
                }
            }
        }
        let (actions, degradation) = self.resource_actions_for_query(&query, &plan).await?;
        result.resource_actions = actions;
        result.degradation.extend(degradation);
        result.results.sort_by(|a, b| {
            b.match_evidence
                .final_score
                .total_cmp(&a.match_evidence.final_score)
                .then_with(|| a.reference.to_string().cmp(&b.reference.to_string()))
        });
        result.results.truncate(query.result_need.limit);
        if !result.degradation.is_empty() {
            result.status = if result
                .degradation
                .iter()
                .any(|d| d.code == "required_external_authority_unresolved")
            {
                QueryStatus::Partial
            } else {
                QueryStatus::Degraded
            };
        }
        explain_plan(&mut result, &query, &plan);
        Ok(result)
    }
}

fn explain_plan(result: &mut CognitiveQueryResult, query: &CognitiveQuery, plan: &QueryPlan) {
    if query.diagnostics != DiagnosticsRequest::None {
        let diagnostics = result.diagnostics.get_or_insert(QueryDiagnostics {
            candidate_counts: BTreeMap::new(),
            lane_status: BTreeMap::new(),
            wave_observability: None,
            trace: None,
        });
        diagnostics.lane_status.insert(
            "effort".into(),
            format!("{:?}", query.effort).to_lowercase(),
        );
        diagnostics.lane_status.insert(
            "cue_sensing_plan".into(),
            if plan.sense_cues {
                "enabled"
            } else {
                "skipped"
            }
            .into(),
        );
        diagnostics.lane_status.insert(
            "topology_plan".into(),
            if plan.expand_topology {
                "enabled"
            } else {
                "skipped"
            }
            .into(),
        );
        diagnostics
            .candidate_counts
            .insert("planned_candidate_bound".into(), plan.candidate_limit);
        diagnostics.lane_status.insert(
            "resource_plan".into(),
            format!(
                "limit={},synopsis_preferred={}",
                plan.resource_limit, plan.prefer_resource_synopsis
            ),
        );
    }
}

fn reference_hit(
    reference: CognitiveRef,
    family: EvidenceFamily,
    query: &CognitiveQuery,
) -> CognitiveHit {
    let authority = match reference {
        CognitiveRef::Memory(_)
        | CognitiveRef::MemoryRevision(_)
        | CognitiveRef::Tag(_)
        | CognitiveRef::Anchor(_) => AuthorityClass::SubjectCognition,
        CognitiveRef::Resource(_) => AuthorityClass::ResourceDescriptor,
        CognitiveRef::DerivedRepresentation(_) | CognitiveRef::DerivedRegion(_) => {
            AuthorityClass::Interpretation
        }
        _ => AuthorityClass::Evidence,
    };
    CognitiveHit {
        reference: reference.clone(),
        revision: None,
        semantic_role: Some("reference".into()),
        memory_class: None,
        representation: None,
        authority,
        freshness: FreshnessDescriptor {
            observed_at: None,
            valid_from: None,
            valid_to: None,
        },
        entity_refs: Vec::new(),
        evidence: if query.result_need.need_evidence {
            vec![EvidenceHandle {
                reference: reference.clone(),
                support_role: "source".into(),
            }]
        } else {
            Vec::new()
        },
        match_evidence: MatchEvidence {
            families: vec![family],
            base_rank_score: 1.0,
            field_contact: 0.0,
            structural_score: 0.0,
            topology_innovation: 0.0,
            wave_observability: 0.0,
            direct_seed_evidence: 1.0,
            final_score: 1.0,
            variants: Vec::new(),
            explanation: None,
        },
        materialization: if query.result_need.need_materialization_handles {
            vec![MaterializationHandle {
                reference,
                level: "source".into(),
            }]
        } else {
            Vec::new()
        },
        supersession_state: None,
    }
}
