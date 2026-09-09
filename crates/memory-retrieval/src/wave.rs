use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSeed {
    pub node: u32,
    pub weight: f64,
    pub seed_family: String,
    pub origin_cue: String,
    pub hop_zero: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedFamily {
    Explicit,
    Exact,
    Entity,
    Tag,
    Anchor,
    Resource,
    Runtime,
    Relation,
    Residual,
}

impl SeedFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit_query",
            Self::Exact => "exact_target",
            Self::Entity => "entity_cue",
            Self::Tag => "tag_cue",
            Self::Anchor => "anchor_cue",
            Self::Resource => "resource_cue",
            Self::Runtime => "runtime_situation",
            Self::Relation => "relation_cue",
            Self::Residual => "residual_discovery",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedOrigin {
    ExplicitQuery,
    ExactTarget,
    EntityCue,
    TagCue,
    AnchorCue,
    ResourceCue,
    RuntimeSituation,
    RelationCue,
    ResidualDiscovery,
}

impl SeedOrigin {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExplicitQuery => "explicit_query",
            Self::ExactTarget => "exact_target",
            Self::EntityCue => "entity_cue",
            Self::TagCue => "tag_cue",
            Self::AnchorCue => "anchor_cue",
            Self::ResourceCue => "resource_cue",
            Self::RuntimeSituation => "runtime_situation",
            Self::RelationCue => "relation_cue",
            Self::ResidualDiscovery => "residual_discovery",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedCognitiveSeed {
    pub node: u32,
    pub weight: f64,
    pub family: SeedFamily,
    pub origin: SeedOrigin,
    pub provenance: Option<String>,
    pub embedding_space: Option<String>,
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
    strongest_energy: f64,
    path_budget: f64,
    origin: String,
    hop: usize,
}

fn merge_propagation_state(existing: &mut PropagationState, next: PropagationState) {
    existing.energy += next.energy;
    let stronger = next.strongest_energy > existing.strongest_energy;
    let tied_and_earlier =
        next.strongest_energy == existing.strongest_energy && next.origin < existing.origin;
    if stronger || tied_and_earlier {
        existing.origin = next.origin;
        existing.path_budget = next.path_budget;
        existing.strongest_energy = next.strongest_energy;
    }
}

// The bounded propagation loop intentionally keeps state, flow and truncation together.
pub fn propagate(graph: &WaveGraphGeneration, seeds: &[SourceSeed]) -> QueryRiver {
    propagate_with_config(graph, seeds, graph.config)
}

pub fn propagate_weighted(
    graph: &WaveGraphGeneration,
    seeds: &[WeightedCognitiveSeed],
) -> QueryRiver {
    propagate_weighted_with_budget(graph, seeds, graph.config.max_hops, graph.config.max_states)
}

pub fn propagate_weighted_with_budget(
    graph: &WaveGraphGeneration,
    seeds: &[WeightedCognitiveSeed],
    max_hops: usize,
    max_states: usize,
) -> QueryRiver {
    let source = seeds
        .iter()
        .map(|seed| SourceSeed {
            node: seed.node,
            weight: seed.weight,
            seed_family: seed.family.as_str().into(),
            origin_cue: seed
                .provenance
                .clone()
                .unwrap_or_else(|| seed.origin.as_str().into()),
            hop_zero: true,
        })
        .collect::<Vec<_>>();
    propagate_with_budget(graph, &source, max_hops, max_states)
}

pub fn propagate_with_budget(
    graph: &WaveGraphGeneration,
    seeds: &[SourceSeed],
    max_hops: usize,
    max_states: usize,
) -> QueryRiver {
    let mut config = graph.config;
    config.max_hops = max_hops.clamp(1, graph.config.max_hops);
    config.max_states = max_states.clamp(1, graph.config.max_states);
    propagate_with_config(graph, seeds, config)
}

#[expect(
    clippy::too_many_lines,
    reason = "bounded propagation is one algorithmic kernel: state, flow, and truncation belong together"
)]
fn propagate_with_config(
    graph: &WaveGraphGeneration,
    seeds: &[SourceSeed],
    config: WaveConfig,
) -> QueryRiver {
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
                strongest_energy: energy,
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
        let mut next = HashMap::<(Option<u32>, u32), PropagationState>::new();
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
                    strongest_energy: flow,
                    path_budget: next_budget,
                    origin: state.origin.clone(),
                    hop: hop + 1,
                };
                let key = (next_state.previous, next_state.current);
                origins
                    .entry(target)
                    .or_default()
                    .insert(state.origin.clone());
                next.entry(key)
                    .and_modify(|existing| merge_propagation_state(existing, next_state.clone()))
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
