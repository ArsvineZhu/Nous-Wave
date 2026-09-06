use crate::{
    LocalRuntime,
    memory::{MemoryDraft, MemoryView, form_memory, lock_subject},
    subjects::{check_version, db},
};
use nous_core::{Error, Result, SubjectId};
use nous_material::EpistemicClass;
use nous_memory_domain::MemoryKind;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidateRequest {
    pub api_version: u32,
    pub operation_id: Uuid,
    pub scope: Option<String>,
    pub targets: Vec<Uuid>,
    pub max_objects: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidateResult {
    pub operation_id: Uuid,
    pub created: Vec<Uuid>,
    pub superseded: Vec<Uuid>,
}

impl LocalRuntime {
    pub async fn consolidate(
        &self,
        subject: SubjectId,
        input: ConsolidateRequest,
    ) -> Result<ConsolidateResult> {
        check_version(input.api_version)?;
        self.require_memory(subject).await?;
        if input.max_objects < 2 || input.max_objects > 256 || input.targets.len() > 256 {
            return Err(Error::Invalid(
                "consolidation budget must be 2..256 objects".into(),
            ));
        }
        let request = serde_json::to_value(&input).map_err(|e| Error::Invalid(e.to_string()))?;
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        lock_subject(&mut tx, subject).await?;
        if let Some(row)=sqlx::query("SELECT request,result,subject_id,kind FROM memory_operation_results WHERE operation_id=$1").bind(input.operation_id).fetch_optional(&mut *tx).await.map_err(db)? {
            if row.try_get::<Uuid,_>("subject_id").map_err(db)?!=subject.0 || row.try_get::<String,_>("kind").map_err(db)?!="consolidate" || row.try_get::<serde_json::Value,_>("request").map_err(db)?!=request {return Err(Error::Conflict("operation identity reused with a different request".into()));}
            return serde_json::from_value(row.try_get("result").map_err(db)?).map_err(|e|Error::Infrastructure(e.to_string()));
        }
        let objects:Vec<Uuid>=sqlx::query_scalar("SELECT o.object_id FROM memory_objects o LEFT JOIN suppression_state s USING(object_id) WHERE o.subject_id=$1 AND o.availability='ready' AND o.superseded_by IS NULL AND NOT COALESCE(s.suppressed,false) AND ($2::text IS NULL OR o.scope=$2) AND (cardinality($3::uuid[])=0 OR o.object_id=ANY($3)) ORDER BY o.object_id LIMIT $4")
            .bind(subject.0).bind(&input.scope).bind(&input.targets).bind(input.max_objects as i64).fetch_all(&mut *tx).await.map_err(db)?;
        let mut memories = vec![];
        for object in objects {
            memories.push(self.memory(subject, object, None).await?);
        }
        let mut duplicate_groups: BTreeMap<String, Vec<MemoryView>> = BTreeMap::new();
        let mut episodes: BTreeMap<String, Vec<MemoryView>> = BTreeMap::new();
        for memory in memories {
            if memory.content.kind == MemoryKind::Episode {
                episodes
                    .entry(memory.content.scope.clone())
                    .or_default()
                    .push(memory.clone());
            }
            if !memory.content.text.is_empty() {
                let key = serde_json::to_string(&(
                    &memory.content.scope,
                    &memory.content.kind,
                    &memory.content.classification,
                    memory
                        .content
                        .text
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" "),
                ))
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
                duplicate_groups.entry(key).or_default().push(memory);
            }
        }
        let mut result = ConsolidateResult {
            operation_id: input.operation_id,
            created: vec![],
            superseded: vec![],
        };
        for group in duplicate_groups.values().filter(|group| group.len() > 1) {
            let draft = merge_support(group, group[0].content.clone());
            let (object, revision) = form_memory(
                &mut tx,
                subject,
                &draft,
                &format!(
                    "consolidation:{}:{}",
                    input.operation_id,
                    result.created.len()
                ),
            )
            .await?;
            for parent in group {
                sqlx::query("INSERT INTO memory_revision_parents(subject_id,revision_id,parent_revision_id,relation) VALUES($1,$2,$3,'CONSOLIDATION')").bind(subject.0).bind(revision).bind(parent.revision_id).execute(&mut *tx).await.map_err(db)?;
                sqlx::query("UPDATE memory_objects SET superseded_by=$2 WHERE object_id=$1")
                    .bind(parent.object_id)
                    .bind(object)
                    .execute(&mut *tx)
                    .await
                    .map_err(db)?;
                result.superseded.push(parent.object_id);
            }
            result.created.push(object);
        }
        for group in episodes.values().filter(|group| group.len() > 1) {
            let source_ids: Vec<Uuid> = group
                .iter()
                .flat_map(|m| m.content.source_refs.iter().copied())
                .collect();
            let kinds:Vec<String>=sqlx::query_scalar("SELECT source_kind FROM source_records WHERE subject_id=$1 AND source_id=ANY($2) GROUP BY source_kind HAVING count(*)>1 ORDER BY source_kind")
                .bind(subject.0).bind(source_ids).fetch_all(&mut *tx).await.map_err(db)?;
            if kinds.is_empty() {
                continue;
            }
            let mut draft = merge_support(group, group[0].content.clone());
            draft.kind = MemoryKind::Semantic;
            draft.title = "Recurring episode evidence".into();
            draft.text = format!(
                "Across {} episodes in this scope, repeated source activity was recorded: {}. This abstraction records recurrence, not agreement or truth of the source claims.",
                group.len(),
                kinds.join(", ")
            );
            draft.classification.epistemic = EpistemicClass::Derived;
            let (object, revision) = form_memory(
                &mut tx,
                subject,
                &draft,
                &format!(
                    "abstraction:{}:{}",
                    input.operation_id,
                    result.created.len()
                ),
            )
            .await?;
            for parent in group {
                sqlx::query("INSERT INTO memory_revision_parents(subject_id,revision_id,parent_revision_id,relation) VALUES($1,$2,$3,'CONSOLIDATION')").bind(subject.0).bind(revision).bind(parent.revision_id).execute(&mut *tx).await.map_err(db)?;
            }
            result.created.push(object);
        }
        sqlx::query("INSERT INTO memory_operation_results(operation_id,subject_id,kind,request,state,result) VALUES($1,$2,'consolidate',$3,'completed',$4)")
            .bind(input.operation_id).bind(subject.0).bind(request).bind(serde_json::to_value(&result).map_err(|e|Error::Infrastructure(e.to_string()))?).execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)?;
        Ok(result)
    }

    pub async fn rebuild_projection(&self, subject: SubjectId) -> Result<usize> {
        let _objects = self.objects.reference_guard(true).await?;
        self.rebuild_projection_unlocked(subject).await
    }
    pub(crate) async fn rebuild_projection_unlocked(&self, subject: SubjectId) -> Result<usize> {
        let projection = self
            .projection
            .as_ref()
            .ok_or_else(|| Error::Unavailable("LanceDB unavailable".into()))?;
        projection.remove_subject(subject).await?;
        let mut after = None::<Uuid>;
        let mut count = 0;
        loop {
            let rows=sqlx::query("SELECT e.*,r.object_id FROM retained_embeddings e JOIN memory_revisions r USING(revision_id) WHERE e.subject_id=$1 AND ($2::uuid IS NULL OR e.embedding_id>$2) ORDER BY e.embedding_id LIMIT 256")
                .bind(subject.0).bind(after).fetch_all(self.store.pool()).await.map_err(db)?;
            if rows.is_empty() {
                break;
            }
            let mut groups: BTreeMap<String, Vec<nous_memory_retrieval::dense::VectorRow>> =
                BTreeMap::new();
            for row in rows {
                after = Some(row.try_get("embedding_id").map_err(db)?);
                groups
                    .entry(row.try_get("projection_table").map_err(db)?)
                    .or_default()
                    .push(nous_memory_retrieval::dense::VectorRow {
                        subject,
                        object: row.try_get("object_id").map_err(db)?,
                        revision: row.try_get("revision_id").map_err(db)?,
                        vector: row.try_get("vector").map_err(db)?,
                    });
                count += 1;
            }
            for (table, rows) in groups {
                projection.upsert(&table, &rows).await?;
            }
        }
        Ok(count)
    }
}
fn merge_support(group: &[MemoryView], mut draft: MemoryDraft) -> MemoryDraft {
    draft.source_refs = group
        .iter()
        .flat_map(|m| m.content.source_refs.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    draft.artifact_refs = group
        .iter()
        .flat_map(|m| m.content.artifact_refs.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    draft.derivation_refs = group
        .iter()
        .flat_map(|m| m.content.derivation_refs.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    draft.entities = group
        .iter()
        .flat_map(|m| m.content.entities.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    draft
}
