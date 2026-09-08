use crate::*;
use rayon::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateRankInput {
    pub reference: CognitiveRef,
    pub family_ranks: HashMap<EvidenceFamily, usize>,
    pub topology: CandidateTopologyObservation,
    pub trail: Option<CandidateSemanticTrail>,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedCandidate {
    pub reference: CognitiveRef,
    pub base_rank_score: f64,
    pub final_score: f64,
    pub field_contact: f64,
    pub structural_score: f64,
    pub topology_innovation: f64,
    pub wave_observability: f64,
    pub direct_seed_evidence: f64,
    pub families: Vec<EvidenceFamily>,
    pub variants: Vec<String>,
}

const FAMILY_WEIGHTS: &[(EvidenceFamily, f64)] = &[
    (EvidenceFamily::Exact, 4.0),
    (EvidenceFamily::Runtime, 2.0),
    (EvidenceFamily::Entity, 2.5),
    (EvidenceFamily::Lexical, 1.5),
    (EvidenceFamily::SemanticDense, 1.5),
    (EvidenceFamily::TagDirect, 1.5),
    (EvidenceFamily::AnchorDirect, 1.5),
    (EvidenceFamily::Temporal, 1.0),
    (EvidenceFamily::WaveField, 1.0),
    (EvidenceFamily::Resource, 2.0),
    (EvidenceFamily::LanguageRerank, 1.0),
];

pub fn rank_candidates(
    candidates: &[CandidateRankInput],
    observability: f64,
) -> Vec<RankedCandidate> {
    let enabled = FAMILY_WEIGHTS
        .iter()
        .filter(|(family, _)| {
            candidates
                .iter()
                .any(|candidate| candidate.family_ranks.contains_key(family))
        })
        .copied()
        .collect::<Vec<_>>();
    let denominator: f64 = enabled
        .iter()
        .map(|(_, weight)| weight / 61.0)
        .sum::<f64>()
        .max(f64::EPSILON);
    let mut base = candidates
        .par_iter()
        .map(|candidate| {
            let utility = enabled
                .iter()
                .filter_map(|(family, weight)| {
                    candidate
                        .family_ranks
                        .get(family)
                        .map(|rank| weight / (60.0 + *rank as f64))
                })
                .sum::<f64>();
            let mut families = candidate.family_ranks.keys().copied().collect::<Vec<_>>();
            families.sort_by_key(|family| format!("{family:?}"));
            RankedCandidate {
                reference: candidate.reference.clone(),
                base_rank_score: (utility / denominator).clamp(0.0, 1.0),
                final_score: 0.0,
                field_contact: candidate.topology.field_contact.clamp(0.0, 1.0),
                structural_score: candidate.topology.structural_score.clamp(0.0, 1.0),
                topology_innovation: 0.0,
                wave_observability: observability,
                direct_seed_evidence: candidate.topology.direct_seed_evidence.clamp(0.0, 1.0),
                families,
                variants: candidate.variants.clone(),
            }
        })
        .collect::<Vec<_>>();
    for index in 0..base.len() {
        let peers = base
            .iter()
            .enumerate()
            .filter(|(peer_index, peer)| {
                *peer_index != index
                    && (peer.base_rank_score - base[index].base_rank_score).abs() <= 0.05
            })
            .map(|(peer_index, _)| peer_index)
            .collect::<Vec<_>>();
        let innovation = if peers.len() >= 5 {
            let mut structural = peers
                .iter()
                .map(|peer_index| candidates[*peer_index].topology.structural_score)
                .collect::<Vec<_>>();
            structural.sort_by(f64::total_cmp);
            let median = structural[structural.len() / 2];
            (candidates[index].topology.structural_score - median).max(0.0)
        } else {
            0.0
        };
        base[index].topology_innovation = innovation.clamp(0.0, 1.0);
        let topology = &candidates[index].topology;
        base[index].final_score = (base[index].base_rank_score
            + 0.20 * topology.field_contact.clamp(0.0, 1.0)
            + 0.20 * observability.clamp(0.0, 1.0).powf(0.75) * base[index].topology_innovation
            + 0.10 * topology.direct_seed_evidence.clamp(0.0, 1.0))
        .clamp(0.0, 1.5);
    }
    base.sort_by(|left, right| {
        right
            .final_score
            .total_cmp(&left.final_score)
            .then_with(|| left.reference.to_string().cmp(&right.reference.to_string()))
    });
    base
}
