use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveObservability {
    pub activity: f64,
    pub emergence: f64,
    pub flow_entropy: f64,
    pub completeness: f64,
    pub omega: f64,
}

pub fn wave_observability(river: &QueryRiver) -> WaveObservability {
    if river.generated_state_mass <= 0.0 {
        return WaveObservability {
            activity: 0.0,
            emergence: 0.0,
            flow_entropy: 0.0,
            completeness: 0.0,
            omega: 0.0,
        };
    }
    let activity = (river.total_edge_flow / river.max_hops.max(1) as f64).clamp(0.0, 1.0);
    let total_potential: f64 = river.node_potential.values().sum();
    let emergent: f64 = river
        .provenance
        .iter()
        .filter(|entry| entry.emergent)
        .map(|entry| entry.potential)
        .sum();
    let emergence = (emergent / total_potential.max(f64::EPSILON)).clamp(0.0, 1.0);
    let flow_total: f64 = river.edges.iter().map(|edge| edge.flow).sum();
    let edge_count = river.edges.len();
    let flow_entropy = if edge_count >= 2 && flow_total > 0.0 {
        let entropy = river
            .edges
            .iter()
            .map(|edge| {
                let probability = edge.flow / flow_total;
                if probability > 0.0 {
                    -probability * probability.ln()
                } else {
                    0.0
                }
            })
            .sum::<f64>();
        (entropy / (edge_count as f64).ln()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let completeness = (1.0
        - river.discarded_state_mass / river.generated_state_mass.max(f64::EPSILON))
    .clamp(0.0, 1.0);
    let omega = (activity * emergence * flow_entropy * completeness).powf(0.25);
    WaveObservability {
        activity,
        emergence,
        flow_entropy,
        completeness,
        omega,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrailOrder {
    Ordered,
    Unordered,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateSemanticTrail {
    pub memory: CognitiveRef,
    pub nodes: Vec<u32>,
    pub order: TrailOrder,
    pub provenance: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CandidateTopologyObservation {
    pub field_contact: f64,
    pub edge_contact: f64,
    pub direction_agreement: f64,
    pub structural_score: f64,
    pub direct_seed_evidence: f64,
}

pub fn field_contact(field: &SparseField, candidate: &[u32], weights: Option<&[f64]>) -> f64 {
    if candidate.is_empty() {
        return 0.0;
    }
    let total: f64 = weights
        .map(|values| values.iter().copied().sum())
        .unwrap_or(candidate.len() as f64)
        .max(f64::EPSILON);
    candidate
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let weight = weights
                .and_then(|values| values.get(index))
                .copied()
                .unwrap_or(1.0)
                / total;
            weight * field.get(node).copied().unwrap_or(0.0)
        })
        .sum()
}

pub fn trail_topology_observation(
    trail: &CandidateSemanticTrail,
    source: &SparseField,
    local: &SparseField,
    transfer: &SparseField,
    river: &QueryRiver,
) -> CandidateTopologyObservation {
    let field = 0.45 * field_contact(source, &trail.nodes, None)
        + 0.35 * field_contact(local, &trail.nodes, None)
        + 0.20 * field_contact(transfer, &trail.nodes, None);
    let direct = field_contact(source, &trail.nodes, None);
    if trail.order != TrailOrder::Ordered || trail.nodes.len() < 2 {
        return CandidateTopologyObservation {
            field_contact: field,
            direct_seed_evidence: direct,
            ..Default::default()
        };
    }
    let mut matches = Vec::new();
    let mut forward_sum = 0.0;
    let mut reverse_sum = 0.0;
    for pair in trail.nodes.windows(2) {
        let forward = river
            .edges
            .iter()
            .find(|edge| edge.from == pair[0] && edge.to == pair[1])
            .map(|edge| edge.flow)
            .unwrap_or(0.0);
        let reverse = river
            .edges
            .iter()
            .find(|edge| edge.from == pair[1] && edge.to == pair[0])
            .map(|edge| edge.flow)
            .unwrap_or(0.0);
        forward_sum += forward;
        reverse_sum += reverse;
        matches.push(forward.max(0.35 * reverse));
    }
    matches.sort_by(f64::total_cmp);
    matches.reverse();
    matches.truncate(4);
    let edge_contact = if matches.is_empty() {
        0.0
    } else {
        matches.iter().sum::<f64>() / matches.len() as f64
    };
    let direction = forward_sum / (forward_sum + reverse_sum).max(f64::EPSILON);
    CandidateTopologyObservation {
        field_contact: field,
        edge_contact,
        direction_agreement: direction,
        structural_score: (field * edge_contact).sqrt() * (0.75 + 0.25 * direction),
        direct_seed_evidence: direct,
    }
}
