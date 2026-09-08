use crate::{AuthorityStore, database_error as db, projection_input::watermark};
use nous_core::*;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdgeSource {
    pub from: CognitiveRef,
    pub to: CognitiveRef,
    pub support_class: String,
    pub association_kind: String,
    pub polarity: String,
    pub support_value: f64,
    pub bridge_hint: bool,
}

pub struct TopologyProjectionInput {
    pub watermark: i64,
    pub nodes: Vec<CognitiveRef>,
    pub edges: Vec<TopologyEdgeSource>,
}

fn link(from: CognitiveRef, to: CognitiveRef, kind: &str) -> TopologyEdgeSource {
    TopologyEdgeSource {
        from,
        to,
        support_class: "memory_evidence".into(),
        association_kind: kind.into(),
        polarity: "positive".into(),
        support_value: 1.0,
        bridge_hint: false,
    }
}

impl AuthorityStore {
    pub async fn topology_projection_input(
        &self,
        subject: SubjectId,
        memory_enabled: bool,
    ) -> Result<TopologyProjectionInput> {
        let mut tx = self.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        let watermark = watermark(&mut tx, subject, "topology", "").await?;
        let mut nodes = HashSet::new();
        let mut edges = Vec::new();
        if memory_enabled {
            let sources = crate::projection_input::memory_sources(&mut tx, subject).await?;
            for source in sources {
                nodes.insert(source.reference.clone());
                for tag in source.tag_ids {
                    let tag = CognitiveRef::Tag(TagId(
                        tag.parse()
                            .map_err(|_| Error::Infrastructure("invalid Tag id".into()))?,
                    ));
                    nodes.insert(tag.clone());
                    edges.push(link(source.reference.clone(), tag.clone(), "structural"));
                    edges.push(link(tag, source.reference.clone(), "structural"));
                }
                for entity in source.entity_refs {
                    let entity = CognitiveRef::Entity(EntityRef::new(entity)?);
                    nodes.insert(entity.clone());
                    edges.push(link(
                        source.reference.clone(),
                        entity.clone(),
                        "experiential",
                    ));
                    edges.push(link(entity, source.reference.clone(), "experiential"));
                }
            }
            let rows = sqlx::query("SELECT from_ref_kind,from_ref,to_ref_kind,to_ref,support_class,association_kind,polarity,support_value,bridge_hint FROM association_evidence WHERE subject_id=$1 AND revoked_at IS NULL AND (valid_from IS NULL OR valid_from<=now()) AND (valid_to IS NULL OR valid_to>=now())")
                .bind(subject.0).fetch_all(&mut *tx).await.map_err(db)?;
            for row in rows {
                let from = parse_reference(
                    &row.try_get::<String, _>("from_ref_kind").map_err(db)?,
                    &row.try_get::<String, _>("from_ref").map_err(db)?,
                )?;
                let to = parse_reference(
                    &row.try_get::<String, _>("to_ref_kind").map_err(db)?,
                    &row.try_get::<String, _>("to_ref").map_err(db)?,
                )?;
                if !allowed(&from) || !allowed(&to) {
                    continue;
                }
                nodes.insert(from.clone());
                nodes.insert(to.clone());
                edges.push(TopologyEdgeSource {
                    from,
                    to,
                    support_class: row.try_get("support_class").map_err(db)?,
                    association_kind: row.try_get("association_kind").map_err(db)?,
                    polarity: row.try_get("polarity").map_err(db)?,
                    support_value: row.try_get("support_value").map_err(db)?,
                    bridge_hint: row.try_get("bridge_hint").map_err(db)?,
                });
            }
            let rows = sqlx::query("SELECT a.anchor_id,s.support_ref_kind,s.support_ref FROM anchors a JOIN anchor_support s ON s.anchor_revision_id=a.current_revision_id WHERE a.subject_id=$1 AND a.status='active'")
                .bind(subject.0).fetch_all(&mut *tx).await.map_err(db)?;
            for row in rows {
                let anchor = CognitiveRef::Anchor(AnchorId(row.try_get("anchor_id").map_err(db)?));
                let mut support = parse_reference(
                    &row.try_get::<String, _>("support_ref_kind").map_err(db)?,
                    &row.try_get::<String, _>("support_ref").map_err(db)?,
                )?;
                if let CognitiveRef::MemoryRevision(revision) = support {
                    let memory: Uuid = sqlx::query_scalar(
                        "SELECT memory_id FROM memory_revisions WHERE memory_revision_id=$1",
                    )
                    .bind(revision.0)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(db)?;
                    support = CognitiveRef::Memory(MemoryId(memory));
                }
                if !allowed(&support) {
                    continue;
                }
                nodes.insert(anchor.clone());
                nodes.insert(support.clone());
                edges.push(link(anchor.clone(), support.clone(), "anchor"));
                edges.push(link(support, anchor, "anchor"));
            }
            let rows = sqlx::query("SELECT l.memory_id AS from_memory,r.memory_id AS to_memory,rr.relation FROM memory_revision_relations rr JOIN memory_revisions l ON l.memory_revision_id=rr.from_revision_id JOIN memory_revisions r ON r.memory_revision_id=rr.to_revision_id JOIN memory_objects lo ON lo.memory_id=l.memory_id JOIN memory_objects ro ON ro.memory_id=r.memory_id WHERE lo.subject_id=$1 AND ro.subject_id=$1 AND lo.status='active' AND ro.status='active'")
                .bind(subject.0).fetch_all(&mut *tx).await.map_err(db)?;
            for row in rows {
                edges.push(link(
                    CognitiveRef::Memory(MemoryId(row.try_get("from_memory").map_err(db)?)),
                    CognitiveRef::Memory(MemoryId(row.try_get("to_memory").map_err(db)?)),
                    &row.try_get::<String, _>("relation").map_err(db)?,
                ));
            }
        }
        for resource in sqlx::query_scalar::<_, String>(
            "SELECT resource_ref FROM resources WHERE subject_id=$1",
        )
        .bind(subject.0)
        .fetch_all(&mut *tx)
        .await
        .map_err(db)?
        {
            nodes.insert(CognitiveRef::Resource(ResourceRef::new(resource)?));
        }
        let mut nodes: Vec<_> = nodes.into_iter().collect();
        nodes.sort_by_key(ToString::to_string);
        tx.commit().await.map_err(db)?;
        Ok(TopologyProjectionInput {
            watermark,
            nodes,
            edges,
        })
    }
}

fn allowed(reference: &CognitiveRef) -> bool {
    matches!(
        reference,
        CognitiveRef::Memory(_)
            | CognitiveRef::Tag(_)
            | CognitiveRef::Anchor(_)
            | CognitiveRef::Entity(_)
            | CognitiveRef::Resource(_)
    )
}
