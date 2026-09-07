//! Serving projections and the request-local cognitive retrieval algorithms.
//!
//! This crate never mutates Cognitive Authority. It consumes immutable inputs
//! and returns bounded, evidence-carrying traces.

use arc_swap::ArcSwap;
use nalgebra::DMatrix;
use nous_core::{CognitiveRef, Error, EvidenceFamily, Result, ServingGenerationId};
use petgraph::{Directed, csr::Csr, visit::EdgeRef};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

pub mod dense;
pub mod lexical;
pub mod residual;

pub use dense::{DenseGeneration, DenseMatch, VectorRecord};
pub use lexical::{LexicalDocument, LexicalGeneration, LexicalMatch};
pub use residual::{
    ResidualConfig, ResidualLevel, ResidualResult, SensedTag, residual_pyramid,
    residual_pyramid_with_search,
};

pub type SparseField = BTreeMap<u32, f64>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaveNodeKind {
    Memory,
    Tag,
    Anchor,
    Entity,
    Resource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveNode {
    pub serving_id: u32,
    pub reference: CognitiveRef,
    pub node_kind: WaveNodeKind,
    pub embedding_key: Option<u64>,
    pub posting_key: Option<u64>,
    pub intrinsic_residual_gain: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveEdgeEvidence {
    pub from: CognitiveRef,
    pub to: CognitiveRef,
    pub support_class: String,
    pub association_kind: String,
    pub polarity: String,
    pub support_value: f64,
    pub bridge_hint: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WaveConfig {
    pub hub_beta: f64,
    pub hub_penalty_min: f64,
    pub hub_penalty_max: f64,
    pub outbound_budget: f64,
    pub bridge_reserve: f64,
    pub max_hops: usize,
    pub max_states: usize,
    pub max_neighbors_per_node: usize,
    pub minimum_state_energy: f64,
    pub immediate_return: f64,
    pub initial_path_budget: f64,
    pub normal_edge_cost: f64,
    pub bridge_edge_cost: f64,
    pub fir_gamma: f64,
    pub local_alpha: f64,
    pub local_iterations: usize,
    pub transfer_alpha: f64,
    pub transfer_iterations: usize,
}

impl Default for WaveConfig {
    fn default() -> Self {
        Self {
            hub_beta: 0.35,
            hub_penalty_min: 0.35,
            hub_penalty_max: 1.25,
            outbound_budget: 0.90,
            bridge_reserve: 0.15,
            max_hops: 4,
            max_states: 4096,
            max_neighbors_per_node: 32,
            minimum_state_energy: 1e-4,
            immediate_return: 0.20,
            initial_path_budget: 2.0,
            normal_edge_cost: 1.0,
            bridge_edge_cost: 0.0,
            fir_gamma: 0.55,
            local_alpha: 0.35,
            local_iterations: 4,
            transfer_alpha: 0.72,
            transfer_iterations: 8,
        }
    }
}

impl WaveConfig {
    pub fn validate(&self) -> Result<()> {
        if !(0.0..=1.0).contains(&self.outbound_budget)
            || !(0.0..=self.outbound_budget).contains(&self.bridge_reserve)
            || self.max_hops == 0
            || self.max_states == 0
            || self.max_neighbors_per_node == 0
            || !(0.0..1.0).contains(&self.immediate_return)
            || !(0.0..1.0).contains(&self.fir_gamma)
            || !(0.0..1.0).contains(&self.local_alpha)
            || !(0.0..1.0).contains(&self.transfer_alpha)
            || self.local_iterations == 0
            || self.transfer_iterations == 0
        {
            return Err(Error::Invalid("invalid WaveConfig bounds".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct EdgeMeta {
    bridge: bool,
    raw: f64,
}

#[derive(Debug, Clone)]
pub struct WaveGraphGeneration {
    pub generation_id: ServingGenerationId,
    pub nodes: Vec<WaveNode>,
    node_by_ref: HashMap<CognitiveRef, u32>,
    csr: Csr<(), f64, Directed, u32>,
    edge_meta: HashMap<(u32, u32), EdgeMeta>,
    pub config: WaveConfig,
}

impl WaveGraphGeneration {
    pub fn build(
        mut nodes: Vec<WaveNode>,
        evidence: &[WaveEdgeEvidence],
        config: WaveConfig,
    ) -> Result<Self> {
        config.validate()?;
        nodes.sort_by_key(|node| node.serving_id);
        for (index, node) in nodes.iter_mut().enumerate() {
            node.serving_id = u32::try_from(index)
                .map_err(|_| Error::Invalid("Wave graph has too many nodes".into()))?;
        }
        let node_by_ref = nodes
            .iter()
            .map(|node| (node.reference.clone(), node.serving_id))
            .collect::<HashMap<_, _>>();

        #[derive(Default)]
        struct Aggregate {
            positive: f64,
            negative: f64,
            bridge: bool,
        }

        let mut aggregates = HashMap::<(u32, u32), Aggregate>::new();
        for item in evidence {
            let (Some(&from), Some(&to)) = (node_by_ref.get(&item.from), node_by_ref.get(&item.to))
            else {
                continue;
            };
            if !item.support_value.is_finite() || item.support_value <= 0.0 {
                continue;
            }
            let weighted =
                class_quality(&item.support_class) * (1.0 + item.support_value.max(0.0)).ln();
            let aggregate = aggregates.entry((from, to)).or_default();
            if item.polarity.eq_ignore_ascii_case("negative") || item.polarity == "revoked" {
                aggregate.negative += weighted;
            } else {
                aggregate.positive += weighted;
            }
            aggregate.bridge |=
                item.bridge_hint || item.association_kind.eq_ignore_ascii_case("bridge");
        }

        let mut raw_edges = Vec::new();
        let mut inflow = vec![0.0; nodes.len()];
        for (&(from, to), aggregate) in &aggregates {
            let raw = (aggregate.positive - aggregate.negative).max(0.0);
            if raw > 0.0 {
                inflow[to as usize] += raw;
                raw_edges.push((from, to, raw, aggregate.bridge));
            }
        }
        let mut positive_inflow = inflow
            .iter()
            .copied()
            .filter(|value| *value > 0.0)
            .collect::<Vec<_>>();
        positive_inflow.sort_by(f64::total_cmp);
        let median = if positive_inflow.is_empty() {
            1.0
        } else {
            positive_inflow[positive_inflow.len() / 2].max(f64::EPSILON)
        };

        let mut adjusted = Vec::new();
        for (from, to, raw, bridge) in raw_edges {
            let relative = inflow[to as usize] / median;
            let penalty = relative
                .powf(-config.hub_beta)
                .clamp(config.hub_penalty_min, config.hub_penalty_max);
            adjusted.push((from, to, raw * penalty, raw, bridge));
        }
        adjusted.sort_by_key(|(from, to, ..)| (*from, *to));

        let mut by_source: HashMap<u32, Vec<(u32, f64, f64, bool)>> = HashMap::new();
        for (from, to, weight, raw, bridge) in adjusted {
            by_source
                .entry(from)
                .or_default()
                .push((to, weight, raw, bridge));
        }

        let mut csr = Csr::<(), f64, Directed, u32>::with_nodes(nodes.len());
        let mut edge_meta = HashMap::new();
        for from in 0..nodes.len() as u32 {
            let row = by_source.remove(&from).unwrap_or_default();
            let has_bridge = row.iter().any(|(_, _, _, bridge)| *bridge);
            let main_mass = if has_bridge {
                (config.outbound_budget - config.bridge_reserve).max(0.0)
            } else {
                config.outbound_budget
            };
            let main_total: f64 = row
                .iter()
                .filter(|(_, _, _, bridge)| !*bridge)
                .map(|(_, weight, _, _)| *weight)
                .sum();
            let bridge_total: f64 = row
                .iter()
                .filter(|(_, _, _, bridge)| *bridge)
                .map(|(_, weight, _, _)| *weight)
                .sum();
            for (to, weight, raw, bridge) in row {
                let conductance = if bridge {
                    if bridge_total > 0.0 {
                        config.bridge_reserve * weight / bridge_total
                    } else {
                        0.0
                    }
                } else if main_total > 0.0 {
                    main_mass * weight / main_total
                } else {
                    0.0
                };
                if conductance > 0.0 {
                    csr.add_edge(from, to, conductance);
                    edge_meta.insert((from, to), EdgeMeta { bridge, raw });
                }
            }
        }

        Ok(Self {
            generation_id: ServingGenerationId::new(),
            nodes,
            node_by_ref,
            csr,
            edge_meta,
            config,
        })
    }

    pub fn node_id(&self, reference: &CognitiveRef) -> Option<u32> {
        self.node_by_ref.get(reference).copied()
    }

    pub fn outgoing(&self, node: u32) -> Vec<(u32, f64, bool)> {
        self.csr
            .edges(node)
            .map(|edge| {
                let target = edge.target();
                let meta = self
                    .edge_meta
                    .get(&(node, target))
                    .copied()
                    .unwrap_or(EdgeMeta {
                        bridge: false,
                        raw: 0.0,
                    });
                (target, *edge.weight(), meta.bridge)
            })
            .collect()
    }

    pub fn outbound_mass(&self, node: u32) -> f64 {
        self.outgoing(node).iter().map(|(_, mass, _)| mass).sum()
    }

    pub fn edge_raw_support(&self, from: u32, to: u32) -> Option<f64> {
        self.edge_meta.get(&(from, to)).map(|meta| meta.raw)
    }
}

fn class_quality(class: &str) -> f64 {
    match class {
        "host_explicit" => 1.00,
        "memory_evidence" => 0.90,
        "derived_structure" => 0.75,
        "consolidation" => 0.70,
        "meaningful_use" => 0.50,
        _ => 0.50,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSeed {
    pub node: u32,
    pub weight: f64,
    pub seed_family: String,
    pub origin_cue: String,
    pub hop_zero: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiverEdgeFlow {
    pub from: u32,
    pub to: u32,
    pub flow: f64,
    pub max_single_hop_flow: f64,
    pub minimum_hop: usize,
    pub bridge: bool,
    pub immediate_return: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWaveProvenance {
    pub node: u32,
    pub potential: f64,
    pub first_hop: usize,
    pub emergent: bool,
    pub strongest_parent: Option<u32>,
    pub origin_seeds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRiver {
    pub source_field: SparseField,
    pub node_potential: SparseField,
    pub edges: Vec<RiverEdgeFlow>,
    pub provenance: Vec<NodeWaveProvenance>,
    pub total_edge_flow: f64,
    pub discarded_state_mass: f64,
    pub generated_state_mass: f64,
    pub complete: bool,
    pub max_hops: usize,
}

#[derive(Debug, Clone)]
struct PropagationState {
    previous: Option<u32>,
    current: u32,
    energy: f64,
    path_budget: f64,
    origin: String,
    hop: usize,
}

pub fn propagate(graph: &WaveGraphGeneration, seeds: &[SourceSeed]) -> QueryRiver {
    let config = graph.config;
    let sum: f64 = seeds
        .iter()
        .filter(|seed| seed.weight.is_finite() && seed.weight > 0.0)
        .map(|seed| seed.weight)
        .sum();
    let mut source_field = SparseField::new();
    let mut states = Vec::new();
    if sum > 0.0 {
        for seed in seeds {
            if seed.weight <= 0.0 || !seed.weight.is_finite() {
                continue;
            }
            let energy = seed.weight / sum;
            *source_field.entry(seed.node).or_default() += energy;
            states.push(PropagationState {
                previous: None,
                current: seed.node,
                energy,
                path_budget: config.initial_path_budget,
                origin: seed.origin_cue.clone(),
                hop: 0,
            });
        }
    }

    let fir_weights = (0..=config.max_hops)
        .map(|hop| config.fir_gamma.powi(hop as i32))
        .collect::<Vec<_>>();
    let fir_sum: f64 = fir_weights.iter().sum();
    let mut node_potential = SparseField::new();
    let mut first_hop = HashMap::<u32, usize>::new();
    let mut parents = HashMap::<u32, (u32, f64)>::new();
    let mut origins = HashMap::<u32, HashSet<String>>::new();
    for state in &states {
        *node_potential.entry(state.current).or_default() += state.energy / fir_sum;
        first_hop.entry(state.current).or_insert(0);
        origins
            .entry(state.current)
            .or_default()
            .insert(state.origin.clone());
    }

    let mut flow_map = HashMap::<(u32, u32), RiverEdgeFlow>::new();
    let mut discarded_state_mass = 0.0;
    let mut generated_state_mass = states.iter().map(|state| state.energy).sum::<f64>();
    let mut complete = true;

    for hop in 0..config.max_hops {
        let mut next = HashMap::<(Option<u32>, u32, String), PropagationState>::new();
        for state in states.drain(..) {
            if state.hop != hop {
                continue;
            }
            let mut outgoing = graph.outgoing(state.current);
            outgoing.sort_by(|left, right| {
                right
                    .1
                    .total_cmp(&left.1)
                    .then_with(|| left.0.cmp(&right.0))
            });
            outgoing.truncate(config.max_neighbors_per_node);
            for (target, conductance, bridge) in outgoing {
                let mut flow = state.energy * conductance;
                let immediate_return = state.previous == Some(target);
                if immediate_return {
                    flow *= config.immediate_return;
                }
                if flow < config.minimum_state_energy {
                    continue;
                }
                let entry =
                    flow_map
                        .entry((state.current, target))
                        .or_insert_with(|| RiverEdgeFlow {
                            from: state.current,
                            to: target,
                            flow: 0.0,
                            max_single_hop_flow: 0.0,
                            minimum_hop: hop + 1,
                            bridge,
                            immediate_return,
                        });
                entry.flow += flow;
                entry.max_single_hop_flow = entry.max_single_hop_flow.max(flow);
                entry.minimum_hop = entry.minimum_hop.min(hop + 1);
                entry.bridge |= bridge;
                entry.immediate_return |= immediate_return;
                generated_state_mass += flow;
                let next_budget = state.path_budget
                    - if bridge {
                        config.bridge_edge_cost
                    } else {
                        config.normal_edge_cost
                    };
                if next_budget < 0.0 && !bridge {
                    continue;
                }
                let next_state = PropagationState {
                    previous: Some(state.current),
                    current: target,
                    energy: flow,
                    path_budget: next_budget,
                    origin: state.origin.clone(),
                    hop: hop + 1,
                };
                let key = (
                    next_state.previous,
                    next_state.current,
                    next_state.origin.clone(),
                );
                next.entry(key)
                    .and_modify(|existing| existing.energy += next_state.energy)
                    .or_insert(next_state);
            }
        }
        if next.is_empty() {
            break;
        }
        let mut next_states = next.into_values().collect::<Vec<_>>();
        next_states.sort_by(|left, right| {
            right
                .energy
                .total_cmp(&left.energy)
                .then_with(|| left.current.cmp(&right.current))
        });
        if next_states.len() > config.max_states {
            complete = false;
            let discarded = next_states
                .drain(config.max_states..)
                .map(|state| state.energy)
                .sum::<f64>();
            discarded_state_mass += discarded;
        }
        for state in &next_states {
            let weight = fir_weights[state.hop] / fir_sum;
            *node_potential.entry(state.current).or_default() += state.energy * weight;
            let first = first_hop.entry(state.current).or_insert(state.hop);
            *first = (*first).min(state.hop);
            origins
                .entry(state.current)
                .or_default()
                .insert(state.origin.clone());
            if let Some(parent) = state.previous {
                let replace = parents
                    .get(&state.current)
                    .is_none_or(|(_, energy)| *energy < state.energy);
                if replace {
                    parents.insert(state.current, (parent, state.energy));
                }
            }
        }
        states = next_states;
    }
    if !states.is_empty() && states.iter().any(|state| state.hop >= config.max_hops) {
        complete = false;
    }

    let mut provenance = node_potential
        .iter()
        .map(|(&node, &potential)| NodeWaveProvenance {
            node,
            potential,
            first_hop: first_hop.get(&node).copied().unwrap_or(config.max_hops),
            emergent: !source_field.contains_key(&node),
            strongest_parent: parents.get(&node).map(|(parent, _)| *parent),
            origin_seeds: origins
                .remove(&node)
                .unwrap_or_default()
                .into_iter()
                .collect(),
        })
        .collect::<Vec<_>>();
    provenance.sort_by_key(|entry| entry.node);
    let mut edges = flow_map.into_values().collect::<Vec<_>>();
    edges.sort_by_key(|edge| (edge.from, edge.to));
    let total_edge_flow = edges.iter().map(|edge| edge.flow).sum();
    QueryRiver {
        source_field,
        node_potential,
        edges,
        provenance,
        total_edge_flow,
        discarded_state_mass,
        generated_state_mass,
        complete,
        max_hops: config.max_hops,
    }
}

pub fn bounded_restart_field(
    graph: &WaveGraphGeneration,
    source: &SparseField,
    alpha: f64,
    iterations: usize,
) -> SparseField {
    let mut current = normalize_field(source);
    if current.is_empty() {
        return current;
    }
    for _ in 0..iterations {
        let mut next = SparseField::new();
        for (&node, &mass) in &current {
            for (target, conductance, _) in graph.outgoing(node) {
                *next.entry(target).or_default() += alpha * mass * conductance;
            }
        }
        for (&node, &mass) in source {
            *next.entry(node).or_default() += (1.0 - alpha) * mass;
        }
        current = normalize_field(&next);
    }
    max_normalize(&current)
}

pub fn local_and_transfer_fields(
    graph: &WaveGraphGeneration,
    source: &SparseField,
) -> (SparseField, SparseField) {
    (
        bounded_restart_field(
            graph,
            source,
            graph.config.local_alpha,
            graph.config.local_iterations,
        ),
        bounded_restart_field(
            graph,
            source,
            graph.config.transfer_alpha,
            graph.config.transfer_iterations,
        ),
    )
}

fn normalize_field(field: &SparseField) -> SparseField {
    let total: f64 = field.values().copied().filter(|value| *value > 0.0).sum();
    if total <= 0.0 {
        return SparseField::new();
    }
    field
        .iter()
        .filter(|(_, value)| **value > 0.0)
        .map(|(&node, &value)| (node, value / total))
        .collect()
}

fn max_normalize(field: &SparseField) -> SparseField {
    let max = field.values().copied().fold(0.0, f64::max);
    if max <= 0.0 {
        return SparseField::new();
    }
    field
        .iter()
        .map(|(&node, &value)| (node, value / max))
        .collect()
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpaBasis {
    pub mean: Vec<f64>,
    pub basis: Vec<Vec<f64>>,
    pub singular_energies: Vec<f64>,
    pub embedding_space: String,
}

/// A published, immutable EPA serving projection for one embedding space.
/// The basis is derived from active Tag vectors and is never Authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpaBasisGeneration {
    pub generation_id: ServingGenerationId,
    pub basis: EpaBasis,
}

pub fn build_epa_basis(vectors: &[(u64, Vec<f64>)], embedding_space: &str) -> Option<EpaBasis> {
    if vectors.len() < 8 || vectors.iter().any(|(_, vector)| vector.is_empty()) {
        return None;
    }
    let dimension = vectors[0].1.len();
    if vectors.iter().any(|(_, vector)| vector.len() != dimension) {
        return None;
    }
    let vectors = vectors
        .iter()
        .filter_map(|(key, vector)| {
            let norm = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
            (norm > f64::EPSILON).then(|| {
                (
                    *key,
                    vector.iter().map(|value| *value / norm).collect::<Vec<_>>(),
                )
            })
        })
        .collect::<Vec<_>>();
    if vectors.len() < 8 {
        return None;
    }
    let vectors = if vectors.len() > 256 {
        let last = vectors.len() - 1;
        (0..256)
            .map(|slot| vectors[(slot * last) / 255].clone())
            .collect::<Vec<_>>()
    } else {
        vectors
    };
    let mut mean = vec![0.0; dimension];
    for (_, vector) in &vectors {
        for (target, value) in mean.iter_mut().zip(vector.iter()) {
            *target += *value;
        }
    }
    for value in &mut mean {
        *value /= vectors.len() as f64;
    }
    let data = vectors
        .iter()
        .flat_map(|(_, vector)| {
            vector
                .iter()
                .enumerate()
                .map(|(index, value)| value - mean[index])
        })
        .collect::<Vec<_>>();
    let matrix = DMatrix::from_row_slice(vectors.len(), dimension, &data);
    let svd = matrix.svd(false, true);
    let singular = svd.singular_values.iter().copied().collect::<Vec<_>>();
    let total_energy = singular
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .max(f64::EPSILON);
    let energies = singular
        .iter()
        .map(|value| value * value / total_energy)
        .collect::<Vec<_>>();
    let mut basis = Vec::new();
    if let Some(v_t) = svd.v_t {
        for (row, energy) in v_t.row_iter().zip(&energies) {
            if *energy < 0.01 || basis.len() >= 64 {
                break;
            }
            basis.push(row.iter().copied().collect());
        }
    }
    if basis.is_empty() {
        None
    } else {
        Some(EpaBasis {
            mean,
            basis,
            singular_energies: energies,
            embedding_space: embedding_space.to_owned(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpaObservation {
    pub axis_energy: Vec<f64>,
    pub entropy: f64,
    pub focus: f64,
    pub dominant_axes: Vec<usize>,
    pub resonances: Vec<(usize, usize, f64)>,
}

pub fn observe_epa(basis: &EpaBasis, query: &[f64]) -> Option<EpaObservation> {
    if query.len() != basis.mean.len() || basis.basis.is_empty() {
        return None;
    }
    let centered = query
        .iter()
        .zip(&basis.mean)
        .map(|(query, mean)| query - mean)
        .collect::<Vec<_>>();
    let projections = basis
        .basis
        .iter()
        .map(|axis| axis.iter().zip(&centered).map(|(a, b)| a * b).sum::<f64>())
        .collect::<Vec<_>>();
    let total = projections.iter().map(|value| value * value).sum::<f64>();
    if total <= f64::EPSILON {
        return Some(EpaObservation {
            axis_energy: vec![0.0; projections.len()],
            entropy: 0.0,
            focus: 1.0,
            dominant_axes: Vec::new(),
            resonances: Vec::new(),
        });
    }
    let energy = projections
        .iter()
        .map(|value| value * value / total)
        .collect::<Vec<_>>();
    let denom = (energy.len() as f64).ln().max(1.0);
    let entropy = -energy
        .iter()
        .filter(|value| **value > 0.0)
        .map(|value| value * value.ln())
        .sum::<f64>()
        / denom;
    let dominant_axes = energy
        .iter()
        .enumerate()
        .filter_map(|(index, value)| (*value >= 0.05).then_some(index))
        .collect::<Vec<_>>();
    let mut resonances = Vec::new();
    for left in 0..dominant_axes.len() {
        for right in (left + 1)..dominant_axes.len() {
            let a = dominant_axes[left];
            let b = dominant_axes[right];
            let resonance = (energy[a] * energy[b]).sqrt();
            if resonance >= 0.15 {
                resonances.push((a, b, resonance));
            }
        }
    }
    Some(EpaObservation {
        axis_energy: energy,
        entropy,
        focus: (1.0 - entropy).clamp(0.0, 1.0),
        dominant_axes,
        resonances,
    })
}

#[derive(Clone)]
pub struct ServingSnapshot {
    pub generation: u64,
    pub lexical: Option<Arc<LexicalGeneration>>,
    pub dense: Vec<Arc<DenseGeneration>>,
    pub wave: Option<Arc<WaveGraphGeneration>>,
    pub epa: Vec<Arc<EpaBasisGeneration>>,
    pub postings: Arc<ExactPostings>,
    pub postings_generation: Option<ServingGenerationId>,
}

impl Default for ServingSnapshot {
    fn default() -> Self {
        Self {
            generation: 0,
            lexical: None,
            dense: Vec::new(),
            wave: None,
            epa: Vec::new(),
            postings: Arc::new(ExactPostings::default()),
            postings_generation: None,
        }
    }
}

#[derive(Clone)]
pub struct ServingPublisher {
    current: Arc<ArcSwap<BTreeMap<nous_core::SubjectId, Arc<ServingSnapshot>>>>,
    next_generation: Arc<AtomicU64>,
}

impl Default for ServingPublisher {
    fn default() -> Self {
        Self {
            current: Arc::new(ArcSwap::from_pointee(BTreeMap::new())),
            next_generation: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl ServingPublisher {
    /// Allocate a process-local monotonic generation number.  The database
    /// generation row remains the durable identity; this number is only the
    /// in-process publication trace.
    pub fn next_generation(&self) -> u64 {
        self.next_generation.fetch_add(1, Ordering::Relaxed)
    }

    pub fn snapshot_for(&self, subject: nous_core::SubjectId) -> Arc<ServingSnapshot> {
        self.current
            .load()
            .get(&subject)
            .cloned()
            .unwrap_or_else(|| Arc::new(ServingSnapshot::default()))
    }

    pub fn snapshot(&self) -> Arc<ServingSnapshot> {
        self.current
            .load()
            .values()
            .max_by_key(|snapshot| snapshot.generation)
            .cloned()
            .unwrap_or_else(|| Arc::new(ServingSnapshot::default()))
    }

    pub fn publish_for(&self, subject: nous_core::SubjectId, snapshot: ServingSnapshot) {
        let snapshot = Arc::new(snapshot);
        self.current.rcu(|current| {
            let mut next = current.as_ref().clone();
            next.insert(subject, snapshot.clone());
            Arc::new(next)
        });
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExactPostings {
    postings: HashMap<String, roaring::RoaringBitmap>,
    references: HashMap<u32, Vec<CognitiveRef>>,
}

impl ExactPostings {
    pub fn insert(&mut self, key: impl Into<String>, serving_id: u32) {
        self.postings
            .entry(key.into())
            .or_default()
            .insert(serving_id);
    }

    pub fn insert_reference(
        &mut self,
        key: impl Into<String>,
        serving_id: u32,
        reference: CognitiveRef,
    ) {
        self.insert(key, serving_id);
        let references = self.references.entry(serving_id).or_default();
        if !references.contains(&reference) {
            references.push(reference);
        }
    }

    pub fn get(&self, key: &str) -> Option<&roaring::RoaringBitmap> {
        self.postings.get(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.postings.keys()
    }

    pub fn references(&self, key: &str) -> Vec<CognitiveRef> {
        self.get(key)
            .into_iter()
            .flat_map(|bitmap| bitmap.iter())
            .flat_map(|serving_id| {
                self.references
                    .get(&serving_id)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(kind: &str, id: u128) -> CognitiveRef {
        let uuid = uuid::Uuid::from_u128(id);
        match kind {
            "tag" => CognitiveRef::Tag(nous_core::TagId(uuid)),
            "memory" => CognitiveRef::Memory(nous_core::MemoryId(uuid)),
            _ => CognitiveRef::Anchor(nous_core::AnchorId(uuid)),
        }
    }

    fn graph() -> WaveGraphGeneration {
        let a = reference("tag", 1);
        let b = reference("tag", 2);
        let c = reference("memory", 3);
        WaveGraphGeneration::build(
            vec![
                WaveNode {
                    serving_id: 9,
                    reference: a.clone(),
                    node_kind: WaveNodeKind::Tag,
                    embedding_key: None,
                    posting_key: None,
                    intrinsic_residual_gain: None,
                },
                WaveNode {
                    serving_id: 4,
                    reference: b.clone(),
                    node_kind: WaveNodeKind::Tag,
                    embedding_key: None,
                    posting_key: None,
                    intrinsic_residual_gain: None,
                },
                WaveNode {
                    serving_id: 5,
                    reference: c.clone(),
                    node_kind: WaveNodeKind::Memory,
                    embedding_key: None,
                    posting_key: None,
                    intrinsic_residual_gain: None,
                },
            ],
            &[
                WaveEdgeEvidence {
                    from: a.clone(),
                    to: b.clone(),
                    support_class: "host_explicit".into(),
                    association_kind: "experiential".into(),
                    polarity: "positive".into(),
                    support_value: 1.0,
                    bridge_hint: true,
                },
                WaveEdgeEvidence {
                    from: b,
                    to: c,
                    support_class: "memory_evidence".into(),
                    association_kind: "structural".into(),
                    polarity: "positive".into(),
                    support_value: 2.0,
                    bridge_hint: false,
                },
            ],
            WaveConfig::default(),
        )
        .unwrap()
    }

    #[test]
    fn row_mass_is_bounded_and_bridge_competes_for_reserve() {
        let graph = graph();
        for node in 0..graph.nodes.len() as u32 {
            assert!(graph.outbound_mass(node) <= graph.config.outbound_budget + 1e-9);
        }
    }

    #[test]
    fn river_records_only_actual_flow_and_is_finite() {
        let graph = graph();
        let river = propagate(
            &graph,
            &[SourceSeed {
                node: 0,
                weight: 1.0,
                seed_family: "tag".into(),
                origin_cue: "test".into(),
                hop_zero: true,
            }],
        );
        assert!(river.edges.iter().all(|edge| edge.flow > 0.0));
        assert!(river.provenance.len() <= graph.nodes.len());
        assert!(river.max_hops <= 4);
    }

    #[test]
    fn unordered_trail_has_no_fabricated_path_contact() {
        let graph = graph();
        let river = propagate(
            &graph,
            &[SourceSeed {
                node: 0,
                weight: 1.0,
                seed_family: "tag".into(),
                origin_cue: "test".into(),
                hop_zero: true,
            }],
        );
        let trail = CandidateSemanticTrail {
            memory: reference("memory", 3),
            nodes: vec![0, 1],
            order: TrailOrder::Unordered,
            provenance: None,
        };
        let observation = trail_topology_observation(
            &trail,
            &river.source_field,
            &river.node_potential,
            &river.node_potential,
            &river,
        );
        assert_eq!(observation.edge_contact, 0.0);
        assert_eq!(observation.structural_score, 0.0);
    }

    #[test]
    fn epa_basis_is_bounded_and_observable() {
        let vectors = (0..8_u64)
            .map(|key| {
                (
                    key,
                    vec![
                        1.0 + key as f64,
                        0.5 * (key as f64 + 1.0),
                        (key % 3) as f64 + 0.25,
                    ],
                )
            })
            .collect::<Vec<_>>();
        let basis = build_epa_basis(&vectors, "test-space").expect("EPA basis");
        assert!(basis.basis.len() <= 64);
        assert!(basis.basis.iter().all(|axis| axis.len() == 3));
        let observation = observe_epa(&basis, &[0.5, 1.0, 0.25]).expect("EPA observation");
        assert_eq!(observation.axis_energy.len(), basis.basis.len());
        assert!((0.0..=1.0).contains(&observation.entropy));
    }
}
