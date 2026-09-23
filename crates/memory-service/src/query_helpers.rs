use super::*;
use std::collections::HashSet;

#[derive(Debug, Default)]
pub(super) struct CandidateAccumulator {
    references: HashMap<CognitiveRef, CandidateLaneEvidence>,
}

#[derive(Debug, Default)]
struct CandidateLaneEvidence {
    lexical: Option<usize>,
    dense: Option<usize>,
    dense_variants: HashSet<String>,
    postings: HashSet<String>,
    exact: bool,
    runtime: bool,
}

impl CandidateAccumulator {
    pub(super) fn add_lexical(&mut self, reference: CognitiveRef, rank: usize) {
        let entry = self.references.entry(reference).or_default();
        entry.lexical = Some(entry.lexical.map_or(rank, |current| current.min(rank)));
    }

    pub(super) fn add_dense(
        &mut self,
        reference: CognitiveRef,
        rank: usize,
        variant: impl Into<String>,
    ) {
        let entry = self.references.entry(reference).or_default();
        entry.dense = Some(entry.dense.map_or(rank, |current| current.min(rank)));
        entry.dense_variants.insert(variant.into());
    }

    pub(super) fn add_posting(&mut self, reference: CognitiveRef, lane: impl Into<String>) {
        self.references
            .entry(reference)
            .or_default()
            .postings
            .insert(lane.into());
    }

    pub(super) fn mark_exact(&mut self, reference: CognitiveRef) {
        self.references.entry(reference).or_default().exact = true;
    }

    pub(super) fn mark_runtime(&mut self, reference: CognitiveRef) {
        self.references.entry(reference).or_default().runtime = true;
    }

    pub(super) fn is_exact(&self, reference: &CognitiveRef) -> bool {
        self.references
            .get(reference)
            .is_some_and(|evidence| evidence.exact)
    }

    pub(super) fn candidate_refs(&self) -> impl Iterator<Item = &CognitiveRef> {
        self.references.keys()
    }

    pub(super) fn lexical_ranks(&self) -> HashMap<CognitiveRef, usize> {
        self.references
            .iter()
            .filter_map(|(reference, evidence)| {
                evidence.lexical.map(|rank| (reference.clone(), rank))
            })
            .collect()
    }

    pub(super) fn dense_ranks(&self) -> HashMap<CognitiveRef, usize> {
        self.references
            .iter()
            .filter_map(|(reference, evidence)| {
                evidence.dense.map(|rank| (reference.clone(), rank))
            })
            .collect()
    }

    pub(super) fn dense_variants(&self) -> HashMap<CognitiveRef, HashSet<String>> {
        self.references
            .iter()
            .filter(|(_, evidence)| !evidence.dense_variants.is_empty())
            .map(|(reference, evidence)| (reference.clone(), evidence.dense_variants.clone()))
            .collect()
    }

    pub(super) fn posting_hits(&self, reference: &CognitiveRef) -> Option<&HashSet<String>> {
        self.references
            .get(reference)
            .map(|evidence| &evidence.postings)
    }

    pub(super) fn dense_variant_set(&self, reference: &CognitiveRef) -> Option<&HashSet<String>> {
        self.references
            .get(reference)
            .map(|evidence| &evidence.dense_variants)
    }
}

pub(super) fn field_vector(
    wave: &WaveGraphGeneration,
    field: &std::collections::BTreeMap<u32, f64>,
    generation: &DenseGeneration,
) -> Option<Vec<f32>> {
    let mut result = vec![0.0_f64; generation.space.dimension as usize];
    let mut total = 0.0;
    for (&node, &mass) in field {
        let Some(reference) = wave
            .nodes
            .get(node as usize)
            .map(|node| node.reference.clone())
        else {
            continue;
        };
        let Some(record) = generation
            .records()
            .find(|record| record.reference == reference)
        else {
            continue;
        };
        let Some(vector) = generation.vector(record.serving_doc_id) else {
            continue;
        };
        total += mass.max(0.0);
        for (target, value) in result.iter_mut().zip(vector) {
            *target += mass.max(0.0) * f64::from(*value);
        }
    }
    if total <= f64::EPSILON {
        return None;
    }
    let norm = result.iter().map(|value| value * value).sum::<f64>().sqrt();
    if norm <= f64::EPSILON {
        return None;
    }
    Some(
        result
            .into_iter()
            .map(|value| (value / norm) as f32)
            .collect(),
    )
}

