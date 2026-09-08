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
            prefer_resource_synopsis: query.resources.synopsis_only,
        }
    }
}

/// Ephemeral reuse within a Host call chain; never a durable runtime owner.
#[derive(Debug, Default)]
pub struct WorkCycle {
    pub inspected: std::collections::HashSet<CognitiveRef>,
    pub frontier: Vec<CognitiveRef>,
}
