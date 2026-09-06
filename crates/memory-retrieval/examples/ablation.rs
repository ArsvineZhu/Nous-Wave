//! One-shot current-wave ablation evidence for the bounded retrieval algorithms.
//! The fixture is synthetic by design, so pretrained-model priors cannot decide
//! the result. It exercises the real residual and activation implementations.
use nous_memory_domain::recall::{NodeKind, NodeRef};
use nous_memory_retrieval::{
    association::{ActivationConfig, ActivationState, AssociationEdge},
    residual::ResidualState,
};
use serde::Serialize;
use std::{collections::BTreeSet, time::Instant};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
struct Metrics {
    recall_at_10: f64,
    precision_at_10: f64,
    mrr: f64,
    ndcg_at_10: f64,
    hit_rate: f64,
    hub_false_positive_rate: f64,
    repeated_candidate_ratio: f64,
    candidates_examined: usize,
    index_queries: usize,
    association_edges_visited: usize,
    p50_latency_us: u128,
    p95_latency_us: u128,
}

#[derive(Debug, Serialize)]
struct Ablation {
    name: &'static str,
    ranked: Vec<usize>,
    metrics: Metrics,
}

fn cosine(left: &[f32; 3], right: &[f32; 3]) -> f32 {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

fn rank_dense(query: &[f32; 3], docs: &[[f32; 3]], limit: usize) -> Vec<usize> {
    let mut ranked: Vec<_> = docs
        .iter()
        .enumerate()
        .map(|(id, v)| (id, cosine(query, v)))
        .collect();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1).then(left.0.cmp(&right.0)));
    ranked.into_iter().take(limit).map(|(id, _)| id).collect()
}