pub(super) fn exact_hit(
    reference: &CognitiveRef,
    need_evidence: bool,
    need_materialization: bool,
) -> Option<CognitiveHit> {
    let (authority, role, level) = match reference {
        CognitiveRef::Memory(_) | CognitiveRef::MemoryRevision(_) => (
            AuthorityClass::SubjectCognition,
            "memory",
            "memory_revision",
        ),
        CognitiveRef::Resource(_) => (
            AuthorityClass::ResourceDescriptor,
            "resource_descriptor",
            "resource",
        ),
        CognitiveRef::DerivedRepresentation(_) | CognitiveRef::DerivedRegion(_) => (
            AuthorityClass::Interpretation,
            "derived_representation",
            "evidence_region",
        ),
        CognitiveRef::Session(_) => (AuthorityClass::Evidence, "session", "session"),
        CognitiveRef::Entity(_) | CognitiveRef::Tag(_) | CognitiveRef::Anchor(_) => (
            AuthorityClass::SubjectCognition,
            "topology",
            "cognitive_ref",
        ),
        CognitiveRef::Artifact(_)
        | CognitiveRef::SourceRegion(_)
        | CognitiveRef::Occurrence(_)
        | CognitiveRef::ExternalObject(_) => (AuthorityClass::Evidence, "evidence", "evidence"),
    };
    Some(CognitiveHit {
        reference: reference.clone(),
        revision: None,
        semantic_role: Some(role.into()),
        memory_class: None,
        representation: Some(reference.to_string()),
        authority,
        freshness: FreshnessDescriptor {
            observed_at: None,
            valid_from: None,
            valid_to: None,
        },
        entity_refs: Vec::new(),
        evidence: if need_evidence {
            vec![EvidenceHandle {
                reference: reference.clone(),
                support_role: "exact".into(),
            }]
        } else {
            Vec::new()
        },
        match_evidence: MatchEvidence {
            families: vec![EvidenceFamily::Exact],
            base_rank_score: 1.0,
            field_contact: 0.0,
            structural_score: 0.0,
            topology_innovation: 0.0,
            wave_observability: 0.0,
            direct_seed_evidence: 1.0,
            final_score: 1.0,
            variants: Vec::new(),
            explanation: Some("explicit exact target".into()),
        },
        materialization: if need_materialization {
            vec![MaterializationHandle {
                reference: reference.clone(),
                level: level.into(),
            }]
        } else {
            Vec::new()
        },
        revision_lifecycle: None,
    })
}

