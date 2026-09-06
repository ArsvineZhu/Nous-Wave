use crate::{
    LocalRuntime,
    material::{decode_enum, enum_name},
    memory::lock_subject,
    subjects::db,
};
use nous_core::{Error, Result, SubjectId};
use nous_memory_domain::{
    learning::UseKind,
    recall::{NodeKind, NodeRef},
};
use nous_memory_retrieval::association::{ActivationState, AssociationEdge};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub(crate) struct WorkingState {
    pub subject: Option<SubjectId>,
    pub candidates: BTreeMap<Uuid, f64>,
    pub inspected: BTreeSet<Uuid>,
    pub exposed: BTreeSet<Uuid>,
    pub exhausted: BTreeSet<Uuid>,
    pub active_entities: BTreeSet<Uuid>,
    pub queried: BTreeSet<String>,
    pub activation: ActivationState,
    pub graph: Vec<AssociationEdge>,
    pub graph_loaded: BTreeSet<NodeRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseFeedback {
    pub api_version: u32,
    pub event_id: Uuid,
    pub kind: UseKind,
    pub object_refs: Vec<Uuid>,
    pub followed_from: Option<Uuid>,
    pub causation_id: Option<Uuid>,
}

impl LocalRuntime {
    pub async fn begin_cycle(
        &self,
        subject: SubjectId,
        context: serde_json::Value,
    ) -> Result<Uuid> {
        self.require_memory(subject).await?;
        if context.to_string().len() > 65536 {
            return Err(Error::Invalid("cycle context exceeds budget".into()));
        }
        let mut cycles = self.cycles.lock().await;
        if cycles.len() >= 1000
            || cycles
                .values()
                .filter(|cycle| cycle.subject == Some(subject))
                .count()
                >= 64
        {
            return Err(Error::Unavailable(
                "active cognitive cycle budget exhausted; close an existing cycle".into(),
            ));
        }
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO cognitive_cycles(cycle_id,subject_id,process_id,status,context) VALUES($1,$2,$3,'active',$4)")
            .bind(id).bind(subject.0).bind(self.process_id).bind(context).execute(self.store.pool()).await.map_err(db)?;
        cycles.insert(
            id,
            WorkingState {
                subject: Some(subject),
                ..Default::default()
            },
        );
        Ok(id)
    }

    pub(crate) async fn check_cycle(&self, subject: SubjectId, cycle: Uuid) -> Result<()> {
        self.require_memory(subject).await?;
        let active:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM cognitive_cycles WHERE subject_id=$1 AND cycle_id=$2 AND process_id=$3 AND status='active')")
            .bind(subject.0).bind(cycle).bind(self.process_id).fetch_one(self.store.pool()).await.map_err(db)?;
        if !active {
            return Err(Error::Conflict("cycle is closed, belongs to another subject, or its process-local state ended on restart".into()));
        }
        Ok(())
    }

    pub async fn inspect_cycle(
        &self,
        subject: SubjectId,
        cycle: Uuid,
        objects: &[Uuid],
    ) -> Result<Vec<crate::memory::MemoryView>> {
        self.check_cycle(subject, cycle).await?;
        if objects.len() > 64 {
            return Err(Error::Invalid("inspection object budget exceeded".into()));
        }
        let mut views = vec![];
        for object in objects {
            views.push(self.memory(subject, *object, None).await?);
        }
        self.record_use(
            subject,
            Some(cycle),
            Uuid::now_v7(),
            UseKind::Inspected,
            objects,
            None,
            None,
        )
        .await?;
        let mut cycles = self.cycles.lock().await;
        let state = cycles
            .get_mut(&cycle)
            .ok_or_else(|| Error::Conflict("cycle state no longer available".into()))?;
        state.inspected.extend(objects);
        state.exhausted.extend(objects);
        for view in &views {
            state.active_entities.extend(&view.content.entities);
        }
        Ok(views)
    }

    pub async fn feedback(
        &self,
        subject: SubjectId,
        cycle: Uuid,
        input: UseFeedback,
    ) -> Result<()> {
        crate::subjects::check_version(input.api_version)?;
        self.check_cycle(subject, cycle).await?;
        if !matches!(
            input.kind,
            UseKind::ExposedToContext
                | UseKind::Followed
                | UseKind::ReferencedOrActedOn
                | UseKind::Abandoned
        ) || input.object_refs.len() > 64
        {
            return Err(Error::Invalid(
                "unsupported host feedback or exceeded object budget".into(),
            ));
        }
        self.record_use(
            subject,
            Some(cycle),
            input.event_id,
            input.kind,
            &input.object_refs,
            input.followed_from,
            input.causation_id,
        )
        .await?;
        let mut cycles = self.cycles.lock().await;
        let state = cycles
            .get_mut(&cycle)
            .ok_or_else(|| Error::Conflict("cycle state no longer available".into()))?;
        if input.kind == UseKind::ExposedToContext {
            state.exposed.extend(&input.object_refs);
        }
        if matches!(
            input.kind,
            UseKind::Abandoned | UseKind::ReferencedOrActedOn
        ) {
            state.exhausted.extend(&input.object_refs);
        }
        if input.kind == UseKind::Followed {
            state.activation.seed(
                &input
                    .object_refs
                    .iter()
                    .map(|id| NodeRef {
                        kind: NodeKind::Memory,
                        id: *id,
                    })
                    .collect::<Vec<_>>(),
                5000,
            );
        }
        Ok(())
    }

    pub async fn close_cycle(&self, subject: SubjectId, cycle: Uuid, outcome: &str) -> Result<()> {
        let kind = match outcome {
            "resolved" => UseKind::CycleResolved,
            "insufficient" => UseKind::CycleInsufficient,
            "abandoned" => UseKind::Abandoned,
            _ => return Err(Error::Invalid("invalid cycle outcome".into())),
        };
        self.check_cycle(subject, cycle).await?;
        self.record_use(subject, Some(cycle), Uuid::now_v7(), kind, &[], None, None)
            .await?;
        sqlx::query("UPDATE cognitive_cycles SET status=$3,closed_at=clock_timestamp() WHERE subject_id=$1 AND cycle_id=$2").bind(subject.0).bind(cycle).bind(outcome).execute(self.store.pool()).await.map_err(db)?;
        self.cycles.lock().await.remove(&cycle);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn record_use(
        &self,
        subject: SubjectId,
        cycle: Option<Uuid>,
        event: Uuid,
        kind: UseKind,
        objects: &[Uuid],
        from: Option<Uuid>,
        causation: Option<Uuid>,
    ) -> Result<()> {
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        lock_subject(&mut tx, subject).await?;
        for object in objects.iter().copied().chain(from) {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM memory_objects WHERE subject_id=$1 AND object_id=$2)",
            )
            .bind(subject.0)
            .bind(object)
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?;
            if !exists {
                return Err(Error::NotFound("feedback memory object not found".into()));
            }
        }
        let inserted=sqlx::query("INSERT INTO cognitive_use_events(event_id,subject_id,cycle_id,kind,object_refs,causation_id) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(event_id) DO NOTHING")
            .bind(event).bind(subject.0).bind(cycle).bind(enum_name(&kind)?).bind(objects).bind(causation).execute(&mut *tx).await.map_err(db)?;
        if inserted.rows_affected() == 0 {
            tx.rollback().await.map_err(db)?;
            return Ok(());
        }
        if kind.strengthens() {
            sqlx::query("UPDATE accessibility_state SET meaningful_uses=meaningful_uses+1,last_meaningful_use=clock_timestamp() WHERE object_id=ANY($1)").bind(objects).execute(&mut *tx).await.map_err(db)?;
            let pairs: Vec<(Uuid, Uuid)> = if let Some(from) = from {
                objects
                    .iter()
                    .filter(|id| **id != from)
                    .map(|id| (from, *id))
                    .collect()
            } else {
                objects
                    .windows(2)
                    .filter(|pair| pair[0] != pair[1])
                    .map(|pair| (pair[0], pair[1]))
                    .collect()
            };
            for (left, right) in pairs {
                sqlx::query("INSERT INTO association_evidence(evidence_id,subject_id,from_kind,from_id,to_kind,to_id,evidence_class,support,use_event_id) VALUES($1,$2,'memory',$3,'memory',$4,$5,1,$6)")
                    .bind(Uuid::now_v7()).bind(subject.0).bind(left).bind(right).bind(if kind==UseKind::Followed {"followed"} else {"meaningful_co_recall"}).bind(event).execute(&mut *tx).await.map_err(db)?;
            }
        }
        tx.commit().await.map_err(db)
    }

    pub async fn link_association(
        &self,
        subject: SubjectId,
        from: NodeRef,
        to: NodeRef,
        source: Option<Uuid>,
    ) -> Result<Uuid> {
        self.require_memory(subject).await?;
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        lock_subject(&mut tx, subject).await?;
        for node in [from, to] {
            let query = match node.kind {
                NodeKind::Memory => {
                    "SELECT EXISTS(SELECT 1 FROM memory_objects WHERE subject_id=$1 AND object_id=$2)"
                }
                NodeKind::Entity => {
                    "SELECT EXISTS(SELECT 1 FROM memory_entities WHERE subject_id=$1 AND entity_id=$2)"
                }
                NodeKind::Source => {
                    "SELECT EXISTS(SELECT 1 FROM source_records WHERE subject_id=$1 AND source_id=$2)"
                }
                NodeKind::Artifact => {
                    "SELECT EXISTS(SELECT 1 FROM artifacts WHERE subject_id=$1 AND artifact_id=$2)"
                }
            };
            let exists: bool = sqlx::query_scalar(query)
                .bind(subject.0)
                .bind(node.id)
                .fetch_one(&mut *tx)
                .await
                .map_err(db)?;
            if !exists {
                return Err(Error::NotFound(
                    "association node not found in subject".into(),
                ));
            }
        }
        let id = Uuid::now_v7();
        sqlx::query("INSERT INTO association_evidence(evidence_id,subject_id,from_kind,from_id,to_kind,to_id,evidence_class,support,source_id) VALUES($1,$2,$3,$4,$5,$6,'host_explicit',1,$7)")
            .bind(id).bind(subject.0).bind(enum_name(&from.kind)?).bind(from.id).bind(enum_name(&to.kind)?).bind(to.id).bind(source).execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)?;
        Ok(id)
    }

    pub async fn association_evidence(
        &self,
        subject: SubjectId,
        object: Uuid,
    ) -> Result<serde_json::Value> {
        self.require_memory(subject).await?;
        let rows = sqlx::query(
            "SELECT evidence_id,from_kind,from_id,to_kind,to_id,evidence_class,support,source_id,use_event_id,created_at FROM association_evidence WHERE subject_id=$1 AND ((from_kind='memory' AND from_id=$2) OR (to_kind='memory' AND to_id=$2)) ORDER BY created_at,evidence_id LIMIT 512",
        )
        .bind(subject.0)
        .bind(object)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?;
        let evidence = rows
            .into_iter()
            .map(|row| {
                Ok(serde_json::json!({
                    "evidence_id": row.try_get::<Uuid, _>("evidence_id").map_err(db)?,
                    "from": {"kind": row.try_get::<String, _>("from_kind").map_err(db)?, "id": row.try_get::<Uuid, _>("from_id").map_err(db)?},
                    "to": {"kind": row.try_get::<String, _>("to_kind").map_err(db)?, "id": row.try_get::<Uuid, _>("to_id").map_err(db)?},
                    "evidence_class": row.try_get::<String, _>("evidence_class").map_err(db)?,
                    "support": row.try_get::<f64, _>("support").map_err(db)?,
                    "source_id": row.try_get::<Option<Uuid>, _>("source_id").map_err(db)?,
                    "use_event_id": row.try_get::<Option<Uuid>, _>("use_event_id").map_err(db)?,
                    "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").map_err(db)?,
                }))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(serde_json::json!({"api_version": 1, "object_id": object, "evidence": evidence}))
    }

    pub(crate) async fn expand_graph(
        &self,
        subject: SubjectId,
        state: &mut WorkingState,
        seeds: &[NodeRef],
        max_edges: usize,
        max_states: usize,
        max_hops: usize,
    ) -> Result<usize> {
        let mut frontier = seeds.to_vec();
        let initial = state.graph.len();
        for _ in 0..max_hops {
            frontier.retain(|node| !state.graph_loaded.contains(node));
            if frontier.is_empty()
                || state.graph.len() >= max_edges
                || state.graph_loaded.len() >= max_states
            {
                break;
            }
            frontier.truncate(max_states - state.graph_loaded.len());
            let ids: Vec<Uuid> = frontier.iter().map(|node| node.id).collect();
            let rows=sqlx::query("SELECT e.*,(SELECT count(*) FROM association_evidence i WHERE i.subject_id=e.subject_id AND i.to_id=e.to_id AND i.to_kind=e.to_kind) AS inbound FROM association_evidence e WHERE e.subject_id=$1 AND e.from_id=ANY($2) ORDER BY e.from_id,e.evidence_id LIMIT $3")
                .bind(subject.0).bind(&ids).bind((max_edges-state.graph.len()) as i64).fetch_all(self.store.pool()).await.map_err(db)?;
            state.graph_loaded.extend(&frontier);
            frontier.clear();
            for row in rows {
                let edge = AssociationEdge {
                    evidence_id: row.try_get("evidence_id").map_err(db)?,
                    from: NodeRef {
                        kind: decode_enum(row.try_get("from_kind").map_err(db)?)?,
                        id: row.try_get("from_id").map_err(db)?,
                    },
                    to: NodeRef {
                        kind: decode_enum(row.try_get("to_kind").map_err(db)?)?,
                        id: row.try_get("to_id").map_err(db)?,
                    },
                    evidence_class: row.try_get("evidence_class").map_err(db)?,
                    support: row.try_get("support").map_err(db)?,
                    target_inbound_degree: row.try_get::<i64, _>("inbound").map_err(db)? as usize,
                };
                frontier.push(edge.to);
                state.graph.push(edge);
            }
        }
        Ok(state.graph.len() - initial)
    }
}
