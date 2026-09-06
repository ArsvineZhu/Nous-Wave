use nous_core::{Error, Result};
use nous_memory_domain::recall::NodeRef;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationEdge {
    pub evidence_id: Uuid,
    pub from: NodeRef,
    pub to: NodeRef,
    pub evidence_class: String,
    pub support: f64,
    pub target_inbound_degree: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivationConfig {
    pub outbound_mass: f64,
    pub backtrack_factor: f64,
    pub min_activation: f64,
    pub max_hops: usize,
}
impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            outbound_mass: 0.95,
            backtrack_factor: 0.12,
            min_activation: 0.00001,
            max_hops: 8,
        }
    }
}
impl ActivationConfig {
    pub fn validate(&self) -> Result<()> {
        if !self.outbound_mass.is_finite()
            || !(0.0..1.0).contains(&self.outbound_mass)
            || !self.backtrack_factor.is_finite()
            || !(0.0..=1.0).contains(&self.backtrack_factor)
            || !self.min_activation.is_finite()
            || self.min_activation <= 0.0
            || self.max_hops == 0
            || self.max_hops > 64
        {
            return Err(Error::Invalid(
                "invalid bounded activation configuration".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationSupport {
    pub seed: NodeRef,
    pub target: NodeRef,
    pub activation: f64,
    pub path: Vec<Uuid>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivationTrace {
    pub states_expanded: usize,
    pub edges_visited: usize,
    pub max_depth: usize,
    pub remaining_frontier: usize,
    pub activation_seed_count: usize,
    pub actual_flow_edges: usize,
    pub positive_flow_entropy: f64,
    pub emergent_support_ratio: f64,
    pub budget_truncated: bool,
}

#[derive(Debug, Clone)]
struct Frontier {
    seed: NodeRef,
    node: NodeRef,
    previous: Option<NodeRef>,
    mass: f64,
    path: Vec<Uuid>,
}
impl PartialEq for Frontier {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Frontier {}
impl PartialOrd for Frontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Frontier {
    fn cmp(&self, other: &Self) -> Ordering {
        self.mass
            .total_cmp(&other.mass)
            .then_with(|| other.node.cmp(&self.node))
            .then_with(|| other.seed.cmp(&self.seed))
            .then_with(|| other.path.cmp(&self.path))
    }
}

/// The frontier is owned by a process-local Cognitive Work Cycle. It can be
/// advanced with additional budgets without re-expanding exhausted regions.
#[derive(Debug, Clone, Default)]
pub struct ActivationState {
    frontier: BinaryHeap<Frontier>,
    best: BTreeMap<(NodeRef, NodeRef), ActivationSupport>,
    expanded: BTreeSet<(NodeRef, NodeRef)>,
    seeds: BTreeSet<NodeRef>,
}
impl ActivationState {
    pub fn seed(&mut self, seeds: &[NodeRef], max_states: usize) {
        for seed in seeds.iter().take(max_states) {
            if self.best.len() >= max_states {
                break;
            }
            if self.seeds.insert(*seed) {
                let support = ActivationSupport {
                    seed: *seed,
                    target: *seed,
                    activation: 1.0,
                    path: vec![],
                };
                self.best.insert((*seed, *seed), support);
                self.frontier.push(Frontier {
                    seed: *seed,
                    node: *seed,
                    previous: None,
                    mass: 1.0,
                    path: vec![],
                });
            }
        }
    }

    pub fn advance(
        &mut self,
        edges: &[AssociationEdge],
        config: &ActivationConfig,
        state_budget: usize,
        edge_budget: usize,
        total_state_limit: usize,
    ) -> Result<ActivationTrace> {
        config.validate()?;
        let mut adjacency: BTreeMap<NodeRef, Vec<&AssociationEdge>> = BTreeMap::new();
        for edge in edges {
            if !edge.support.is_finite() || edge.support < 0.0 {
                return Err(Error::Invalid(
                    "association support must be finite and nonnegative".into(),
                ));
            }
            adjacency.entry(edge.from).or_default().push(edge);
        }
        for neighbors in adjacency.values_mut() {
            neighbors.sort_by_key(|edge| (edge.to, edge.evidence_id));
        }
        let mut trace = ActivationTrace::default();
        let mut positive_masses = BTreeMap::<Uuid, f64>::new();
        let mut emergent_mass = 0.0_f64;
        while trace.states_expanded < state_budget && trace.edges_visited < edge_budget {
            let Some(current) = self.frontier.pop() else {
                break;
            };
            if self.expanded.contains(&(current.seed, current.node))
                || self
                    .best
                    .get(&(current.seed, current.node))
                    .is_some_and(|best| current.mass < best.activation)
            {
                continue;
            }
            if current.path.len() >= config.max_hops {
                self.expanded.insert((current.seed, current.node));
                continue;
            }
            let Some(neighbors) = adjacency.get(&current.node) else {
                self.expanded.insert((current.seed, current.node));
                continue;
            };
            // Do not consume a partial adjacency and pretend its region is exhausted.
            if neighbors.len() > edge_budget - trace.edges_visited {
                trace.budget_truncated = true;
                self.frontier.push(current);
                break;
            }
            self.expanded.insert((current.seed, current.node));
            trace.states_expanded += 1;
            trace.edges_visited += neighbors.len();
            let conductance = |edge: &&AssociationEdge| {
                edge.support.ln_1p() / (1.0 + (edge.target_inbound_degree as f64).ln_1p())
            };
            let total: f64 = neighbors.iter().map(conductance).sum();
            if total <= 0.0 {
                continue;
            }
            for edge in neighbors {
                let backtrack = if current.previous == Some(edge.to) {
                    config.backtrack_factor
                } else {
                    1.0
                };
                let mass =
                    current.mass * config.outbound_mass * conductance(edge) / total * backtrack;
                if mass < config.min_activation {
                    continue;
                }
                let key = (current.seed, edge.to);
                if self
                    .best
                    .get(&key)
                    .is_some_and(|prior| prior.activation >= mass)
                {
                    continue;
                }
                if !self.best.contains_key(&key) && self.best.len() >= total_state_limit {
                    trace.budget_truncated = true;
                    continue;
                }
                *positive_masses.entry(edge.evidence_id).or_default() += mass;
                if !self.seeds.contains(&edge.to) {
                    emergent_mass += mass;
                }
                let mut path = current.path.clone();
                path.push(edge.evidence_id);
                trace.max_depth = trace.max_depth.max(path.len());
                self.best.insert(
                    key,
                    ActivationSupport {
                        seed: current.seed,
                        target: edge.to,
                        activation: mass,
                        path: path.clone(),
                    },
                );
                self.frontier.push(Frontier {
                    seed: current.seed,
                    node: edge.to,
                    previous: Some(current.node),
                    mass,
                    path,
                });
            }
        }
        trace.remaining_frontier = self.frontier.len();
        trace.activation_seed_count = self.seeds.len();
        trace.actual_flow_edges = positive_masses.len();
        trace.budget_truncated |= trace.remaining_frontier > 0
            && (trace.states_expanded >= state_budget || trace.edges_visited >= edge_budget);
        let total_mass = positive_masses.values().sum::<f64>();
        if positive_masses.len() >= 2 && total_mass.is_finite() && total_mass > 0.0 {
            let entropy = positive_masses
                .values()
                .map(|mass| {
                    let probability = *mass / total_mass;
                    -probability * probability.ln()
                })
                .sum::<f64>();
            trace.positive_flow_entropy =
                (entropy / (positive_masses.len() as f64).ln()).clamp(0.0, 1.0);
        } else {
            trace.positive_flow_entropy = 0.0;
        }
        trace.emergent_support_ratio = if total_mass > 0.0 {
            (emergent_mass / (total_mass + self.seeds.len() as f64)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Ok(trace)
    }

    pub fn supports(&self) -> Vec<ActivationSupport> {
        self.best.values().cloned().collect()
    }
    pub fn scores(&self) -> BTreeMap<NodeRef, f64> {
        let mut scores = BTreeMap::new();
        let divisor = self.seeds.len().max(1) as f64;
        for support in self.best.values() {
            *scores.entry(support.target).or_insert(0.0) += support.activation / divisor;
        }
        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nous_memory_domain::recall::NodeKind;
    fn node(id: u128) -> NodeRef {
        NodeRef {
            kind: NodeKind::Memory,
            id: Uuid::from_u128(id),
        }
    }
    fn edge(from: u128, to: u128, hub: usize) -> AssociationEdge {
        AssociationEdge {
            evidence_id: Uuid::from_u128(from * 10000 + to),
            from: node(from),
            to: node(to),
            evidence_class: "host_explicit".into(),
            support: 10.0,
            target_inbound_degree: hub,
        }
    }
    #[test]
    fn outbound_mass_is_bounded_and_hubs_compete() {
        let edges: Vec<_> = (2..1002)
            .map(|to| edge(1, to, if to == 2 { 10000 } else { 1 }))
            .collect();
        let mut state = ActivationState::default();
        state.seed(&[node(1)], 2000);
        let trace = state
            .advance(&edges, &ActivationConfig::default(), 1, 2000, 2000)
            .unwrap();
        assert_eq!(trace.edges_visited, 1000);
        let scores = state.scores();
        let outgoing: f64 = scores
            .iter()
            .filter(|(node, _)| node.id != Uuid::from_u128(1))
            .map(|(_, score)| score)
            .sum();
        assert!(outgoing <= 0.950000001);
        assert!(scores[&node(2)] < scores[&node(3)]);
    }
    #[test]
    fn incremental_private_path_reuses_work_and_does_not_oscillate() {
        let edges = vec![edge(1, 2, 1), edge(2, 1, 1), edge(2, 3, 1), edge(3, 4, 1)];
        let mut state = ActivationState::default();
        state.seed(&[node(1)], 100);
        let first = state
            .advance(&edges, &ActivationConfig::default(), 1, 10, 100)
            .unwrap();
        assert_eq!(first.edges_visited, 1);
        let next = state
            .advance(&edges, &ActivationConfig::default(), 10, 10, 100)
            .unwrap();
        assert_eq!(next.edges_visited, 3);
        assert!(state.scores()[&node(4)] > 0.0);
        assert_eq!(state.scores()[&node(1)], 1.0);
        assert_eq!(
            state
                .advance(&edges, &ActivationConfig::default(), 10, 10, 100)
                .unwrap()
                .edges_visited,
            0
        );
    }
}
