use super::*;
use std::collections::HashSet;

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
        supersession_state: None,
    })
}

pub(super) fn wave_seeds(
    wave: &WaveGraphGeneration,
    query: &CognitiveQuery,
    tag_cues: &[TagId],
    entity_cues: &[EntityRef],
    anchor_cues: &[AnchorId],
    exact_refs: &HashSet<CognitiveRef>,
) -> Vec<SourceSeed> {
    let mut seeds = tag_cues
        .iter()
        .filter_map(|tag| wave.node_id(&CognitiveRef::Tag(*tag)))
        .map(|node| SourceSeed {
            node,
            weight: 1.0,
            seed_family: "tag".into(),
            origin_cue: "query".into(),
            hop_zero: true,
        })
        .collect::<Vec<_>>();
    seeds.extend(
        entity_cues
            .iter()
            .filter_map(|entity| wave.node_id(&CognitiveRef::Entity(entity.clone())))
            .map(|node| SourceSeed {
                node,
                weight: 1.0,
                seed_family: "entity".into(),
                origin_cue: "query".into(),
                hop_zero: true,
            }),
    );
    seeds.extend(
        anchor_cues
            .iter()
            .filter_map(|anchor| wave.node_id(&CognitiveRef::Anchor(*anchor)))
            .map(|node| SourceSeed {
                node,
                weight: 1.0,
                seed_family: "anchor".into(),
                origin_cue: "query".into(),
                hop_zero: true,
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
            seeds.push(SourceSeed {
                node,
                weight: 1.0,
                seed_family: "situation".into(),
                origin_cue: reference.to_string(),
                hop_zero: true,
            });
        }
    }
    for reference in query.cues.iter().filter_map(|cue| match cue {
        Cue::Relation(relation) => Some(&relation.to),
        _ => None,
    }) {
        if let Some(node) = wave.node_id(reference) {
            seeds.push(SourceSeed {
                node,
                weight: 1.0,
                seed_family: "relation".into(),
                origin_cue: reference.to_string(),
                hop_zero: true,
            });
        }
    }
    for resource in query.cues.iter().filter_map(|cue| match cue {
        Cue::Resource(resource) => Some(&resource.resource),
        _ => None,
    }) {
        if let Some(node) = wave.node_id(&CognitiveRef::Resource(resource.clone())) {
            seeds.push(SourceSeed {
                node,
                weight: 1.0,
                seed_family: "resource".into(),
                origin_cue: resource.as_str().into(),
                hop_zero: true,
            });
        }
    }
    for reference in exact_refs {
        if let Some(node) = wave.node_id(reference) {
            seeds.push(SourceSeed {
                node,
                weight: 1.0,
                seed_family: "exact".into(),
                origin_cue: reference.to_string(),
                hop_zero: true,
            });
        }
    }
    seeds
}
