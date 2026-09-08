use nous_core::*;

/// Internal work budgets derived from semantic intent. Engine parameters stay local.
#[derive(Debug, Clone)]
pub struct QueryPlan {
    pub candidate_limit: usize,
    pub sense_cues: bool,
    pub expand_topology: bool,
    pub topology_rounds: usize,
    pub topology_nodes: usize,
    pub resource_limit: usize,
    pub materialize_evidence: bool,
    pub prefer_resource_synopsis: bool,
}

impl QueryPlan {
    pub fn for_query(query: &CognitiveQuery) -> Self {
        let (breadth, sense, rounds, nodes, resources) = match query.effort {
            CognitiveEffort::Light => (2, false, 1, 128, 1),
            CognitiveEffort::Normal => (4, true, 3, 512, 4),
            CognitiveEffort::Deep => (8, true, 5, 2048, 8),
            CognitiveEffort::Maximum => (16, true, 8, 8192, 16),
        };
        let explicit_topology = query
            .cues
            .iter()
            .any(|cue| matches!(cue, Cue::Tag(_) | Cue::Anchor(_)))
            || query.targets.iter().any(|target| {
                matches!(
                    target,
                    QueryTarget::EntityNeighborhood { .. } | QueryTarget::AnchorNeighborhood { .. }
                )
            });
        Self {
            candidate_limit: query
                .result_need
                .limit
                .saturating_mul(breadth)
                .clamp(16, 8192),
            sense_cues: (sense
                || query.capabilities.residual_sensing == RequirementStrength::Required)
                && query.capabilities.residual_sensing != RequirementStrength::Forbidden,
            expand_topology: explicit_topology || query.effort != CognitiveEffort::Light,
            topology_rounds: rounds,
            topology_nodes: nodes,
            resource_limit: resources,
            materialize_evidence: query.result_need.need_evidence
                && matches!(
                    query.effort,
                    CognitiveEffort::Deep | CognitiveEffort::Maximum
                ),
            prefer_resource_synopsis: query.resources.synopsis_only
                || matches!(query.exploration, ExplorationIntent::Global),
        }
    }

    pub fn serving_need(&self, query: &CognitiveQuery) -> ServingNeed {
        let has_text = query
            .cues
            .iter()
            .any(|cue| matches!(cue, Cue::Text(_) | Cue::Example(_)));
        let exact = query.targets.iter().any(|target| {
            matches!(
                target,
                QueryTarget::Exact { .. }
                    | QueryTarget::EntityNeighborhood { .. }
                    | QueryTarget::AnchorNeighborhood { .. }
            )
        }) || query.cues.iter().any(|cue| {
            matches!(
                cue,
                Cue::Entity(_) | Cue::Tag(_) | Cue::Anchor(_) | Cue::Relation(_) | Cue::Resource(_)
            )
        });
        ServingNeed {
            exact,
            lexical: has_text,
            dense: has_text && query.capabilities.text_embedding != RequirementStrength::Forbidden,
            topology: self.expand_topology,
        }
    }
}

/// Ephemeral reuse within a Host call chain; never a durable runtime owner.
#[derive(Debug, Default)]
pub struct WorkCycle {
    pub inspected: std::collections::HashSet<CognitiveRef>,
    pub frontier: Vec<CognitiveRef>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(effort: CognitiveEffort, capabilities: CapabilityPolicy) -> CognitiveQuery {
        CognitiveQuery {
            api_version: API_VERSION,
            subject: SubjectId::new(),
            session: None,
            situation: Default::default(),
            targets: Vec::new(),
            cues: vec![Cue::Text(TextCue {
                text: "semantic query".into(),
            })],
            constraints: Default::default(),
            exploration: ExplorationIntent::None,
            resources: Default::default(),
            result_need: Default::default(),
            effort,
            capabilities,
            diagnostics: DiagnosticsRequest::Summary,
        }
    }

    #[test]
    fn effort_levels_produce_materially_different_work_bounds() {
        let light = QueryPlan::for_query(&query(CognitiveEffort::Light, Default::default()));
        let maximum = QueryPlan::for_query(&query(CognitiveEffort::Maximum, Default::default()));
        assert!(light.candidate_limit < maximum.candidate_limit);
        assert!(light.topology_nodes < maximum.topology_nodes);
        assert!(light.topology_rounds < maximum.topology_rounds);
        assert!(light.resource_limit < maximum.resource_limit);
        assert!(!light.sense_cues);
        assert!(maximum.sense_cues);
    }

    #[test]
    fn residual_strength_overrides_effort_and_forbidden_disables_it() {
        let required = QueryPlan::for_query(&query(
            CognitiveEffort::Light,
            CapabilityPolicy {
                residual_sensing: RequirementStrength::Required,
                ..Default::default()
            },
        ));
        assert!(required.sense_cues);
        let forbidden = QueryPlan::for_query(&query(
            CognitiveEffort::Maximum,
            CapabilityPolicy {
                residual_sensing: RequirementStrength::Forbidden,
                ..Default::default()
            },
        ));
        assert!(!forbidden.sense_cues);
    }

    #[test]
    fn serving_need_follows_semantic_cues_not_global_availability() {
        let text = QueryPlan::for_query(&query(CognitiveEffort::Light, Default::default()));
        let need = text.serving_need(&query(CognitiveEffort::Light, Default::default()));
        assert!(need.lexical);
        assert!(need.dense);
        assert!(!need.topology);
        let tagged = CognitiveQuery {
            cues: vec![Cue::Tag(TagCue { tag: TagId::new() })],
            ..query(CognitiveEffort::Light, Default::default())
        };
        let tagged_plan = QueryPlan::for_query(&tagged);
        let tagged_need = tagged_plan.serving_need(&tagged);
        assert!(tagged_need.exact);
        assert!(tagged_need.topology);
        assert!(!tagged_need.lexical);
        let no_dense = QueryPlan::for_query(&query(
            CognitiveEffort::Maximum,
            CapabilityPolicy {
                text_embedding: RequirementStrength::Forbidden,
                ..Default::default()
            },
        ));
        assert!(
            !no_dense
                .serving_need(&query(
                    CognitiveEffort::Maximum,
                    CapabilityPolicy {
                        text_embedding: RequirementStrength::Forbidden,
                        ..Default::default()
                    },
                ))
                .dense
        );
    }
}
