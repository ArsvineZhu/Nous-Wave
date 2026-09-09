use super::support::*;
use super::*;

use super::query_diagnostics::{QueryDiagnosticsInput, build_query_diagnostics};
use super::query_helpers::*;

impl MemoryService {
    // Query orchestration retains lane ordering and evidence-family semantics together.
    pub async fn query(&self, query: CognitiveQuery) -> Result<CognitiveQueryResult> {
        let plan = nous_cognitive_runtime::QueryPlan::for_query(&query);
        self.query_with_plan(query, &plan).await
    }

    #[expect(
        clippy::excessive_nesting,
        clippy::too_many_lines,
        reason = "query orchestration owns the ordered candidate and evidence phases"
    )]
    pub async fn query_with_plan(
        &self,
        query: CognitiveQuery,
        plan: &nous_cognitive_runtime::QueryPlan,
    ) -> Result<CognitiveQueryResult> {
        query.validate()?;
        self.require_subject(query.subject).await?;
        if let Some(session) = query.session {
            self.cognition
                .require_session(query.subject, session)
                .await?;
        }
        let snapshot = self.serving.publisher.snapshot_for(query.subject);
        let mut degradation = Vec::new();
        let has_text_cue = query
            .cues
            .iter()
            .any(|cue| matches!(cue, Cue::Text(_) | Cue::Example(_)));
        if matches!(
            query.capabilities.text_embedding,
            RequirementStrength::Required
        ) && has_text_cue
            && (snapshot.dense.is_empty() || self.serving.embedding.is_none())
        {
            return Err(Error::Unavailable(
                "text embedding capability/generation is unavailable".into(),
            ));
        }
        if matches!(
            query.capabilities.text_embedding,
            RequirementStrength::Preferred
        ) && has_text_cue
            && (snapshot.dense.is_empty() || self.serving.embedding.is_none())
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
            && (snapshot.dense.is_empty() || self.serving.embedding.is_none())
        {
            return Err(Error::Unavailable(
                "residual sensing requires a compatible dense generation".into(),
            ));
        }
        if matches!(
            query.capabilities.residual_sensing,
            RequirementStrength::Preferred
        ) && has_text_cue
            && (snapshot.dense.is_empty() || self.serving.embedding.is_none())
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
        let mut candidates = CandidateAccumulator::default();
        let mut executed_candidate_bound = 0usize;
        if !pattern.is_empty()
            && let Some(lexical) = &snapshot.lexical
        {
            executed_candidate_bound = executed_candidate_bound.max(plan.candidate_limit);
            match lexical.search(&pattern, plan.candidate_limit) {
                Ok(matches) => {
                    for (rank, item) in matches.into_iter().enumerate() {
                        if let Some(reference) = item.reference {
                            candidates.add_lexical(reference, rank + 1);
                        }
                    }
                }
                Err(error) => degradation.push(Degradation {
                    code: "lexical_generation_unavailable".into(),
                    detail: Some(error.to_string()),
                }),
            }
        }
        let mut query_embedding = None;
        if !pattern.is_empty()
            && query.capabilities.text_embedding != RequirementStrength::Forbidden
            && let Some(provider) = &self.serving.embedding
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
                            .search(&output.vector, plan.candidate_limit)
                            .map_err(|error| Error::Infrastructure(error.to_string()))?
                            .into_iter()
                            .enumerate()
                        {
                            if let Some(record) = item.record {
                                candidates.add_dense(record.reference, rank + 1, "direct_dense");
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
                    } else {
                        executed_candidate_bound =
                            executed_candidate_bound.max(plan.candidate_limit);
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
        let tag_cues = query
            .cues
            .iter()
            .filter_map(|cue| match cue {
                Cue::Tag(tag) => Some(tag.tag),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut residual_tag_cues = Vec::new();
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
        let mut residual_seed_refs = Vec::new();
        let mut epa_trace = None;
        let mut epa_generation = None;
        if let Some(output) = &query_embedding {
            if plan.sense_cues
                && query.capabilities.residual_sensing != RequirementStrength::Forbidden
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
                let residual = self.cue_sensing.sense(
                    &output.vector,
                    generation,
                    ResidualConfig {
                        top_k_per_level: plan.candidate_limit.clamp(1, 64),
                        ..ResidualConfig::default()
                    },
                )?;
                if let Some(residual) = residual {
                    for sensed in residual.levels.iter().flat_map(|level| level.sensed.iter()) {
                        if let Some(CognitiveRef::Tag(tag)) = tag_by_doc.get(&sensed.tag_key) {
                            if !residual_tag_cues.contains(tag) {
                                residual_tag_cues.push(*tag);
                            }
                            residual_seed_refs
                                .push((CognitiveRef::Tag(*tag), sensed.seed_weight.max(0.0)));
                        }
                    }
                    residual_trace = Some(residual);
                }
            }
            if plan.sense_cues
                && let Some(generation) = snapshot
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
                        for (rank, item) in generation
                            .search(&denoised, plan.candidate_limit)?
                            .into_iter()
                            .enumerate()
                        {
                            executed_candidate_bound =
                                executed_candidate_bound.max(plan.candidate_limit);
                            if let Some(record) = item.record {
                                candidates.add_dense(record.reference, rank + 1, "denoised_dense");
                            }
                        }
                    }
                }
            }
        }
        let mut candidate_memory_ids = HashSet::<Uuid>::new();
        for reference in &exact_refs {
            candidates.mark_exact(reference.clone());
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
            candidates.mark_runtime(reference.clone());
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
            residual_tag_cues
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
        let mut posting_budget = plan.candidate_limit;
        for key in posting_keys {
            let lane = if key.starts_with("entity:") {
                "entity_posting"
            } else if key.starts_with("tag:") {
                "tag_posting"
            } else {
                "anchor_posting"
            };
            if posting_budget == 0 {
                break;
            }
            executed_candidate_bound = executed_candidate_bound.max(plan.candidate_limit);
            let references = snapshot.postings.references(&key);
            let consumed = references.len().min(posting_budget);
            for reference in references.into_iter().take(posting_budget) {
                candidates.add_posting(reference.clone(), lane);
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
            posting_budget -= consumed;
        }
        let mut topology_state = None;
        if plan.expand_topology
            && let Some(wave) = &snapshot.wave
        {
            let seeds = wave_seeds(
                wave,
                &query,
                &tag_cues,
                &entity_cues,
                &anchor_cues,
                &exact_refs,
                &residual_seed_refs,
                query_embedding
                    .as_ref()
                    .map(|embedding| embedding.space.space_hash.as_str()),
            );
            if !seeds.is_empty() {
                let river =
                    self.expansion
                        .expand(wave, &seeds, plan.topology_rounds, plan.topology_nodes);
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
                for (_, memory) in topology_candidates
                    .into_iter()
                    .take(plan.topology_nodes.min(plan.candidate_limit))
                {
                    candidate_memory_ids.insert(memory);
                }
                let local = bounded_restart_field(
                    wave,
                    &river.source_field,
                    wave.config.local_alpha,
                    plan.topology_rounds,
                );
                let transfer = bounded_restart_field(
                    wave,
                    &river.source_field,
                    wave.config.transfer_alpha,
                    plan.topology_rounds,
                );
                topology_state = Some((river, local, transfer, wave.clone()));
            }
        }
        if let Some((_, local, transfer, wave)) = &topology_state
            && let Some(output) = &query_embedding
        {
            for (variant, field) in [
                ("local_field_dense", local),
                ("transfer_field_dense", transfer),
            ] {
                for generation in &snapshot.dense {
                    if !generation.space.compatible_with(&output.space) {
                        continue;
                    }
                    let Some(vector) = field_vector(wave, field, generation) else {
                        continue;
                    };
                    for (rank, item) in generation
                        .search(&vector, plan.candidate_limit)?
                        .into_iter()
                        .enumerate()
                    {
                        executed_candidate_bound =
                            executed_candidate_bound.max(plan.candidate_limit);
                        if let Some(record) = item.record {
                            let reference = record.reference.clone();
                            candidates.add_dense(reference.clone(), rank + 1, variant);
                            if let CognitiveRef::Memory(memory) = record.reference {
                                candidate_memory_ids.insert(memory.0);
                            }
                        }
                    }
                }
            }
        }
        for reference in candidates.candidate_refs() {
            if let CognitiveRef::Memory(memory) = reference {
                candidate_memory_ids.insert(memory.0);
            }
        }
        let lexical_ranks = candidates.lexical_ranks();
        let dense_ranks = candidates.dense_ranks();
        let dense_variants = candidates.dense_variants();
        let resident_refs = if let Some(session) = query.session {
            self.cognition.resident_reference_strings(session).await?
        } else {
            HashSet::new()
        };
        for value in &resident_refs {
            if let Some((kind, value)) = value.split_once(':')
                && let Ok(reference) = parse_reference(kind, value)
            {
                candidates.mark_runtime(reference);
            }
        }
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
            sqlx::query("SELECT o.memory_id,o.subject_id,o.memory_class,o.current_revision_id,o.status,r.revision_no,r.semantic_role,r.title,r.representation_text,r.epistemic_class,r.confidence,r.occurred_at,r.observed_at,r.valid_from,r.valid_to,r.supersession_state FROM memory_objects o JOIN memory_revisions r ON r.memory_revision_id=o.current_revision_id WHERE o.subject_id=$1 AND ($2 OR o.status='active') AND o.memory_id=ANY($3) ORDER BY r.observed_at DESC")
                .bind(query.subject.0)
                .bind(query.constraints.include_suppressed)
                .bind(ids)
                .fetch_all(self.store.pool())
                .await
                .map_err(db)?
        } else if pattern.is_empty() {
            sqlx::query("SELECT o.memory_id,o.subject_id,o.memory_class,o.current_revision_id,o.status,r.revision_no,r.semantic_role,r.title,r.representation_text,r.epistemic_class,r.confidence,r.occurred_at,r.observed_at,r.valid_from,r.valid_to,r.supersession_state FROM memory_objects o JOIN memory_revisions r ON r.memory_revision_id=o.current_revision_id WHERE o.subject_id=$1 AND ($2 OR o.status='active') ORDER BY r.observed_at DESC LIMIT $3")
                .bind(query.subject.0).bind(query.constraints.include_suppressed).bind(plan.candidate_limit as i64).fetch_all(self.store.pool()).await.map_err(db)?
        } else if snapshot.lexical.is_some() && !query.constraints.include_suppressed {
            Vec::new()
        } else {
            let terms = pattern
                .split_whitespace()
                .take(16)
                .map(|term| format!("%{}%", term.replace('%', "")))
                .collect::<Vec<_>>();
            sqlx::query("SELECT o.memory_id,o.subject_id,o.memory_class,o.current_revision_id,o.status,r.revision_no,r.semantic_role,r.title,r.representation_text,r.epistemic_class,r.confidence,r.occurred_at,r.observed_at,r.valid_from,r.valid_to,r.supersession_state FROM memory_objects o JOIN memory_revisions r ON r.memory_revision_id=o.current_revision_id WHERE o.subject_id=$1 AND ($2 OR o.status='active') AND r.representation_text ILIKE ANY($3) ORDER BY r.observed_at DESC LIMIT $4")
                .bind(query.subject.0).bind(query.constraints.include_suppressed).bind(&terms).bind(plan.candidate_limit as i64).fetch_all(self.store.pool()).await.map_err(db)?
        };
        let mut rank_inputs = Vec::new();
        let mut views = Vec::new();
        for row in rows {
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
            let exact = candidates.is_exact(&reference)
                || candidates.is_exact(&CognitiveRef::MemoryRevision(revision))
                || exact_refs.contains(&reference)
                || exact_refs.contains(&CognitiveRef::MemoryRevision(revision));
            let runtime = resident_refs.contains(&format!("memory:{}", memory_id.0))
                || resident_refs.contains(&format!("memory_revision:{}", revision.0));
            let lexical_hit = lexical_ranks.contains_key(&reference);
            let substring_fallback = !lexical_hit
                && (snapshot.lexical.is_none() || query.constraints.include_suppressed)
                && !pattern.is_empty()
                && pattern
                    .split_whitespace()
                    .all(|term| representation.to_lowercase().contains(&term.to_lowercase()));
            let lane_variants = candidates
                .posting_hits(&reference)
                .map(|lanes| lanes.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            let entity = lane_variants.iter().any(|lane| lane == "entity_posting");
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
            if lexical_hit {
                family_ranks.insert(
                    EvidenceFamily::Lexical,
                    lexical_ranks.get(&reference).copied().unwrap_or(1),
                );
            }
            let mut variants = candidates
                .dense_variant_set(&reference)
                .map(|variants| {
                    let mut variants = variants.iter().cloned().collect::<Vec<_>>();
                    variants.sort();
                    variants
                })
                .unwrap_or_default();
            if substring_fallback {
                variants.push("substring_fallback".into());
            }
            variants.extend(lane_variants);
            if let Some(rank) = dense_ranks.get(&reference) {
                family_ranks.insert(EvidenceFamily::SemanticDense, *rank);
            }
            if entity {
                family_ranks.insert(EvidenceFamily::Entity, 1);
            }
            if !temporal_cues.is_empty()
                || query.constraints.occurred.is_some()
                || query.constraints.observed.is_some()
                || query.constraints.valid.is_some()
            {
                family_ranks.insert(EvidenceFamily::Temporal, 1);
            }
            if candidates
                .posting_hits(&reference)
                .is_some_and(|hits| hits.contains("tag_posting"))
            {
                family_ranks.insert(EvidenceFamily::TagDirect, 1);
            }
            if candidates
                .posting_hits(&reference)
                .is_some_and(|hits| hits.contains("anchor_posting"))
            {
                family_ranks.insert(EvidenceFamily::AnchorDirect, 1);
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
                variants,
            });
            views.push((reference, memory_id, revision, row));
        }
        let observability = if let Some((river, local, transfer, wave)) = &topology_state {
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
                        local,
                        transfer,
                        river,
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
                        local,
                        transfer,
                        river,
                    );
                    if candidate.topology.field_contact > 0.0 {
                        candidate.family_ranks.insert(EvidenceFamily::WaveField, 1);
                    }
                    candidate.trail = Some(trail);
                }
            }
            wave_observability(river).omega
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
        let (mut results, mut result_references) = self
            .project_memory_results(&query, ranked, &views, plan.materialize_evidence)
            .await?;
        self.append_evidence_results(
            &query,
            &candidates,
            resident_occurrences,
            &mut results,
            &mut result_references,
            plan.materialize_evidence,
        )
        .await?;
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
            // Resource routing is owned by Cognitive Runtime. Keeping it out
            // of the Memory contributor prevents one host query from invoking
            // a resolver twice.
            resource_actions: Vec::new(),
            degradation,
            diagnostics: build_query_diagnostics(QueryDiagnosticsInput {
                query: &query,
                plan,
                structured_count: rank_inputs.len(),
                lexical_count: lexical_ranks.len(),
                dense_count: dense_ranks.len(),
                all_lane_count: candidates.candidate_refs().count(),
                field_dense_count: dense_variants
                    .values()
                    .filter(|variants| {
                        variants.contains("local_field_dense")
                            || variants.contains("transfer_field_dense")
                    })
                    .count(),
                executed_candidate_bound,
                epa_trace: epa_trace.as_ref(),
                residual_trace: residual_trace.as_ref(),
                topology_executed: topology_state.is_some(),
                observability,
            }),
        })
    }
}

#[async_trait::async_trait]
impl nous_cognitive_runtime::CognitiveContributor for MemoryService {
    async fn contribute(
        &self,
        query: &CognitiveQuery,
        plan: &nous_cognitive_runtime::QueryPlan,
    ) -> Result<CognitiveQueryResult> {
        self.query_with_plan(query.clone(), plan).await
    }
}
