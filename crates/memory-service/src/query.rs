use super::support::*;
use super::*;

fn field_vector(
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

fn exact_hit(
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

fn wave_seeds(
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

impl LocalRuntime {
    pub async fn query(&self, query: CognitiveQuery) -> Result<CognitiveQueryResult> {
        query.validate()?;
        self.require_subject(query.subject).await?;
        if let Some(session) = query.session {
            self.require_session(query.subject, session).await?;
        }
        let snapshot = self.publisher.snapshot_for(query.subject);
        let mut degradation = Vec::new();
        let has_text_cue = query
            .cues
            .iter()
            .any(|cue| matches!(cue, Cue::Text(_) | Cue::Example(_)));
        if matches!(
            query.capabilities.text_embedding,
            RequirementStrength::Required
        ) && has_text_cue
            && (snapshot.dense.is_empty() || self.text_embedding_provider.is_none())
        {
            return Err(Error::Unavailable(
                "text embedding capability/generation is unavailable".into(),
            ));
        }
        if matches!(
            query.capabilities.text_embedding,
            RequirementStrength::Preferred
        ) && has_text_cue
            && (snapshot.dense.is_empty() || self.text_embedding_provider.is_none())
        {
            degradation.push(Degradation {
                code: "text_embedding_unavailable".into(),
                detail: Some("exact and structured lanes continue".into()),
            });
        }
        if matches!(
            query.capabilities.residual_sensing,
            RequirementStrength::Required
        ) && has_text_cue
            && (snapshot.dense.is_empty() || self.text_embedding_provider.is_none())
        {
            return Err(Error::Unavailable(
                "residual sensing requires a compatible dense generation".into(),
            ));
        }
        if matches!(
            query.capabilities.residual_sensing,
            RequirementStrength::Preferred
        ) && has_text_cue
            && (snapshot.dense.is_empty() || self.text_embedding_provider.is_none())
        {
            degradation.push(Degradation {
                code: "residual_sensing_unavailable".into(),
                detail: Some("compatible dense Tag vectors are not ready".into()),
            });
        }
        if matches!(
            query.capabilities.text_rerank,
            RequirementStrength::Required
        ) {
            return Err(Error::Unavailable(
                "text rerank capability is not configured".into(),
            ));
        }
        if matches!(
            query.capabilities.text_rerank,
            RequirementStrength::Preferred
        ) {
            degradation.push(Degradation {
                code: "rerank_unavailable".into(),
                detail: Some("no text reranker is configured".into()),
            });
        }
        if snapshot.wave.is_none() && query.exploration != ExplorationIntent::None {
            degradation.push(Degradation {
                code: "wave_generation_unavailable".into(),
                detail: Some("topology lane was not published".into()),
            });
        }
        let pattern = query
            .cues
            .iter()
            .filter_map(|cue| match cue {
                Cue::Text(cue) => Some(cue.text.as_str()),
                Cue::Example(cue) => Some(cue.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_owned();
        let mut lexical_ranks = HashMap::<CognitiveRef, usize>::new();
        if !pattern.is_empty()
            && let Some(lexical) = &snapshot.lexical
        {
            match lexical.search(&pattern, 64) {
                Ok(matches) => {
                    for (rank, item) in matches.into_iter().enumerate() {
                        if let Some(reference) = item.reference {
                            lexical_ranks.entry(reference).or_insert(rank + 1);
                        }
                    }
                }
                Err(error) => degradation.push(Degradation {
                    code: "lexical_generation_unavailable".into(),
                    detail: Some(error.to_string()),
                }),
            }
        }
        let mut dense_ranks = HashMap::<CognitiveRef, usize>::new();
        let mut dense_variants = HashMap::<CognitiveRef, HashSet<String>>::new();
        let mut query_embedding = None;
        if !pattern.is_empty()
            && query.capabilities.text_embedding != RequirementStrength::Forbidden
            && let Some(provider) = &self.text_embedding_provider
        {
            match provider
                .embed(TextEmbeddingRequest {
                    subject: query.subject,
                    text: pattern.clone(),
                    query: true,
                })
                .await
            {
                Ok(output) => {
                    query_embedding = Some(output.clone());
                    let mut matched_space = false;
                    for generation in &snapshot.dense {
                        if !generation.space.compatible_with(&output.space) {
                            continue;
                        }
                        matched_space = true;
                        for (rank, item) in generation
                            .search(&output.vector, 64)
                            .map_err(|error| Error::Infrastructure(error.to_string()))?
                            .into_iter()
                            .enumerate()
                        {
                            if let Some(record) = item.record
                                && matches!(record.reference, CognitiveRef::Memory(_))
                            {
                                dense_ranks
                                    .entry(record.reference.clone())
                                    .or_insert(rank + 1);
                                dense_variants
                                    .entry(record.reference)
                                    .or_default()
                                    .insert("direct_dense".into());
                            }
                        }
                    }
                    if !matched_space {
                        let space_degradation = Degradation {
                            code: "embedding_space_not_ready".into(),
                            detail: Some("no dense generation matches the query space".into()),
                        };
                        if query.capabilities.text_embedding == RequirementStrength::Required {
                            return Err(Error::Unavailable(space_degradation.code.clone()));
                        }
                        degradation.push(space_degradation);
                    }
                }
                Err(error)
                    if query.capabilities.text_embedding == RequirementStrength::Required =>
                {
                    return Err(error);
                }
                Err(error) => degradation.push(Degradation {
                    code: "text_embedding_unavailable".into(),
                    detail: Some(error.to_string()),
                }),
            }
        }
        let exact_refs = query
            .targets
            .iter()
            .filter_map(|target| {
                if let QueryTarget::Exact { reference } = target {
                    Some(reference)
                } else {
                    None
                }
            })
            .cloned()
            .collect::<HashSet<_>>();
        let mut entity_cues = query
            .cues
            .iter()
            .filter_map(|cue| {
                if let Cue::Entity(entity) = cue {
                    Some(entity.entity_ref.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for target in &query.targets {
            if let QueryTarget::EntityNeighborhood { entity_ref } = target
                && !entity_cues.contains(entity_ref)
            {
                entity_cues.push(entity_ref.clone());
            }
        }
        let required_entities = query
            .constraints
            .entity_requirements
            .iter()
            .collect::<Vec<_>>();
        let mut tag_cues = query
            .cues
            .iter()
            .filter_map(|cue| match cue {
                Cue::Tag(tag) => Some(tag.tag),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut anchor_cues = query
            .cues
            .iter()
            .filter_map(|cue| match cue {
                Cue::Anchor(anchor) => Some(anchor.anchor),
                _ => None,
            })
            .collect::<Vec<_>>();
        for target in &query.targets {
            if let QueryTarget::AnchorNeighborhood { anchor } = target
                && !anchor_cues.contains(anchor)
            {
                anchor_cues.push(*anchor);
            }
        }
        let mut residual_trace = None;
        let mut epa_trace = None;
        let mut epa_generation = None;
        if let Some(output) = &query_embedding {
            if query.capabilities.residual_sensing != RequirementStrength::Forbidden
                && let Some(generation) = snapshot
                    .dense
                    .iter()
                    .find(|generation| generation.space.compatible_with(&output.space))
            {
                let tag_by_doc = generation
                    .records()
                    .filter_map(|record| {
                        matches!(record.reference, CognitiveRef::Tag(_))
                            .then_some((record.serving_doc_id, record.reference.clone()))
                    })
                    .collect::<HashMap<_, _>>();
                let residual = residual_pyramid_with_search(
                    &output
                        .vector
                        .iter()
                        .map(|value| f64::from(*value))
                        .collect::<Vec<_>>(),
                    ResidualConfig::default(),
                    |residual, limit| {
                        let residual = residual
                            .iter()
                            .map(|value| *value as f32)
                            .collect::<Vec<_>>();
                        generation
                            .search(&residual, limit.saturating_mul(4).max(limit))
                            .map(|matches| {
                                matches
                                    .into_iter()
                                    .filter_map(|item| item.record)
                                    .filter_map(|record| {
                                        if !matches!(record.reference, CognitiveRef::Tag(_)) {
                                            return None;
                                        }
                                        generation.vector(record.serving_doc_id).map(|vector| {
                                            (
                                                record.serving_doc_id,
                                                vector
                                                    .iter()
                                                    .map(|value| f64::from(*value))
                                                    .collect(),
                                            )
                                        })
                                    })
                                    .take(limit)
                                    .collect::<Vec<_>>()
                            })
                    },
                )?;
                if let Some(residual) = residual {
                    for sensed in residual.levels.iter().flat_map(|level| level.sensed.iter()) {
                        if let Some(CognitiveRef::Tag(tag)) = tag_by_doc.get(&sensed.tag_key)
                            && !tag_cues.contains(tag)
                        {
                            tag_cues.push(*tag);
                        }
                    }
                    residual_trace = Some(residual);
                }
            }
            if let Some(generation) = snapshot
                .epa
                .iter()
                .find(|generation| generation.basis.embedding_space == output.space.space_hash)
            {
                epa_generation = Some(generation.generation_id);
                epa_trace = observe_epa(
                    &generation.basis,
                    &output
                        .vector
                        .iter()
                        .map(|value| f64::from(*value))
                        .collect::<Vec<_>>(),
                );
            }
        }
        if has_text_cue
            && query.capabilities.residual_sensing == RequirementStrength::Preferred
            && residual_trace.is_none()
            && !degradation
                .iter()
                .any(|degradation| degradation.code == "residual_sensing_unavailable")
        {
            degradation.push(Degradation {
                code: "residual_sensing_unavailable".into(),
                detail: Some("no compatible Tag vectors were available".into()),
            });
        }
        if let Some(output) = &query_embedding {
            let mut denoised = output
                .vector
                .iter()
                .map(|value| f64::from(*value))
                .collect::<Vec<_>>();
            if let Some(residual) = &residual_trace
                && let Some(generation) = snapshot
                    .dense
                    .iter()
                    .find(|generation| generation.space.compatible_with(&output.space))
            {
                for sensed in residual.levels.iter().flat_map(|level| level.sensed.iter()) {
                    if let Some(vector) = generation.vector(sensed.tag_key) {
                        for (value, tag_value) in denoised.iter_mut().zip(vector) {
                            *value += 0.25 * sensed.seed_weight * f64::from(*tag_value);
                        }
                    }
                }
                let norm = denoised
                    .iter()
                    .map(|value| value * value)
                    .sum::<f64>()
                    .sqrt();
                if norm > f64::EPSILON {
                    let denoised = denoised
                        .iter()
                        .map(|value| (*value / norm) as f32)
                        .collect::<Vec<_>>();
                    for generation in &snapshot.dense {
                        if !generation.space.compatible_with(&output.space) {
                            continue;
                        }
                        for (rank, item) in
                            generation.search(&denoised, 64)?.into_iter().enumerate()
                        {
                            if let Some(record) = item.record
                                && matches!(record.reference, CognitiveRef::Memory(_))
                            {
                                dense_ranks
                                    .entry(record.reference.clone())
                                    .or_insert(rank + 1);
                                dense_variants
                                    .entry(record.reference)
                                    .or_default()
                                    .insert("denoised_dense".into());
                            }
                        }
                    }
                }
            }
        }
        let mut candidate_memory_ids = HashSet::<Uuid>::new();
        for reference in &exact_refs {
            match reference {
                CognitiveRef::Memory(memory) => {
                    candidate_memory_ids.insert(memory.0);
                }
                CognitiveRef::MemoryRevision(revision) => {
                    if let Some(memory) = sqlx::query_scalar::<_, Uuid>(
                        "SELECT memory_id FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$2",
                    )
                    .bind(query.subject.0)
                    .bind(revision.0)
                    .fetch_optional(self.store.pool())
                    .await
                    .map_err(db)?
                    {
                        candidate_memory_ids.insert(memory);
                    }
                }
                _ => {}
            }
        }
        for reference in query
            .situation
            .current_refs
            .iter()
            .chain(query.targets.iter().filter_map(|target| match target {
                QueryTarget::Exact { reference } => Some(reference),
                _ => None,
            }))
        {
            if let CognitiveRef::Memory(memory) = reference {
                candidate_memory_ids.insert(memory.0);
            }
        }
        let mut posting_keys = Vec::new();
        posting_keys.extend(
            entity_cues
                .iter()
                .map(|entity| CognitiveRef::Entity(entity.clone()).to_string()),
        );
        posting_keys.extend(
            required_entities
                .iter()
                .map(|entity| CognitiveRef::Entity((*entity).clone()).to_string()),
        );
        posting_keys.extend(
            tag_cues
                .iter()
                .map(|tag| CognitiveRef::Tag(*tag).to_string()),
        );
        posting_keys.extend(
            anchor_cues
                .iter()
                .map(|anchor| CognitiveRef::Anchor(*anchor).to_string()),
        );
        posting_keys.extend(
            query
                .situation
                .current_refs
                .iter()
                .filter(|reference| {
                    matches!(
                        reference,
                        CognitiveRef::Entity(_) | CognitiveRef::Tag(_) | CognitiveRef::Anchor(_)
                    )
                })
                .map(|reference| reference.to_string()),
        );
        for key in posting_keys {
            for reference in snapshot.postings.references(&key) {
                match reference {
                    CognitiveRef::Memory(memory) => {
                        candidate_memory_ids.insert(memory.0);
                    }
                    CognitiveRef::MemoryRevision(revision) => {
                        if let Some(memory) = sqlx::query_scalar::<_, Uuid>(
                            "SELECT memory_id FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$2",
                        )
                        .bind(query.subject.0)
                        .bind(revision.0)
                        .fetch_optional(self.store.pool())
                        .await
                        .map_err(db)?
                        {
                            candidate_memory_ids.insert(memory);
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some(wave) = &snapshot.wave {
            let seeds = wave_seeds(
                wave,
                &query,
                &tag_cues,
                &entity_cues,
                &anchor_cues,
                &exact_refs,
            );
            if !seeds.is_empty() {
                let river = propagate(wave, &seeds);
                let mut topology_candidates = river
                    .provenance
                    .iter()
                    .filter_map(|entry| {
                        let reference = wave.nodes.get(entry.node as usize)?.reference.clone();
                        match reference {
                            CognitiveRef::Memory(memory) => Some((entry.potential, memory.0)),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();
                topology_candidates.sort_by(|left, right| {
                    right
                        .0
                        .total_cmp(&left.0)
                        .then_with(|| left.1.cmp(&right.1))
                });
                for (_, memory) in topology_candidates.into_iter().take(128) {
                    candidate_memory_ids.insert(memory);
                }
            }
        }
        for reference in lexical_ranks.keys() {
            if let CognitiveRef::Memory(memory) = reference {
                candidate_memory_ids.insert(memory.0);
            }
        }
        for reference in dense_ranks.keys() {
            if let CognitiveRef::Memory(memory) = reference {
                candidate_memory_ids.insert(memory.0);
            }
        }
        let resident_refs = if let Some(session) = query.session {
            self.resident_reference_strings(session).await?
        } else {
            HashSet::new()
        };
        let mut resident_occurrences = resident_refs
            .iter()
            .filter_map(|value| {
                let (kind, value) = value.split_once(':')?;
                (kind == "occurrence")
                    .then(|| value.parse::<Uuid>().ok())
                    .flatten()
                    .map(OccurrenceId)
            })
            .collect::<Vec<_>>();
        resident_occurrences.sort_unstable();
        for value in &resident_refs {
            if let Some((kind, value)) = value.split_once(':')
                && kind == "memory"
                && let Ok(memory) = value.parse::<Uuid>()
            {
                candidate_memory_ids.insert(memory);
            }
        }
        let target_allows_memory = query.targets.is_empty()
            || query.targets.iter().any(|target| {
                matches!(
                    target,
                    QueryTarget::AnyRelevantCognition
                        | QueryTarget::Memory
                        | QueryTarget::EntityNeighborhood { .. }
                        | QueryTarget::AnchorNeighborhood { .. }
                        | QueryTarget::Exact {
                            reference: CognitiveRef::Memory(_) | CognitiveRef::MemoryRevision(_)
                        }
                )
            });
        let rows = if !target_allows_memory {
            Vec::new()
        } else if !candidate_memory_ids.is_empty() {
            let ids = candidate_memory_ids.iter().copied().collect::<Vec<_>>();
            sqlx::query("SELECT o.memory_id,o.subject_id,o.memory_class,o.current_revision_id,o.status,r.revision_no,r.semantic_role,r.title,r.representation_text,r.epistemic_class,r.confidence,r.occurred_at,r.observed_at,r.valid_from,r.valid_to,r.supersession_state FROM memory_objects o JOIN memory_revisions r ON r.memory_revision_id=o.current_revision_id WHERE o.subject_id=$1 AND ($2 OR o.status='active') AND o.memory_id=ANY($3) ORDER BY r.observed_at DESC LIMIT 2048")
                .bind(query.subject.0)
                .bind(query.constraints.include_suppressed)
                .bind(ids)
                .fetch_all(self.store.pool())
                .await
                .map_err(db)?
        } else if pattern.is_empty() {
            sqlx::query("SELECT o.memory_id,o.subject_id,o.memory_class,o.current_revision_id,o.status,r.revision_no,r.semantic_role,r.title,r.representation_text,r.epistemic_class,r.confidence,r.occurred_at,r.observed_at,r.valid_from,r.valid_to,r.supersession_state FROM memory_objects o JOIN memory_revisions r ON r.memory_revision_id=o.current_revision_id WHERE o.subject_id=$1 AND ($2 OR o.status='active') ORDER BY r.observed_at DESC LIMIT 512")
                .bind(query.subject.0).bind(query.constraints.include_suppressed).fetch_all(self.store.pool()).await.map_err(db)?
        } else if snapshot.lexical.is_some() && !query.constraints.include_suppressed {
            Vec::new()
        } else {
            let terms = pattern
                .split_whitespace()
                .take(16)
                .map(|term| format!("%{}%", term.replace('%', "")))
                .collect::<Vec<_>>();
            sqlx::query("SELECT o.memory_id,o.subject_id,o.memory_class,o.current_revision_id,o.status,r.revision_no,r.semantic_role,r.title,r.representation_text,r.epistemic_class,r.confidence,r.occurred_at,r.observed_at,r.valid_from,r.valid_to,r.supersession_state FROM memory_objects o JOIN memory_revisions r ON r.memory_revision_id=o.current_revision_id WHERE o.subject_id=$1 AND ($2 OR o.status='active') AND r.representation_text ILIKE ANY($3) ORDER BY r.observed_at DESC LIMIT 512")
                .bind(query.subject.0).bind(query.constraints.include_suppressed).bind(&terms).fetch_all(self.store.pool()).await.map_err(db)?
        };
        let mut rank_inputs = Vec::new();
        let mut views = Vec::new();
        for (index, row) in rows.into_iter().enumerate() {
            if !target_allows_memory {
                continue;
            }
            let memory_class: String = row.try_get("memory_class").map_err(db)?;
            if !query.constraints.memory_classes_include.is_empty()
                && !query
                    .constraints
                    .memory_classes_include
                    .iter()
                    .any(|class| class == &memory_class)
            {
                continue;
            }
            if query
                .constraints
                .memory_classes_exclude
                .iter()
                .any(|class| class == &memory_class)
            {
                continue;
            }
            let observed_at: DateTime<Utc> = row.try_get("observed_at").map_err(db)?;
            if !interval_contains(query.constraints.observed, observed_at) {
                continue;
            }
            let memory_id = MemoryId(row.try_get("memory_id").map_err(db)?);
            let revision = MemoryRevisionId(row.try_get("current_revision_id").map_err(db)?);
            let revision_tags = self.revision_tags(revision.0).await?;
            let revision_anchors = self.revision_anchors(revision.0).await?;
            if query
                .constraints
                .authority
                .is_some_and(|authority| authority != AuthorityClass::SubjectCognition)
            {
                continue;
            }
            let source_classes = self.memory_source_classes(revision.0).await?;
            if !query.constraints.source_classes_include.is_empty()
                && !query
                    .constraints
                    .source_classes_include
                    .iter()
                    .any(|class| source_classes.iter().any(|source| source == class.as_str()))
            {
                continue;
            }
            if query
                .constraints
                .source_classes_exclude
                .iter()
                .any(|class| source_classes.iter().any(|source| source == class.as_str()))
            {
                continue;
            }
            let occurred_at: Option<DateTime<Utc>> = row.try_get("occurred_at").map_err(db)?;
            let valid_from: Option<DateTime<Utc>> = row.try_get("valid_from").map_err(db)?;
            let valid_to: Option<DateTime<Utc>> = row.try_get("valid_to").map_err(db)?;
            if !interval_overlaps(query.constraints.occurred, occurred_at, occurred_at) {
                continue;
            }
            if !interval_overlaps(query.constraints.valid, valid_from, valid_to) {
                continue;
            }
            let temporal_cues = query
                .cues
                .iter()
                .filter_map(|cue| match cue {
                    Cue::Temporal(temporal) => Some(temporal.interval),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if !temporal_cues.is_empty()
                && !temporal_cues.iter().any(|interval| {
                    interval_overlaps(Some(*interval), occurred_at, occurred_at)
                        || interval_overlaps(Some(*interval), Some(observed_at), Some(observed_at))
                        || interval_overlaps(Some(*interval), valid_from, valid_to)
                })
            {
                continue;
            }
            if !query.constraints.evidence_classes.is_empty()
                && !self
                    .memory_has_evidence_classes(revision.0, &query.constraints.evidence_classes)
                    .await?
            {
                continue;
            }
            if !query.constraints.modalities.is_empty()
                && !self
                    .memory_has_modalities(revision.0, &query.constraints.modalities)
                    .await?
            {
                continue;
            }
            let reference = CognitiveRef::Memory(memory_id);
            let representation: String = row.try_get("representation_text").map_err(db)?;
            let exact = exact_refs.contains(&reference)
                || exact_refs.contains(&CognitiveRef::MemoryRevision(revision));
            let runtime = resident_refs.contains(&format!("memory:{}", memory_id.0))
                || resident_refs.contains(&format!("memory_revision:{}", revision.0));
            let lexical = lexical_ranks.contains_key(&reference)
                || (!pattern.is_empty()
                    && pattern
                        .split_whitespace()
                        .all(|term| representation.to_lowercase().contains(&term.to_lowercase())));
            let entity = if entity_cues.is_empty() {
                false
            } else {
                self.memory_has_entities(revision.0, &entity_cues).await?
            };
            let required_entity = if required_entities.is_empty() {
                true
            } else {
                self.memory_has_all_entities(revision.0, &required_entities)
                    .await?
            };
            if !required_entity {
                continue;
            }
            let mut family_ranks = HashMap::new();
            if exact {
                family_ranks.insert(EvidenceFamily::Exact, 1);
            }
            if runtime {
                family_ranks.insert(EvidenceFamily::Runtime, 1);
            }
            if lexical {
                family_ranks.insert(
                    EvidenceFamily::Lexical,
                    lexical_ranks.get(&reference).copied().unwrap_or(index + 1),
                );
            }
            if let Some(rank) = dense_ranks.get(&reference) {
                family_ranks.insert(EvidenceFamily::SemanticDense, *rank);
            }
            if entity {
                family_ranks.insert(EvidenceFamily::Entity, index + 1);
            }
            if !temporal_cues.is_empty()
                || query.constraints.occurred.is_some()
                || query.constraints.observed.is_some()
                || query.constraints.valid.is_some()
            {
                family_ranks.insert(EvidenceFamily::Temporal, index + 1);
            }
            if tag_cues.iter().any(|tag| revision_tags.contains(tag)) {
                family_ranks.insert(EvidenceFamily::TagDirect, index + 1);
            }
            if anchor_cues
                .iter()
                .any(|anchor| revision_anchors.contains(anchor))
            {
                family_ranks.insert(EvidenceFamily::AnchorDirect, index + 1);
            }
            let trail = CandidateSemanticTrail {
                memory: reference.clone(),
                nodes: Vec::new(),
                order: TrailOrder::Unavailable,
                provenance: None,
            };
            rank_inputs.push(CandidateRankInput {
                reference: reference.clone(),
                family_ranks,
                topology: CandidateTopologyObservation::default(),
                trail: Some(trail),
                variants: dense_variants
                    .get(&reference)
                    .map(|variants| {
                        let mut variants = variants.iter().cloned().collect::<Vec<_>>();
                        variants.sort();
                        variants
                    })
                    .unwrap_or_default(),
            });
            views.push((reference, memory_id, revision, row));
        }
        let observability = if let Some(wave) = &snapshot.wave {
            let seeds = wave_seeds(
                wave,
                &query,
                &tag_cues,
                &entity_cues,
                &anchor_cues,
                &exact_refs,
            );
            if seeds.is_empty() {
                0.0
            } else {
                let river = propagate(wave, &seeds);
                let local = bounded_restart_field(
                    wave,
                    &river.source_field,
                    wave.config.local_alpha,
                    wave.config.local_iterations,
                );
                let transfer = bounded_restart_field(
                    wave,
                    &river.source_field,
                    wave.config.transfer_alpha,
                    wave.config.transfer_iterations,
                );
                if let Some(output) = &query_embedding {
                    for (variant, field) in [
                        ("local_field_dense", &local),
                        ("transfer_field_dense", &transfer),
                    ] {
                        for generation in &snapshot.dense {
                            if !generation.space.compatible_with(&output.space) {
                                continue;
                            }
                            let Some(vector) = field_vector(wave, field, generation) else {
                                continue;
                            };
                            for (rank, item) in
                                generation.search(&vector, 64)?.into_iter().enumerate()
                            {
                                if let Some(record) = item.record
                                    && matches!(record.reference, CognitiveRef::Memory(_))
                                {
                                    dense_ranks
                                        .entry(record.reference.clone())
                                        .or_insert(rank + 1);
                                    dense_variants
                                        .entry(record.reference)
                                        .or_default()
                                        .insert(variant.into());
                                }
                            }
                        }
                    }
                }
                for candidate in &mut rank_inputs {
                    if let CognitiveRef::Memory(_) = &candidate.reference
                        && let Some((_, _, revision, _)) = views
                            .iter()
                            .find(|(reference, _, _, _)| reference == &candidate.reference)
                    {
                        let trail = self
                            .candidate_trail(revision.0, candidate.reference.clone(), wave)
                            .await?;
                        candidate.topology = trail_topology_observation(
                            &trail,
                            &river.source_field,
                            &local,
                            &transfer,
                            &river,
                        );
                        if candidate.topology.field_contact > 0.0 {
                            candidate.family_ranks.insert(EvidenceFamily::WaveField, 1);
                        }
                        candidate.trail = Some(trail);
                    } else if let Some(node) = wave.node_id(&candidate.reference) {
                        let trail = CandidateSemanticTrail {
                            memory: candidate.reference.clone(),
                            nodes: vec![node],
                            order: TrailOrder::Unordered,
                            provenance: Some("current Wave node projection".into()),
                        };
                        candidate.topology = trail_topology_observation(
                            &trail,
                            &river.source_field,
                            &local,
                            &transfer,
                            &river,
                        );
                        if candidate.topology.field_contact > 0.0 {
                            candidate.family_ranks.insert(EvidenceFamily::WaveField, 1);
                        }
                        candidate.trail = Some(trail);
                    }
                }
                wave_observability(&river).omega
            }
        } else {
            0.0
        };
        for candidate in &mut rank_inputs {
            if let Some(variants) = dense_variants.get(&candidate.reference) {
                let mut variants = variants.iter().cloned().collect::<Vec<_>>();
                variants.sort();
                candidate.variants = variants;
            }
        }
        let ranked = rank_candidates(&rank_inputs, observability);
        let mut results = Vec::new();
        let mut result_references = HashSet::new();
        for target in &query.targets {
            let QueryTarget::Exact { reference } = target else {
                continue;
            };
            if !self.reference_in_subject(query.subject, reference).await? {
                return Err(Error::Invalid(
                    "exact query reference is outside Subject".into(),
                ));
            }
            if matches!(reference, CognitiveRef::Memory(_)) {
                continue;
            }
            let exact_authority = match reference {
                CognitiveRef::Resource(_) => AuthorityClass::ResourceDescriptor,
                CognitiveRef::DerivedRepresentation(_) | CognitiveRef::DerivedRegion(_) => {
                    AuthorityClass::Interpretation
                }
                CognitiveRef::Artifact(_)
                | CognitiveRef::SourceRegion(_)
                | CognitiveRef::Occurrence(_)
                | CognitiveRef::ExternalObject(_) => AuthorityClass::Evidence,
                CognitiveRef::Session(_) => AuthorityClass::Evidence,
                CognitiveRef::MemoryRevision(_)
                | CognitiveRef::Entity(_)
                | CognitiveRef::Tag(_)
                | CognitiveRef::Anchor(_) => AuthorityClass::SubjectCognition,
                CognitiveRef::Memory(_) => AuthorityClass::SubjectCognition,
            };
            if query
                .constraints
                .authority
                .is_some_and(|authority| authority != exact_authority)
            {
                continue;
            }
            let Some(hit) = exact_hit(
                reference,
                query.result_need.need_evidence,
                query.result_need.need_materialization_handles,
            ) else {
                continue;
            };
            result_references.insert(reference.clone());
            results.push(hit);
            if results.len() >= query.result_need.limit {
                break;
            }
        }
        for candidate in ranked
            .into_iter()
            .take(query.result_need.limit.saturating_sub(results.len()))
        {
            if result_references.contains(&candidate.reference) {
                continue;
            }
            let Some((_, memory_id, revision, row)) = views
                .iter()
                .find(|(reference, _, _, _)| *reference == candidate.reference)
            else {
                continue;
            };
            let memory_view = self
                .memory(subject_id_from_row(row)?, *memory_id, Some(*revision))
                .await?;
            let evidence = if query.result_need.need_evidence {
                memory_view
                    .evidence
                    .into_iter()
                    .map(|item| EvidenceHandle {
                        reference: item.evidence.cognitive_ref(),
                        support_role: format!("{:?}", item.support_role).to_lowercase(),
                    })
                    .collect()
            } else {
                Vec::new()
            };
            results.push(CognitiveHit {
                reference: candidate.reference.clone(),
                revision: Some(*revision),
                semantic_role: row.try_get("semantic_role").map_err(db)?,
                memory_class: row.try_get("memory_class").map_err(db)?,
                representation: Some(row.try_get("representation_text").map_err(db)?),
                authority: AuthorityClass::SubjectCognition,
                freshness: FreshnessDescriptor {
                    observed_at: row.try_get("observed_at").map_err(db)?,
                    valid_from: row.try_get("valid_from").map_err(db)?,
                    valid_to: row.try_get("valid_to").map_err(db)?,
                },
                entity_refs: memory_view.entities,
                evidence,
                match_evidence: MatchEvidence {
                    families: candidate.families,
                    base_rank_score: candidate.base_rank_score,
                    field_contact: candidate.field_contact,
                    structural_score: candidate.structural_score,
                    topology_innovation: candidate.topology_innovation,
                    wave_observability: candidate.wave_observability,
                    direct_seed_evidence: candidate.direct_seed_evidence,
                    final_score: candidate.final_score,
                    variants: candidate.variants,
                    explanation: Some("candidate evidence retained by family".into()),
                },
                materialization: if query.result_need.need_materialization_handles {
                    vec![MaterializationHandle {
                        reference: CognitiveRef::MemoryRevision(*revision),
                        level: "memory_revision".into(),
                    }]
                } else {
                    Vec::new()
                },
                supersession_state: row.try_get("supersession_state").map_err(db)?,
            });
            result_references.insert(candidate.reference);
        }
        let target_allows_derived = query.targets.is_empty()
            || query.targets.iter().any(|target| {
                matches!(
                    target,
                    QueryTarget::AnyRelevantCognition | QueryTarget::Evidence
                )
            });
        if target_allows_derived && results.len() < query.result_need.limit {
            for occurrence_id in resident_occurrences {
                let reference = CognitiveRef::Occurrence(occurrence_id);
                if result_references.contains(&reference) {
                    continue;
                }
                let Some(row) = sqlx::query("SELECT o.source_class,o.observed_at,a.content_hash FROM observation_occurrences o LEFT JOIN artifacts a ON a.artifact_id=o.artifact_id WHERE o.subject_id=$1 AND o.occurrence_id=$2")
                    .bind(query.subject.0)
                    .bind(occurrence_id.0)
                    .fetch_optional(self.store.pool())
                    .await
                    .map_err(db)?
                else {
                    continue;
                };
                let source_class: String = row.try_get("source_class").map_err(db)?;
                if (!query.constraints.source_classes_include.is_empty()
                    && !query
                        .constraints
                        .source_classes_include
                        .iter()
                        .any(|class| class.as_str() == source_class))
                    || query
                        .constraints
                        .source_classes_exclude
                        .iter()
                        .any(|class| class.as_str() == source_class)
                    || query
                        .constraints
                        .authority
                        .is_some_and(|authority| authority != AuthorityClass::Evidence)
                {
                    continue;
                }
                let representation = if let Some(hash) = row
                    .try_get::<Option<String>, _>("content_hash")
                    .map_err(db)?
                {
                    self.objects
                        .get(&hash)
                        .await
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                } else {
                    None
                };
                results.push(CognitiveHit {
                    reference: reference.clone(),
                    revision: None,
                    semantic_role: Some("observation".into()),
                    memory_class: None,
                    representation,
                    authority: AuthorityClass::Evidence,
                    freshness: FreshnessDescriptor {
                        observed_at: Some(row.try_get("observed_at").map_err(db)?),
                        valid_from: None,
                        valid_to: None,
                    },
                    entity_refs: Vec::new(),
                    evidence: if query.result_need.need_evidence {
                        vec![EvidenceHandle {
                            reference: reference.clone(),
                            support_role: "runtime_observation".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    match_evidence: MatchEvidence {
                        families: vec![EvidenceFamily::Runtime],
                        base_rank_score: 1.0,
                        field_contact: 0.0,
                        structural_score: 0.0,
                        topology_innovation: 0.0,
                        wave_observability: 0.0,
                        direct_seed_evidence: 0.0,
                        final_score: 1.0,
                        variants: Vec::new(),
                        explanation: Some("resident observation".into()),
                    },
                    materialization: if query.result_need.need_materialization_handles {
                        vec![MaterializationHandle {
                            reference: reference.clone(),
                            level: "evidence_occurrence".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    supersession_state: None,
                });
                result_references.insert(reference);
                if results.len() >= query.result_need.limit {
                    break;
                }
            }
        }
        if target_allows_derived && results.len() < query.result_need.limit {
            let mut derived_candidates = lexical_ranks
                .iter()
                .filter_map(|(reference, rank)| match reference {
                    CognitiveRef::DerivedRepresentation(id) => Some((*rank, *id)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            derived_candidates.sort_by_key(|(rank, id)| (*rank, id.0));
            for (rank, id) in derived_candidates
                .into_iter()
                .take(query.result_need.limit.saturating_sub(results.len()))
            {
                let Some(row) = sqlx::query("SELECT representation_kind,payload_text,source_region_id FROM derived_representations WHERE subject_id=$1 AND derived_representation_id=$2")
                    .bind(query.subject.0)
                    .bind(id.0)
                    .fetch_optional(self.store.pool())
                    .await
                    .map_err(db)? else {
                    continue;
                };
                let source_region = SourceRegionId(row.try_get("source_region_id").map_err(db)?);
                let reference = CognitiveRef::DerivedRepresentation(id);
                if result_references.contains(&reference) {
                    continue;
                }
                if query
                    .constraints
                    .authority
                    .is_some_and(|authority| authority != AuthorityClass::Interpretation)
                {
                    continue;
                }
                results.push(CognitiveHit {
                    reference: reference.clone(),
                    revision: None,
                    semantic_role: Some(row.try_get("representation_kind").map_err(db)?),
                    memory_class: None,
                    representation: row.try_get("payload_text").map_err(db)?,
                    authority: AuthorityClass::Interpretation,
                    freshness: FreshnessDescriptor {
                        observed_at: None,
                        valid_from: None,
                        valid_to: None,
                    },
                    entity_refs: Vec::new(),
                    evidence: if query.result_need.need_evidence {
                        vec![EvidenceHandle {
                            reference: CognitiveRef::SourceRegion(source_region),
                            support_role: "interpretation".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    match_evidence: MatchEvidence {
                        families: vec![EvidenceFamily::Lexical],
                        base_rank_score: 1.0 / (60.0 + rank as f64),
                        field_contact: 0.0,
                        structural_score: 0.0,
                        topology_innovation: 0.0,
                        wave_observability: 0.0,
                        direct_seed_evidence: 0.0,
                        final_score: 1.0 / (60.0 + rank as f64),
                        variants: Vec::new(),
                        explanation: Some("persisted derived textual surrogate".into()),
                    },
                    materialization: if query.result_need.need_materialization_handles {
                        vec![MaterializationHandle {
                            reference: CognitiveRef::SourceRegion(source_region),
                            level: "evidence_region".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    supersession_state: None,
                });
                result_references.insert(reference);
            }
        }
        if target_allows_derived && results.len() < query.result_need.limit {
            let mut occurrence_candidates = lexical_ranks
                .iter()
                .filter_map(|(reference, rank)| match reference {
                    CognitiveRef::Occurrence(id) => Some((*rank, *id)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            occurrence_candidates.sort_by_key(|(rank, id)| (*rank, id.0));
            for (rank, occurrence_id) in occurrence_candidates
                .into_iter()
                .take(query.result_need.limit.saturating_sub(results.len()))
            {
                let reference = CognitiveRef::Occurrence(occurrence_id);
                if result_references.contains(&reference) {
                    continue;
                }
                let Some(row) = sqlx::query("SELECT o.source_class,o.observed_at,a.content_hash FROM observation_occurrences o JOIN artifacts a ON a.artifact_id=o.artifact_id WHERE o.subject_id=$1 AND o.occurrence_id=$2")
                    .bind(query.subject.0)
                    .bind(occurrence_id.0)
                    .fetch_optional(self.store.pool())
                    .await
                    .map_err(db)? else {
                    continue;
                };
                let source_class: String = row.try_get("source_class").map_err(db)?;
                if query
                    .constraints
                    .authority
                    .is_some_and(|authority| authority != AuthorityClass::Evidence)
                {
                    continue;
                }
                if (!query.constraints.source_classes_include.is_empty()
                    && !query
                        .constraints
                        .source_classes_include
                        .iter()
                        .any(|class| class.as_str() == source_class))
                    || query
                        .constraints
                        .source_classes_exclude
                        .iter()
                        .any(|class| class.as_str() == source_class)
                {
                    continue;
                }
                let hash: String = row.try_get("content_hash").map_err(db)?;
                let Ok(bytes) = self.objects.get(&hash).await else {
                    continue;
                };
                let Ok(text) = String::from_utf8(bytes) else {
                    continue;
                };
                results.push(CognitiveHit {
                    reference: reference.clone(),
                    revision: None,
                    semantic_role: Some("observation".into()),
                    memory_class: None,
                    representation: Some(text),
                    authority: AuthorityClass::Evidence,
                    freshness: FreshnessDescriptor {
                        observed_at: Some(row.try_get("observed_at").map_err(db)?),
                        valid_from: None,
                        valid_to: None,
                    },
                    entity_refs: Vec::new(),
                    evidence: if query.result_need.need_evidence {
                        vec![EvidenceHandle {
                            reference: CognitiveRef::Occurrence(occurrence_id),
                            support_role: "direct".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    match_evidence: MatchEvidence {
                        families: vec![EvidenceFamily::Lexical],
                        base_rank_score: 1.0 / (60.0 + rank as f64),
                        field_contact: 0.0,
                        structural_score: 0.0,
                        topology_innovation: 0.0,
                        wave_observability: 0.0,
                        direct_seed_evidence: 0.0,
                        final_score: 1.0 / (60.0 + rank as f64),
                        variants: Vec::new(),
                        explanation: Some("raw observed textual evidence".into()),
                    },
                    materialization: if query.result_need.need_materialization_handles {
                        vec![MaterializationHandle {
                            reference: CognitiveRef::Occurrence(occurrence_id),
                            level: "evidence_occurrence".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    supersession_state: None,
                });
                result_references.insert(reference);
            }
        }
        if query
            .targets
            .iter()
            .any(|target| matches!(target, QueryTarget::Resource))
            && results.len() < query.result_need.limit
        {
            for resource in self.list_resources(query.subject).await? {
                let descriptor = resource.descriptor;
                if query
                    .constraints
                    .authority
                    .is_some_and(|authority| authority != AuthorityClass::ResourceDescriptor)
                {
                    continue;
                }
                let reference = CognitiveRef::Resource(descriptor.resource_ref.clone());
                if result_references.contains(&reference) {
                    continue;
                }
                results.push(CognitiveHit {
                    reference: reference.clone(),
                    revision: None,
                    semantic_role: Some("resource_descriptor".into()),
                    memory_class: None,
                    representation: descriptor.display_label.clone(),
                    authority: AuthorityClass::ResourceDescriptor,
                    freshness: FreshnessDescriptor {
                        observed_at: Some(descriptor.updated_at),
                        valid_from: None,
                        valid_to: None,
                    },
                    entity_refs: Vec::new(),
                    evidence: if query.result_need.need_evidence {
                        vec![EvidenceHandle {
                            reference: reference.clone(),
                            support_role: "resource_awareness".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    match_evidence: MatchEvidence {
                        families: vec![EvidenceFamily::Resource],
                        base_rank_score: 1.0,
                        field_contact: 0.0,
                        structural_score: 0.0,
                        topology_innovation: 0.0,
                        wave_observability: 0.0,
                        direct_seed_evidence: 0.0,
                        final_score: 1.0,
                        variants: Vec::new(),
                        explanation: Some("resource awareness descriptor".into()),
                    },
                    materialization: Vec::new(),
                    supersession_state: None,
                });
                result_references.insert(reference);
                if results.len() >= query.result_need.limit {
                    break;
                }
            }
        }
        let (resource_actions, resource_degradation) =
            self.resource_actions_for_query(&query).await?;
        degradation.extend(resource_degradation);
        let status = if degradation.is_empty() {
            QueryStatus::Complete
        } else {
            QueryStatus::Degraded
        };
        Ok(CognitiveQueryResult {
            query_id: Uuid::now_v7(),
            generation: QueryGenerationTrace {
                lexical: snapshot
                    .lexical
                    .as_ref()
                    .map(|generation| generation.generation_id),
                dense: snapshot
                    .dense
                    .iter()
                    .map(|generation| generation.generation_id)
                    .collect(),
                wave: snapshot
                    .wave
                    .as_ref()
                    .map(|generation| generation.generation_id),
                epa_basis: epa_generation,
                postings: snapshot.postings_generation,
            },
            status,
            results,
            resource_actions,
            degradation,
            diagnostics: (query.diagnostics != DiagnosticsRequest::None).then_some(
                QueryDiagnostics {
                    candidate_counts: BTreeMap::from([
                        ("structured".into(), rank_inputs.len()),
                        ("lexical".into(), lexical_ranks.len()),
                        ("dense".into(), dense_ranks.len()),
                    ]),
                    lane_status: BTreeMap::from([
                        (
                            "epa".into(),
                            if epa_trace.is_some() {
                                "ready"
                            } else {
                                "unavailable"
                            }
                            .into(),
                        ),
                        (
                            "residual".into(),
                            if residual_trace.is_some() {
                                "ready"
                            } else {
                                "unavailable"
                            }
                            .into(),
                        ),
                    ]),
                    wave_observability: Some(observability),
                    trace: Some(serde_json::json!({
                        "epa": epa_trace,
                        "residual": residual_trace,
                    })),
                },
            ),
        })
    }
}
