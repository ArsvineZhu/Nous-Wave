use crate::{
    LocalRuntime,
    cycles::WorkingState,
    memory::MemoryView,
    subjects::{check_version, db},
};
use chrono::{DateTime, Utc};
use nous_core::{Error, Result, SubjectId};
use nous_material::ProcessorProvenance;
use nous_memory_domain::{
    learning::{UseKind, accessibility},
    recall::{NodeKind, NodeRef, RecallEffort, RecallEffortTrace, RecallIntent, RecallObjective},
};
use nous_memory_retrieval::association::ActivationSupport;
use nous_memory_retrieval::residual;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallHit {
    pub memory: MemoryView,
    pub rank_score: f64,
    pub exact: bool,
    pub association_support: Vec<ActivationSupport>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallResponse {
    pub api_version: u32,
    pub recall_id: Uuid,
    pub cycle_id: Option<Uuid>,
    pub results: Vec<RecallHit>,
    pub sufficiency: String,
    pub degraded: Vec<String>,
    pub effort: RecallEffortTrace,
    pub models: Vec<ProcessorProvenance>,
    pub can_continue: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecallBudget {
    pub max_candidates: usize,
    pub max_association_states: usize,
    pub max_association_edges: usize,
}
impl RecallBudget {
    pub fn validate(&self) -> Result<()> {
        if self.max_candidates == 0
            || self.max_candidates > 400
            || self.max_association_states == 0
            || self.max_association_states > 5000
            || self.max_association_edges == 0
            || self.max_association_edges > 40000
        {
            return Err(Error::Invalid(
                "recall budgets exceed bounded product limits".into(),
            ));
        }
        Ok(())
    }
    fn for_effort(effort: RecallEffort) -> Self {
        let (max_candidates, max_association_states, max_association_edges) = match effort {
            RecallEffort::Light => (24, 64, 256),
            RecallEffort::Normal => (64, 256, 1500),
            RecallEffort::Deep => (160, 1200, 8000),
            RecallEffort::Maximum => (400, 5000, 40000),
        };
        Self {
            max_candidates,
            max_association_states,
            max_association_edges,
        }
    }
}

impl LocalRuntime {
    pub fn recall_input(&self, mut value: serde_json::Value) -> Result<RecallIntent> {
        let object = value
            .as_object_mut()
            .ok_or_else(|| Error::Invalid("recall input must be an object".into()))?;
        object
            .entry("effort")
            .or_insert_with(|| serde_json::json!(self.default_effort));
        serde_json::from_value(value).map_err(|e| Error::Invalid(e.to_string()))
    }
    pub async fn recall(&self, subject: SubjectId, intent: RecallIntent) -> Result<RecallResponse> {
        check_version(intent.api_version)?;
        self.require_memory(subject).await?;
        if intent.result_need.limit == 0
            || intent.result_need.limit > 400
            || intent.target.len() > 64
            || intent.cues.references.len() > 64
            || intent.cues.entities.len() > 64
            || intent
                .cues
                .text
                .as_ref()
                .is_some_and(|text| text.len() > 65536)
        {
            return Err(Error::Invalid("recall intent exceeds bounds".into()));
        }
        for time in [&intent.temporal.occurred, &intent.temporal.observed]
            .into_iter()
            .flatten()
        {
            time.validate()?;
        }
        if let Some(cycle) = intent.cycle_id {
            self.check_cycle(subject, cycle).await?;
        }
        let started = Instant::now();
        let budget = self
            .recall_budgets
            .get(&intent.effort)
            .copied()
            .unwrap_or_else(|| RecallBudget::for_effort(intent.effort));
        budget.validate()?;
        let mut cycles = self.cycles.lock().await;
        let mut standalone = WorkingState::default();
        let state = match intent.cycle_id {
            Some(cycle) => cycles
                .get_mut(&cycle)
                .ok_or_else(|| Error::Conflict("cycle working state no longer available".into()))?,
            None => &mut standalone,
        };
        let mut trace = RecallEffortTrace {
            recall_rounds: 1,
            ..Default::default()
        };
        let mut degraded = vec![];
        let mut models = vec![];
        let mut exact = BTreeSet::new();
        let mut scores = if intent.target.is_empty() {
            state.candidates.clone()
        } else {
            BTreeMap::new()
        };
        trace.reused_candidates = scores.len();
        let signature = serde_json::to_string(&(
            &intent.target,
            &intent.objective,
            &intent.temporal,
            &intent.cues,
            &intent.constraints,
            budget.max_candidates,
        ))
        .map_err(|e| Error::Invalid(e.to_string()))?;
        let first_query = state.queried.insert(signature);
        if first_query {
            for target in &intent.target {
                let ids = self
                    .reference_candidates(subject, *target, budget.max_candidates)
                    .await?;
                trace.index_queries += 1;
                trace.candidate_channels.push("exact_reference".into());
                exact.extend(&ids);
                fuse(&mut scores, &ids);
            }
            if intent.target.is_empty() || scores.len() < intent.result_need.limit {
                let entities: Vec<Uuid> = intent
                    .cues
                    .entities
                    .iter()
                    .chain(state.active_entities.iter())
                    .copied()
                    .collect();
                if !entities.is_empty() {
                    let ids:Vec<Uuid>=sqlx::query_scalar("SELECT DISTINCT r.object_id FROM memory_entity_mentions e JOIN memory_revisions r USING(revision_id) WHERE e.subject_id=$1 AND e.entity_id=ANY($2) ORDER BY r.object_id LIMIT $3")
                        .bind(subject.0).bind(&entities).bind(budget.max_candidates as i64).fetch_all(self.store.pool()).await.map_err(db)?;
                    trace.index_queries += 1;
                    trace.candidate_channels.push("entity".into());
                    fuse(&mut scores, &ids);
                }
                if let Some(text) = intent
                    .cues
                    .text
                    .as_ref()
                    .filter(|text| !text.trim().is_empty())
                {
                    let ids:Vec<Uuid>=sqlx::query_scalar("SELECT object_id FROM (SELECT r.object_id,max(ts_rank_cd(r.search_text,websearch_to_tsquery('simple',$2))) AS rank FROM memory_revisions r WHERE r.subject_id=$1 AND (r.search_text@@websearch_to_tsquery('simple',$2) OR r.representation_text ILIKE '%' || $2 || '%') GROUP BY r.object_id) ranked ORDER BY rank DESC,object_id LIMIT $3")
                        .bind(subject.0).bind(text).bind(budget.max_candidates as i64).fetch_all(self.store.pool()).await.map_err(db)?;
                    trace.index_queries += 1;
                    trace.candidate_channels.push("lexical".into());
                    fuse(&mut scores, &ids);
                    let ids:Vec<Uuid>=sqlx::query_scalar("SELECT DISTINCT r.object_id FROM document_sections d JOIN artifact_sources a ON a.artifact_id=d.source_artifact JOIN memory_revision_sources s ON s.source_id=a.source_id JOIN memory_revisions r ON r.revision_id=s.revision_id WHERE d.subject_id=$1 AND (d.search_text@@websearch_to_tsquery('simple',$2) OR d.section_text ILIKE '%' || $2 || '%') ORDER BY r.object_id LIMIT $3")
                        .bind(subject.0).bind(text).bind(budget.max_candidates as i64).fetch_all(self.store.pool()).await.map_err(db)?;
                    trace.index_queries += 1;
                    trace.candidate_channels.push("document_section".into());
                    fuse(&mut scores, &ids);
                    if self.models.embedding_available()
                        && self.projection.is_some()
                        && (intent.effort != RecallEffort::Light
                            || scores.len() < intent.result_need.limit)
                    {
                        match self.models.embed_query(text).await {
                            Ok((result, provenance)) => {
                                trace.model_assisted_calls += 1;
                                let table = nous_memory_retrieval::dense::model_table(
                                    &provenance.identity,
                                    &provenance.revision,
                                    &provenance.preprocessing,
                                    result.vectors[0].len(),
                                );
                                if let Some(projection) = self.projection.as_ref() {
                                    match projection
                                        .search_rows(
                                            &table,
                                            subject,
                                            &result.vectors[0],
                                            budget.max_candidates.min(8),
                                        )
                                        .await
                                    {
                                        Ok(probe_rows) => {
                                            let revisions = probe_rows
                                                .iter()
                                                .map(|row| row.revision)
                                                .collect::<Vec<_>>();
                                            let rows=sqlx::query("SELECT revision_id,object_id FROM memory_revisions WHERE subject_id=$1 AND revision_id=ANY($2)").bind(subject.0).bind(&revisions).fetch_all(self.store.pool()).await.map_err(db)?;
                                            let map: BTreeMap<Uuid, Uuid> = rows
                                                .iter()
                                                .map(|row| {
                                                    Ok((
                                                        row.try_get("revision_id").map_err(db)?,
                                                        row.try_get("object_id").map_err(db)?,
                                                    ))
                                                })
                                                .collect::<Result<_>>()?;
                                            let ids: Vec<Uuid> = revisions
                                                .iter()
                                                .filter_map(|id| map.get(id).copied())
                                                .collect();
                                            fuse(&mut scores, &ids);
                                            trace.index_queries += 1;
                                            trace.direct_dense_queries += 1;
                                            trace.candidate_channels.push("dense".into());

                                            let max_rounds = match intent.effort {
                                                RecallEffort::Light => 0,
                                                RecallEffort::Normal => 1,
                                                RecallEffort::Deep => 2,
                                                RecallEffort::Maximum => 3,
                                            };
                                            if max_rounds > 0 {
                                                let mut residual = residual::ResidualState::new(
                                                    &result.vectors[0],
                                                )
                                                .ok_or_else(|| {
                                                    Error::Invalid("invalid query embedding".into())
                                                })?;
                                                let mut probe = probe_rows;
                                                let mut search_vector = result.vectors[0].clone();
                                                for _ in 0..max_rounds {
                                                    if scores.len() >= budget.max_candidates {
                                                        trace.residual_stop_reason = Some(
                                                            "candidate_budget_exhausted".into(),
                                                        );
                                                        break;
                                                    }
                                                    let vectors: Vec<_> = probe
                                                        .iter()
                                                        .take(8)
                                                        .map(|r| r.vector.clone())
                                                        .collect();
                                                    let similarities: Vec<_> = vectors
                                                        .iter()
                                                        .map(|v| {
                                                            v.iter()
                                                                .zip(&search_vector)
                                                                .map(|(a, b)| a * b)
                                                                .sum()
                                                        })
                                                        .collect();
                                                    search_vector = match residual
                                                        .next(&vectors, &similarities)
                                                    {
                                                        Ok(vector) => vector,
                                                        Err(reason) => {
                                                            trace.residual_stop_reason =
                                                                Some(reason.into());
                                                            break;
                                                        }
                                                    };
                                                    probe = projection
                                                        .search_rows(
                                                            &table,
                                                            subject,
                                                            &search_vector,
                                                            budget.max_candidates,
                                                        )
                                                        .await?;
                                                    let revisions: Vec<_> =
                                                        probe.iter().map(|r| r.revision).collect();
                                                    let rows=sqlx::query("SELECT revision_id,object_id FROM memory_revisions WHERE subject_id=$1 AND revision_id=ANY($2)").bind(subject.0).bind(&revisions).fetch_all(self.store.pool()).await.map_err(db)?;
                                                    let map: BTreeMap<Uuid, Uuid> = rows
                                                        .iter()
                                                        .map(|row| {
                                                            Ok((
                                                                row.try_get("revision_id")
                                                                    .map_err(db)?,
                                                                row.try_get("object_id")
                                                                    .map_err(db)?,
                                                            ))
                                                        })
                                                        .collect::<Result<_>>()?;
                                                    let ids: Vec<_> = revisions
                                                        .iter()
                                                        .filter_map(|id| map.get(id).copied())
                                                        .collect();
                                                    let before = scores.len();
                                                    for (rank, id) in ids.iter().enumerate() {
                                                        if scores.contains_key(id)
                                                            || scores.len() < budget.max_candidates
                                                        {
                                                            *scores.entry(*id).or_insert(0.0) +=
                                                                1.0 / (32.0 + rank as f64);
                                                        }
                                                    }
                                                    let added = scores.len() - before;
                                                    trace.residual_candidates_added += added;
                                                    trace.residual_rounds += 1;
                                                    trace.index_queries += 1;
                                                    trace.candidate_channels.push(format!(
                                                        "residual_dense_{}",
                                                        trace.residual_rounds
                                                    ));
                                                    if added == 0 {
                                                        trace.residual_stop_reason =
                                                            Some("no_novel_candidate".into());
                                                        break;
                                                    }
                                                }
                                                trace.residual_energy_ratios =
                                                    residual.energy_ratios;
                                                if trace.residual_stop_reason.is_none() {
                                                    trace.residual_stop_reason =
                                                        Some("round_budget_exhausted".into());
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            degraded.push("dense_projection_unavailable".into())
                                        }
                                    }
                                } else {
                                    degraded.push("dense_projection_unavailable".into());
                                }
                                models.push(provenance);
                            }
                            Err(_) => degraded.push("embedding_service_unavailable".into()),
                        }
                    } else if !self.models.embedding_available() {
                        degraded.push("embedding_not_configured".into());
                    }
                }
                if intent.temporal.occurred.is_some()
                    || intent.temporal.observed.is_some()
                    || intent.temporal.known_by.is_some()
                {
                    let ids = self
                        .temporal_candidates(subject, &intent, budget.max_candidates)
                        .await?;
                    trace.index_queries += 1;
                    trace.candidate_channels.push("temporal".into());
                    fuse(&mut scores, &ids);
                }
                for reference in &intent.cues.references {
                    let ids = self
                        .reference_candidates(subject, *reference, budget.max_candidates)
                        .await?;
                    trace.index_queries += 1;
                    trace.candidate_channels.push("reference_cue".into());
                    fuse(&mut scores, &ids);
                }
            }
        }
        if !first_query {
            exact.extend(
                intent
                    .target
                    .iter()
                    .filter(|node| node.kind == NodeKind::Memory)
                    .map(|node| node.id),
            );
            if !intent.target.is_empty() {
                for id in &exact {
                    scores.insert(*id, 1.0);
                }
            }
        }
        let broaden = intent.objective == RecallObjective::AssociativeRecollection
            || (intent.effort >= RecallEffort::Normal
                && (scores.len() < intent.result_need.limit
                    || intent.effort >= RecallEffort::Deep));
        let mut continuation = false;
        if broaden {
            let direct_scores = scores.clone();
            trace.recall_rounds += 1;
            let mut seeds = intent.cues.references.clone();
            seeds.extend(&intent.target);
            seeds.extend(
                intent
                    .cues
                    .entities
                    .iter()
                    .chain(state.active_entities.iter())
                    .map(|id| NodeRef {
                        kind: NodeKind::Entity,
                        id: *id,
                    }),
            );
            let mut ranked: Vec<_> = scores.iter().collect();
            ranked.sort_by(|a, b| b.1.total_cmp(a.1).then(a.0.cmp(b.0)));
            seeds.extend(ranked.into_iter().take(16).map(|(id, _)| NodeRef {
                kind: NodeKind::Memory,
                id: *id,
            }));
            let config = self.association_config.clone();
            self.expand_graph(
                subject,
                state,
                &seeds,
                budget.max_association_edges,
                budget.max_association_states,
                config.max_hops,
            )
            .await?;
            state.activation.seed(&seeds, budget.max_association_states);
            let activation = state.activation.advance(
                &state.graph,
                &config,
                budget.max_association_states,
                budget.max_association_edges,
                budget.max_association_states,
            )?;
            continuation = activation.remaining_frontier > 0;
            trace.edges_visited = activation.edges_visited;
            trace.association_nodes_activated = activation.states_expanded;
            trace.max_graph_depth = activation.max_depth;
            trace.activation_seed_count = activation.activation_seed_count;
            trace.actual_flow_edges = activation.actual_flow_edges;
            trace.positive_flow_entropy = activation.positive_flow_entropy;
            trace.emergent_support_ratio = activation.emergent_support_ratio;
            trace.activation_budget_truncated = activation.budget_truncated;
            trace.association_observability = association_observability(&activation);
            trace.candidate_channels.push("association".into());
            let mut active: Vec<_> = state.activation.scores().into_iter().collect();
            active.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            for (node, association_score) in active
                .iter()
                .filter(|(node, _)| node.kind == NodeKind::Memory)
            {
                let direct_score = direct_scores
                    .get(&node.id)
                    .copied()
                    .unwrap_or(0.0)
                    .clamp(0.0, 1.0);
                let effective = association_score.clamp(0.0, 1.0)
                    * (0.35 + 0.65 * trace.association_observability);
                scores.insert(node.id, 1.0 - (1.0 - direct_score) * (1.0 - effective));
            }
            for (node, _) in active
                .into_iter()
                .filter(|(node, _)| node.kind != NodeKind::Memory)
                .take(16)
            {
                let ids = self
                    .reference_candidates(subject, node, budget.max_candidates)
                    .await?;
                trace.index_queries += 1;
                fuse(&mut scores, &ids);
            }
        }
        if matches!(
            intent.objective,
            RecallObjective::EpisodeReconstruction
                | RecallObjective::Explanation
                | RecallObjective::Verification
                | RecallObjective::Comparison
        ) {
            let seeds: Vec<Uuid> = scores.keys().copied().take(budget.max_candidates).collect();
            let ids:Vec<Uuid>=sqlx::query_scalar("SELECT id FROM (SELECT episode_id AS id FROM episode_members WHERE subject_id=$1 AND member_id=ANY($2) UNION SELECT member_id FROM episode_members WHERE subject_id=$1 AND episode_id=ANY($2)) related ORDER BY id LIMIT $3")
                .bind(subject.0).bind(seeds).bind(budget.max_candidates as i64).fetch_all(self.store.pool()).await.map_err(db)?;
            trace.index_queries += 1;
            trace.candidate_channels.push("episodic_relation".into());
            fuse(&mut scores, &ids);
        }
        let mut ranked: Vec<_> = scores.into_iter().collect();
        ranked.sort_by(|a, b| {
            exact
                .contains(&b.0)
                .cmp(&exact.contains(&a.0))
                .then(b.1.total_cmp(&a.1))
                .then(a.0.cmp(&b.0))
        });
        ranked.truncate(budget.max_candidates);
        state.candidates = ranked.iter().copied().collect();
        let supports = state.activation.supports();
        let mut results = vec![];
        for (object, score) in ranked {
            if state.exhausted.contains(&object) && !exact.contains(&object) {
                trace.repeated_candidates += 1;
                continue;
            }
            let revisions = self.admissible_revisions(subject, object, &intent).await?;
            for revision in revisions {
                trace.candidates_examined += 1;
                let memory = self.memory(subject, object, Some(revision)).await?;
                if memory.suppressed
                    || memory.availability != "ready"
                    || (memory.superseded_by.is_some()
                        && intent.objective == RecallObjective::Current)
                {
                    continue;
                }
                let row=sqlx::query("SELECT a.meaningful_uses,COALESCE(a.last_meaningful_use,o.created_at) AS last_use,a.retention_hint FROM accessibility_state a JOIN memory_objects o USING(object_id) WHERE object_id=$1")
                    .bind(object).fetch_one(self.store.pool()).await.map_err(db)?;
                let access = accessibility(
                    row.try_get::<i64, _>("meaningful_uses").map_err(db)? as u64,
                    row.try_get::<DateTime<Utc>, _>("last_use").map_err(db)?,
                    Utc::now(),
                    self.use_decay,
                    row.try_get("retention_hint").map_err(db)?,
                );
                let threshold = match intent.effort {
                    RecallEffort::Light => 0.15,
                    RecallEffort::Normal => 0.05,
                    RecallEffort::Deep | RecallEffort::Maximum => 0.0,
                };
                if !exact.contains(&object) && access < threshold {
                    continue;
                }
                state.active_entities.extend(&memory.content.entities);
                results.push(RecallHit {
                    memory,
                    rank_score: if exact.contains(&object) {
                        1.0
                    } else {
                        score * (0.25 + 0.75 * access)
                    },
                    exact: exact.contains(&object),
                    association_support: supports
                        .iter()
                        .filter(|support| support.target.id == object)
                        .cloned()
                        .collect(),
                });
            }
        }
        results.sort_by(|a, b| {
            b.exact
                .cmp(&a.exact)
                .then(b.rank_score.total_cmp(&a.rank_score))
                .then(a.memory.object_id.cmp(&b.memory.object_id))
        });
        if results.len() > 1
            && exact.is_empty()
            && self.models.rerank.is_some()
            && intent.effort != RecallEffort::Light
            && let Some(text) = &intent.cues.text
        {
            let documents = results
                .iter()
                .map(|hit| format!("{}\n{}", hit.memory.content.title, hit.memory.content.text))
                .collect::<Vec<_>>();
            match self.models.rerank(text, &documents).await {
                Ok((rerank, provenance)) => {
                    let old = results;
                    results = rerank
                        .ranked
                        .iter()
                        .map(|rank| old[rank.index].clone())
                        .collect();
                    trace.reranker_calls += 1;
                    models.push(provenance);
                }
                Err(_) => degraded.push("reranker_unavailable".into()),
            }
        }
        results.truncate(intent.result_need.limit);
        let ids: Vec<Uuid> = results.iter().map(|hit| hit.memory.object_id).collect();
        self.record_use(
            subject,
            intent.cycle_id,
            Uuid::now_v7(),
            UseKind::Surfaced,
            &ids,
            None,
            None,
        )
        .await?;
        trace.wall_time_ms = started.elapsed().as_secs_f64() * 1000.0;
        Ok(RecallResponse {
            api_version: 1,
            recall_id: Uuid::now_v7(),
            cycle_id: intent.cycle_id,
            sufficiency: if results.len() >= intent.result_need.limit {
                "requested_count_met"
            } else {
                "insufficient_candidates"
            }
            .into(),
            results,
            degraded,
            effort: trace,
            models,
            can_continue: continuation,
        })
    }

    async fn reference_candidates(
        &self,
        subject: SubjectId,
        node: NodeRef,
        limit: usize,
    ) -> Result<Vec<Uuid>> {
        let query = match node.kind {
            NodeKind::Memory => {
                "SELECT object_id FROM memory_objects WHERE subject_id=$1 AND object_id=$2 LIMIT $3"
            }
            NodeKind::Source => {
                "SELECT DISTINCT r.object_id FROM memory_revision_sources s JOIN memory_revisions r USING(revision_id) WHERE s.subject_id=$1 AND s.source_id=$2 ORDER BY r.object_id LIMIT $3"
            }
            NodeKind::Artifact => {
                "SELECT DISTINCT r.object_id FROM memory_revision_artifacts a JOIN memory_revisions r USING(revision_id) WHERE a.subject_id=$1 AND a.artifact_id=$2 ORDER BY r.object_id LIMIT $3"
            }
            NodeKind::Entity => {
                "SELECT DISTINCT r.object_id FROM memory_entity_mentions e JOIN memory_revisions r USING(revision_id) WHERE e.subject_id=$1 AND e.entity_id=$2 ORDER BY r.object_id LIMIT $3"
            }
        };
        sqlx::query_scalar(query)
            .bind(subject.0)
            .bind(node.id)
            .bind(limit as i64)
            .fetch_all(self.store.pool())
            .await
            .map_err(db)
    }
    async fn temporal_candidates(
        &self,
        subject: SubjectId,
        intent: &RecallIntent,
        limit: usize,
    ) -> Result<Vec<Uuid>> {
        sqlx::query_scalar("SELECT DISTINCT object_id FROM memory_revisions WHERE subject_id=$1 AND ($2::timestamptz IS NULL OR recorded_at<=$2) AND (NOT $3 OR occurred && tstzrange($4,$5,'[]')) AND (NOT $6 OR (observed_at IS NOT NULL AND ($7::timestamptz IS NULL OR observed_at>=$7) AND ($8::timestamptz IS NULL OR observed_at<=$8))) ORDER BY object_id LIMIT $9")
            .bind(subject.0).bind(intent.temporal.known_by).bind(intent.temporal.occurred.is_some()).bind(intent.temporal.occurred.as_ref().and_then(|r|r.start)).bind(intent.temporal.occurred.as_ref().and_then(|r|r.end))
            .bind(intent.temporal.observed.is_some()).bind(intent.temporal.observed.as_ref().and_then(|r|r.start)).bind(intent.temporal.observed.as_ref().and_then(|r|r.end)).bind(limit as i64).fetch_all(self.store.pool()).await.map_err(db)
    }
    async fn admissible_revisions(
        &self,
        subject: SubjectId,
        object: Uuid,
        intent: &RecallIntent,
    ) -> Result<Vec<Uuid>> {
        let evolution = matches!(
            intent.objective,
            RecallObjective::Evolution | RecallObjective::Transition
        );
        let current = intent.temporal.known_by.is_none()
            && !evolution
            && !matches!(intent.objective, RecallObjective::Historical);
        let classes: Vec<String> = intent
            .constraints
            .epistemic
            .iter()
            .map(crate::material::enum_name)
            .collect::<Result<_>>()?;
        sqlx::query_scalar("SELECT r.revision_id FROM memory_revisions r JOIN memory_objects o USING(object_id) LEFT JOIN memory_current_heads h ON h.object_id=o.object_id WHERE r.subject_id=$1 AND r.object_id=$2 AND ($3::text IS NULL OR o.scope=$3) AND ($4::timestamptz IS NULL OR r.recorded_at<=$4) AND (NOT $5 OR r.revision_id=h.revision_id) AND (NOT $6 OR r.occurred && tstzrange($7,$8,'[]')) AND (NOT $9 OR (r.observed_at IS NOT NULL AND ($10::timestamptz IS NULL OR r.observed_at>=$10) AND ($11::timestamptz IS NULL OR r.observed_at<=$11))) AND (cardinality($12::text[])=0 OR r.epistemic_class=ANY($12)) ORDER BY r.recorded_at DESC,r.revision_id DESC LIMIT $13")
            .bind(subject.0).bind(object).bind(&intent.constraints.scope).bind(intent.temporal.known_by).bind(current)
            .bind(intent.temporal.occurred.is_some()).bind(intent.temporal.occurred.as_ref().and_then(|r|r.start)).bind(intent.temporal.occurred.as_ref().and_then(|r|r.end))
            .bind(intent.temporal.observed.is_some()).bind(intent.temporal.observed.as_ref().and_then(|r|r.start)).bind(intent.temporal.observed.as_ref().and_then(|r|r.end)).bind(classes).bind(if evolution {intent.result_need.limit as i64}else{1})
            .fetch_all(self.store.pool()).await.map_err(db)
    }
}
fn fuse(scores: &mut BTreeMap<Uuid, f64>, ids: &[Uuid]) {
    let mut seen = BTreeSet::new();
    for (rank, id) in ids.iter().enumerate() {
        if seen.insert(*id) {
            *scores.entry(*id).or_default() += 1.0 / (60.0 + rank as f64 + 1.0);
        }
    }
}

fn association_observability(trace: &nous_memory_retrieval::association::ActivationTrace) -> f64 {
    if trace.actual_flow_edges == 0 {
        return 0.0;
    }
    let seeds = trace.activation_seed_count.max(1) as f64;
    let edge_sufficiency = 1.0 - (-(trace.actual_flow_edges as f64) / (2.0 * seeds)).exp();
    let completion_factor = if trace.budget_truncated { 0.5 } else { 1.0 };
    (edge_sufficiency
        * trace.emergent_support_ratio.max(1e-6)
        * trace.positive_flow_entropy.max(1e-6)
        * completion_factor)
        .powf(0.25)
        .clamp(0.0, 1.0)
}
