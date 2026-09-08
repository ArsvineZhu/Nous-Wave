use crate::*;
use petgraph::{Directed, csr::Csr, visit::EdgeRef};

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

type WeightedEdgeRow = (u32, f64, f64, bool);
type SourceRows = HashMap<u32, Vec<WeightedEdgeRow>>;

fn edge_conductance(
    weight: f64,
    bridge: bool,
    main_mass: f64,
    main_total: f64,
    bridge_total: f64,
    config: WaveConfig,
) -> f64 {
    if bridge {
        let budget = if main_total > 0.0 {
            config.bridge_reserve
        } else {
            config.outbound_budget
        };
        budget * weight / bridge_total.max(f64::EPSILON)
    } else {
        main_mass * weight / main_total.max(f64::EPSILON)
    }
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
    pub fn artifact(&self) -> TopologyArtifact {
        let mut edges = Vec::new();
        for from in 0..self.nodes.len() as u32 {
            for (to, weight, _) in self.outgoing(from) {
                let meta = self.edge_meta.get(&(from, to));
                edges.push((
                    from,
                    to,
                    weight,
                    meta.is_some_and(|meta| meta.bridge),
                    meta.map_or(0.0, |meta| meta.raw),
                ));
            }
        }
        TopologyArtifact {
            generation_id: self.generation_id,
            nodes: self.nodes.clone(),
            edges,
            config: self.config,
        }
    }

    pub fn from_artifact(artifact: TopologyArtifact) -> Result<Self> {
        artifact.config.validate()?;
        for (index, node) in artifact.nodes.iter().enumerate() {
            if node.serving_id as usize != index {
                return Err(Error::Infrastructure(
                    "topology node ordinals are corrupt".into(),
                ));
            }
        }
        let mut csr = Csr::with_nodes(artifact.nodes.len());
        let mut edge_meta = HashMap::new();
        for (from, to, weight, bridge, raw) in artifact.edges {
            if from as usize >= artifact.nodes.len()
                || to as usize >= artifact.nodes.len()
                || !weight.is_finite()
                || weight < 0.0
                || !raw.is_finite()
            {
                return Err(Error::Infrastructure(
                    "topology edge artifact is corrupt".into(),
                ));
            }
            if edge_meta
                .insert((from, to), EdgeMeta { bridge, raw })
                .is_some()
            {
                return Err(Error::Infrastructure(
                    "duplicate persisted topology edge".into(),
                ));
            }
            csr.add_edge(from, to, weight);
        }
        let node_by_ref = artifact
            .nodes
            .iter()
            .map(|node| (node.reference.clone(), node.serving_id))
            .collect();
        Ok(Self {
            generation_id: artifact.generation_id,
            nodes: artifact.nodes,
            node_by_ref,
            csr,
            edge_meta,
            config: artifact.config,
        })
    }
    // Normalization and CSR publication are one invariant-preserving construction step.
    #[allow(clippy::too_many_lines)]
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

        let mut by_source: SourceRows = HashMap::new();
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
                let conductance =
                    edge_conductance(weight, bridge, main_mass, main_total, bridge_total, config);
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyArtifact {
    pub generation_id: ServingGenerationId,
    pub nodes: Vec<WaveNode>,
    pub edges: Vec<(u32, u32, f64, bool, f64)>,
    pub config: WaveConfig,
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