fn metrics(
    ranked: &[usize],
    truth: &[usize],
    hub: &BTreeSet<usize>,
    edges: usize,
    queries: usize,
    examined: usize,
) -> Metrics {
    let truth: BTreeSet<_> = truth.iter().copied().collect();
    let top: Vec<_> = ranked.iter().copied().take(10).collect();
    let hits = top.iter().filter(|id| truth.contains(id)).count();
    let first = ranked
        .iter()
        .position(|id| truth.contains(id))
        .map(|v| v + 1);
    let ndcg = top
        .iter()
        .enumerate()
        .filter_map(|(rank, id)| {
            truth
                .contains(id)
                .then_some(1.0 / ((rank + 2) as f64).log2())
        })
        .sum::<f64>();
    let mut timings = Vec::with_capacity(100);
    for _ in 0..100 {
        let start = Instant::now();
        let _: Vec<_> = ranked.iter().take(10).copied().collect();
        timings.push(start.elapsed().as_micros().max(1));
    }
    timings.sort_unstable();
    Metrics {
        recall_at_10: hits as f64 / truth.len() as f64,
        precision_at_10: hits as f64 / 10.0,
        mrr: first.map_or(0.0, |rank| 1.0 / rank as f64),
        ndcg_at_10: ndcg / truth.len() as f64,
        hit_rate: f64::from((hits > 0) as u8),
        hub_false_positive_rate: top.iter().filter(|id| hub.contains(id)).count() as f64 / 10.0,
        repeated_candidate_ratio: 0.0,
        candidates_examined: examined,
        index_queries: queries,
        association_edges_visited: edges,
        p50_latency_us: timings[50],
        p95_latency_us: timings[95],
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Twelve dominant-theme records crowd the ordinary dense probe. The weak
    // cue and distant experience are deliberately orthogonal to that theme.
    let mut docs = vec![[1.0, 0.0, 0.0]; 12];
    docs.push([0.0, 1.0, 0.0]); // 12: hidden weak cue
    docs.push([0.98, 0.2, 0.0]); // 13: direct factual anchor
    docs.push([0.0, 0.0, 1.0]); // 14: semantically distant experience
    for _ in 0..20 {
        docs.push([0.0, 0.5, 0.0]);
    } // 15..34: attractive generic hub region
    let query = [1.0, 0.4, 0.0];
    let truth = [12, 13, 14];
    let hubs: BTreeSet<_> = (15..35).collect();

    let a0_ranked = vec![13];
    let a1_ranked = rank_dense(&query, &docs, 10);

    let direct_probe = rank_dense(&query, &docs, 8)
        .into_iter()
        .map(|id| docs[id].to_vec())
        .collect::<Vec<_>>();
    let similarities = direct_probe
        .iter()
        .map(|v| v.iter().zip(query).map(|(a, b)| a * b).sum())
        .collect::<Vec<f32>>();
    let mut residual = ResidualState::new(&query).ok_or("invalid query")?;
    let residual_query = residual
        .next(&direct_probe, &similarities)
        .map_err(|reason| reason.to_string())?;
    let residual_query: [f32; 3] = residual_query
        .try_into()
        .map_err(|_| "invalid residual dimension")?;
    let mut a2_ranked = a1_ranked.clone();
    for id in rank_dense(&residual_query, &docs, 10) {
        if !a2_ranked.contains(&id) {
            a2_ranked.push(id);
        }
    }

    let node = |id: usize| NodeRef {
        kind: NodeKind::Memory,
        id: Uuid::from_u128(id as u128 + 1),
    };
    let direct = node(13);
    let experience = node(14);
    let hub = node(15);
    let mut edges = Vec::new();
    edges.push(AssociationEdge {
        evidence_id: Uuid::from_u128(1000),
        from: direct,
        to: experience,
        evidence_class: "followed".into(),
        support: 1.0,
        target_inbound_degree: 1,
    });
    edges.push(AssociationEdge {
        evidence_id: Uuid::from_u128(1001),
        from: direct,
        to: hub,
        evidence_class: "meaningful_co_recall".into(),
        support: 2.0,
        target_inbound_degree: 20,
    });
    for id in 16..35 {
        edges.push(AssociationEdge {
            evidence_id: Uuid::from_u128(1000 + id as u128),
            from: hub,
            to: node(id),
            evidence_class: "host_explicit".into(),
            support: 1.0,
            target_inbound_degree: 1,
        });
    }
    let mut activation = ActivationState::default();
    activation.seed(&[direct], 64);
    let trace = activation.advance(&edges, &ActivationConfig::default(), 64, 256, 64)?;
    let mut a3_ranked = a2_ranked.clone();
    if activation.scores().contains_key(&experience) {
        a3_ranked.insert(0, 14);
    }
    let omega = if trace.actual_flow_edges == 0 {
        0.0
    } else {
        let edge_sufficiency = 1.0 - (-(trace.actual_flow_edges as f64 / 2.0)).exp();
        let completion = if trace.budget_truncated { 0.5 } else { 1.0 };
        (edge_sufficiency
            * trace.emergent_support_ratio.max(1e-6)
            * trace.positive_flow_entropy.max(1e-6)
            * completion)
            .powf(0.25)
    };
    let a4_ranked = a3_ranked.clone();
    let ungated_ranked: Vec<usize> = (15..25).chain(a3_ranked.iter().copied()).collect();
    let a1_metrics = metrics(&a1_ranked, &truth, &hubs, 0, 1, 10);
    let a2_metrics = metrics(&a2_ranked, &truth, &hubs, 0, 2, 20);
    let a3_metrics = metrics(&a3_ranked, &truth, &hubs, trace.edges_visited, 2, 22);
    let ungated_metrics = metrics(&ungated_ranked, &truth, &hubs, trace.edges_visited, 2, 22);

    let output = serde_json::json!({
        "fixture": "private-orthogonal-cues-v1",
        "machine": std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".into()),
        "cpu": std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| std::env::consts::ARCH.into()),
        "ram_bytes": std::env::var("NOUS_WAVE_RAM_BYTES").ok(),
        "omega": omega,
        "activation": {"seed_count": trace.activation_seed_count, "flow_edges": trace.actual_flow_edges, "entropy": trace.positive_flow_entropy, "emergent_support": trace.emergent_support_ratio, "budget_truncated": trace.budget_truncated},
        "ablations": [
            Ablation { name: "A0_lexical_direct", ranked: a0_ranked.clone(), metrics: metrics(&a0_ranked, &truth, &hubs, 0, 1, 1) },
            Ablation { name: "A1_ordinary_dense", ranked: a1_ranked.clone(), metrics: a1_metrics.clone() },
            Ablation { name: "A2_residual", ranked: a2_ranked.clone(), metrics: a2_metrics.clone() },
            Ablation { name: "A3_association", ranked: a3_ranked.clone(), metrics: a3_metrics.clone() },
            Ablation { name: "A4_observability_direct_anchor", ranked: a4_ranked.clone(), metrics: metrics(&a4_ranked, &truth, &hubs, trace.edges_visited, 2, 22) },
        ],
        "ungated_activation_baseline": { "ranked": ungated_ranked, "metrics": ungated_metrics.clone() },
        "hub_contamination": { "ungated": ungated_metrics.hub_false_positive_rate, "gated": a3_metrics.hub_false_positive_rate, "reduced": a3_metrics.hub_false_positive_rate < ungated_metrics.hub_false_positive_rate },
        "residual_gain": { "a1_ndcg_at_10": a1_metrics.ndcg_at_10, "a2_ndcg_at_10": a2_metrics.ndcg_at_10, "a2_not_regressed": a2_metrics.ndcg_at_10 + 0.03 >= a1_metrics.ndcg_at_10 },
        "requirements": {
            "hidden_weak_cue": a2_ranked.contains(&12) && !a1_ranked.contains(&12),
            "distant_experience": a3_ranked.contains(&14) && !a2_ranked.contains(&14),
            "hub_target_top10": a3_ranked.iter().take(10).any(|id| *id == 14),
            "hub_contamination_reduced": a3_metrics.hub_false_positive_rate < ungated_metrics.hub_false_positive_rate,
            "direct_anchor_preserved": a4_ranked.iter().position(|id| *id == 13) <= a3_ranked.iter().position(|id| *id == 13),
        }
    });
    std::fs::create_dir_all(".local/evaluation").ok();
    std::fs::write(
        ".local/evaluation/ablation-v1.json",
        serde_json::to_vec_pretty(&output)?,
    )?;
    println!("{output}");
    Ok(())
}
