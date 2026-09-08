use crate::*;

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
    strongest_energy: f64,
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
                origins.entry(target).or_default().insert(state.origin.clone());
                next.entry(key)
                    .and_modify(|existing| {
                        existing.energy += next_state.energy;
                        if next_state.strongest_energy > existing.strongest_energy
                            || (next_state.strongest_energy == existing.strongest_energy && next_state.origin < existing.origin) {
                            existing.origin = next_state.origin.clone();
                            existing.path_budget = next_state.path_budget;
                            existing.strongest_energy = next_state.strongest_energy;
                        }
                    })
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