pub(super) fn wave_seeds(
    wave: &WaveGraphGeneration,
    query: &CognitiveQuery,
    tag_cues: &[TagId],
    entity_cues: &[EntityRef],
    anchor_cues: &[AnchorId],
    exact_refs: &HashSet<CognitiveRef>,
    residual_seeds: &[(CognitiveRef, f64)],
    embedding_space: Option<&str>,
) -> Vec<WeightedCognitiveSeed> {
    let mut seeds = tag_cues
        .iter()
        .filter_map(|tag| wave.node_id(&CognitiveRef::Tag(*tag)))
        .map(|node| WeightedCognitiveSeed {
            node,
            weight: 1.0,
            family: SeedFamily::Tag,
            origin: SeedOrigin::TagCue,
            provenance: Some("explicit tag cue".into()),
            embedding_space: None,
        })
        .collect::<Vec<_>>();
    seeds.extend(
        entity_cues
            .iter()
            .filter_map(|entity| wave.node_id(&CognitiveRef::Entity(entity.clone())))
            .map(|node| WeightedCognitiveSeed {
                node,
                weight: 1.0,
                family: SeedFamily::Entity,
                origin: SeedOrigin::EntityCue,
                provenance: Some("explicit entity cue".into()),
                embedding_space: None,
            }),
    );
    seeds.extend(
        anchor_cues
            .iter()
            .filter_map(|anchor| wave.node_id(&CognitiveRef::Anchor(*anchor)))
            .map(|node| WeightedCognitiveSeed {
                node,
                weight: 1.0,
                family: SeedFamily::Anchor,
                origin: SeedOrigin::AnchorCue,
                provenance: Some("explicit anchor cue".into()),
                embedding_space: None,
            }),
    );
    for reference in query
        .situation
        .current_refs
        .iter()
        .chain(query.cues.iter().filter_map(|cue| match cue {
            Cue::Relation(relation) => Some(&relation.from),
            _ => None,
        }))
    {
        if let Some(node) = wave.node_id(reference) {
            seeds.push(WeightedCognitiveSeed {
                node,
                weight: 1.0,
                family: SeedFamily::Runtime,
                origin: SeedOrigin::RuntimeSituation,
                provenance: Some(reference.to_string()),
                embedding_space: None,
            });
        }
    }
    for reference in query.cues.iter().filter_map(|cue| match cue {
        Cue::Relation(relation) => Some(&relation.to),
        _ => None,
    }) {
        if let Some(node) = wave.node_id(reference) {
            seeds.push(WeightedCognitiveSeed {
                node,
                weight: 1.0,
                family: SeedFamily::Relation,
                origin: SeedOrigin::RelationCue,
                provenance: Some(reference.to_string()),
                embedding_space: None,
            });
        }
    }
    for resource in query.cues.iter().filter_map(|cue| match cue {
        Cue::Resource(resource) => Some(&resource.resource),
        _ => None,
    }) {
        if let Some(node) = wave.node_id(&CognitiveRef::Resource(resource.clone())) {
            seeds.push(WeightedCognitiveSeed {
                node,
                weight: 1.0,
                family: SeedFamily::Resource,
                origin: SeedOrigin::ResourceCue,
                provenance: Some(resource.as_str().into()),
                embedding_space: None,
            });
        }
    }
    for reference in exact_refs {
        if let Some(node) = wave.node_id(reference) {
            seeds.push(WeightedCognitiveSeed {
                node,
                weight: 1.0,
                family: SeedFamily::Exact,
                origin: SeedOrigin::ExactTarget,
                provenance: Some(reference.to_string()),
                embedding_space: None,
            });
        }
    }
    for (reference, weight) in residual_seeds {
        if let Some(node) = wave.node_id(reference) {
            seeds.push(WeightedCognitiveSeed {
                node,
                weight: *weight,
                family: SeedFamily::Residual,
                origin: SeedOrigin::ResidualDiscovery,
                provenance: Some(format!("residual:{reference}")),
                embedding_space: embedding_space.map(str::to_owned),
            });
        }
    }
    seeds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substring_fallback_never_claims_lexical_family() {
        let reference = CognitiveRef::Memory(MemoryId::new());
        let mut candidates = CandidateAccumulator::default();
        candidates.add_dense(reference.clone(), 1, "substring_fallback");
        candidates.add_dense(reference.clone(), 1, "direct_dense");
        assert!(!candidates.lexical_ranks().contains_key(&reference));
        assert!(candidates.dense_ranks().contains_key(&reference));
        assert!(
            candidates
                .dense_variant_set(&reference)
                .is_some_and(|variants| {
                    variants.contains("substring_fallback") && variants.contains("direct_dense")
                })
        );
    }
}
