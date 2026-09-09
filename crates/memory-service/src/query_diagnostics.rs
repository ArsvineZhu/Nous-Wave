use super::*;
use nous_memory_retrieval::{EpaObservation, ResidualResult};
use std::collections::BTreeMap;

pub(super) struct QueryDiagnosticsInput<'a> {
    pub query: &'a CognitiveQuery,
    pub plan: &'a nous_cognitive_runtime::QueryPlan,
    pub structured_count: usize,
    pub lexical_count: usize,
    pub dense_count: usize,
    pub all_lane_count: usize,
    pub field_dense_count: usize,
    pub executed_candidate_bound: usize,
    pub epa_trace: Option<&'a EpaObservation>,
    pub residual_trace: Option<&'a ResidualResult>,
    pub topology_executed: bool,
    pub observability: f64,
}

pub(super) fn build_query_diagnostics(
    input: QueryDiagnosticsInput<'_>,
) -> Option<QueryDiagnostics> {
    if input.query.diagnostics == DiagnosticsRequest::None {
        return None;
    }
    let mut candidate_counts = BTreeMap::from([
        ("structured".into(), input.structured_count),
        ("lexical".into(), input.lexical_count),
        ("dense".into(), input.dense_count),
        ("all_lanes".into(), input.all_lane_count),
        ("field_dense".into(), input.field_dense_count),
        (
            "executed_candidate_bound".into(),
            input.executed_candidate_bound,
        ),
    ]);
    let lane_status = BTreeMap::from([
        (
            "epa".into(),
            if input.epa_trace.is_some() {
                "ready"
            } else {
                "unavailable"
            }
            .into(),
        ),
        (
            "residual".into(),
            if input.residual_trace.is_some() {
                "ready"
            } else {
                "unavailable"
            }
            .into(),
        ),
        (
            "cue_sensing_executed".into(),
            if input.plan.sense_cues && input.residual_trace.is_some() {
                "yes"
            } else {
                "no"
            }
            .into(),
        ),
        (
            "topology_executed".into(),
            if input.topology_executed { "yes" } else { "no" }.into(),
        ),
    ]);
    if input.topology_executed {
        candidate_counts.insert("topology_budget".into(), input.plan.topology_nodes);
        candidate_counts.insert(
            "executed_topology_rounds".into(),
            input.plan.topology_rounds,
        );
        candidate_counts.insert("executed_topology_nodes".into(), input.plan.topology_nodes);
    }
    Some(QueryDiagnostics {
        candidate_counts,
        lane_status,
        wave_observability: Some(input.observability),
        trace: Some(serde_json::json!({
            "epa": input.epa_trace,
            "residual": input.residual_trace,
        })),
    })
}
