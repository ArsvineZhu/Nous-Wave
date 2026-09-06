use crate::{
    LocalRuntime,
    memory::lock_subject,
    subjects::{check_version, db},
};
use nous_core::{Error, Result, SubjectId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum PurgeTarget {
    Subject,
    Source(Uuid),
    Artifact(Uuid),
    Memory(Uuid),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeRequest {
    pub api_version: u32,
    pub operation_id: Uuid,
    pub target: PurgeTarget,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeResult {
    pub operation_id: Uuid,
    pub state: String,
    pub removed_sources: usize,
    pub removed_artifacts: usize,
    pub removed_revisions: usize,
    pub removed_objects: usize,
    pub regenerating_objects: Vec<Uuid>,
    pub last_cleanup_deleted_hashes: usize,
}

impl LocalRuntime {
    pub async fn purge(&self, subject: SubjectId, input: PurgeRequest) -> Result<PurgeResult> {
        self.require_memory(subject).await?;
        check_version(input.api_version)?;
        let _objects = self.objects.reference_guard(true).await?;
        let request = serde_json::to_value(&input).map_err(|e| Error::Invalid(e.to_string()))?;
        let mut result = if let Some(row) =
            sqlx::query("SELECT * FROM memory_operation_results WHERE operation_id=$1")
                .bind(input.operation_id)
                .fetch_optional(self.store.pool())
                .await
                .map_err(db)?
        {
            if row.try_get::<Uuid, _>("subject_id").map_err(db)? != subject.0
                || row.try_get::<String, _>("kind").map_err(db)? != "purge"
                || row.try_get::<serde_json::Value, _>("request").map_err(db)? != request
            {
                return Err(Error::Conflict(
                    "operation identity reused with a different request".into(),
                ));
            }
            let prior: PurgeResult = serde_json::from_value(row.try_get("result").map_err(db)?)
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
            if prior.state == "completed" {
                return Ok(prior);
            }
            prior
        } else {
            self.purge_canonical(subject, &input, request).await?
        };
        self.cycles
            .lock()
            .await
            .retain(|_, state| state.subject != Some(subject));
        if self.projection.is_some() {
            self.rebuild_projection_unlocked(subject).await?;
        } else {
            // A configured but failed projection must be repaired before physical purge can finish.
            if self.projection_failed() {
                return Err(Error::Unavailable(
                    "projection must be available to complete purge".into(),
                ));
            }
        }
        result.last_cleanup_deleted_hashes = self.remove_unreferenced_bytes().await?;
        result.state = "completed".into();
        sqlx::query("UPDATE memory_operation_results SET state='completed',result=$2,updated_at=clock_timestamp() WHERE operation_id=$1")
            .bind(input.operation_id).bind(serde_json::to_value(&result).map_err(|e|Error::Infrastructure(e.to_string()))?).execute(self.store.pool()).await.map_err(db)?;
        Ok(result)
    }

    pub async fn resume_purges(&self, limit: usize) -> Result<usize> {
        let rows=sqlx::query("SELECT subject_id,request FROM memory_operation_results WHERE kind='purge' AND state='physical_pending' ORDER BY created_at LIMIT $1")
            .bind(limit.min(100) as i64).fetch_all(self.store.pool()).await.map_err(db)?;
        let count = rows.len();
        for row in rows {
            let request = serde_json::from_value(row.try_get("request").map_err(db)?)
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
            self.purge(SubjectId(row.try_get("subject_id").map_err(db)?), request)
                .await?;
        }
        Ok(count)
    }

    async fn purge_canonical(
        &self,
        subject: SubjectId,
        input: &PurgeRequest,
        request: serde_json::Value,
    ) -> Result<PurgeResult> {
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        lock_subject(&mut tx, subject).await?;
        let whole = matches!(input.target, PurgeTarget::Subject);
        let sources: Vec<Uuid> = match input.target {
            PurgeTarget::Subject => {
                sqlx::query_scalar("SELECT source_id FROM source_records WHERE subject_id=$1")
                    .bind(subject.0)
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(db)?
            }
            PurgeTarget::Source(id) => {
                sqlx::query(
                    "SELECT source_id FROM source_records WHERE subject_id=$1 AND source_id=$2",
                )
                .bind(subject.0)
                .bind(id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(db)?
                .ok_or_else(|| Error::NotFound("purge source not found".into()))?;
                vec![id]
            }
            _ => vec![],
        };
        let initial_artifacts:Vec<Uuid>=match input.target {
            PurgeTarget::Subject=>sqlx::query_scalar("SELECT artifact_id FROM artifacts WHERE subject_id=$1").bind(subject.0).fetch_all(&mut *tx).await.map_err(db)?,
            PurgeTarget::Artifact(id)=>{sqlx::query("SELECT artifact_id FROM artifacts WHERE subject_id=$1 AND artifact_id=$2").bind(subject.0).bind(id).fetch_optional(&mut *tx).await.map_err(db)?.ok_or_else(||Error::NotFound("purge artifact not found".into()))?;vec![id]},
            PurgeTarget::Source(_)=>sqlx::query_scalar("SELECT DISTINCT a.artifact_id FROM artifact_sources a WHERE a.subject_id=$1 AND a.source_id=ANY($2) AND NOT EXISTS(SELECT 1 FROM artifact_sources retained WHERE retained.artifact_id=a.artifact_id AND NOT(retained.source_id=ANY($2)))")
                .bind(subject.0).bind(&sources).fetch_all(&mut *tx).await.map_err(db)?,
            PurgeTarget::Memory(id)=>{sqlx::query("SELECT object_id FROM memory_objects WHERE subject_id=$1 AND object_id=$2").bind(subject.0).bind(id).fetch_optional(&mut *tx).await.map_err(db)?.ok_or_else(||Error::NotFound("purge memory not found".into()))?;vec![]},
        };
        let artifacts:Vec<Uuid>=sqlx::query_scalar("WITH RECURSIVE affected(id) AS (SELECT unnest($2::uuid[]) UNION SELECT o.artifact_id FROM affected a JOIN derivation_inputs i ON i.artifact_id=a.id JOIN derivation_outputs o USING(derivation_id)) SELECT DISTINCT id FROM affected JOIN artifacts ON artifact_id=id WHERE subject_id=$1")
            .bind(subject.0).bind(&initial_artifacts).fetch_all(&mut *tx).await.map_err(db)?;
        if !whole {
            let seed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM character_seed_revisions WHERE subject_id=$1 AND artifact_id=ANY($2))").bind(subject.0).bind(&artifacts).fetch_one(&mut *tx).await.map_err(db)?;
            if seed {
                return Err(Error::Conflict("Character Seed belongs to Subject Core; use subject purge for its retained history".into()));
            }
        }
        let derivations:Vec<Uuid>=sqlx::query_scalar("SELECT DISTINCT d.derivation_id FROM derivations d LEFT JOIN derivation_inputs i USING(derivation_id) LEFT JOIN derivation_outputs o USING(derivation_id) WHERE d.subject_id=$1 AND ($2 OR i.artifact_id=ANY($3) OR o.artifact_id=ANY($3))")
            .bind(subject.0).bind(whole).bind(&artifacts).fetch_all(&mut *tx).await.map_err(db)?;
        let direct = if let PurgeTarget::Memory(id) = input.target {
            Some(id)
        } else {
            None
        };
        let revisions:Vec<Uuid>=sqlx::query_scalar("WITH RECURSIVE affected(id) AS (SELECT r.revision_id FROM memory_revisions r WHERE r.subject_id=$1 AND ($2 OR r.object_id=$3 OR EXISTS(SELECT 1 FROM memory_revision_sources s WHERE s.revision_id=r.revision_id AND s.source_id=ANY($4)) OR EXISTS(SELECT 1 FROM memory_revision_artifacts a WHERE a.revision_id=r.revision_id AND a.artifact_id=ANY($5)) OR EXISTS(SELECT 1 FROM memory_revision_derivations d WHERE d.revision_id=r.revision_id AND d.derivation_id=ANY($6))) UNION SELECT p.revision_id FROM affected a JOIN memory_revision_parents p ON p.parent_revision_id=a.id) SELECT DISTINCT id FROM affected")
            .bind(subject.0).bind(whole).bind(direct).bind(&sources).bind(&artifacts).bind(&derivations).fetch_all(&mut *tx).await.map_err(db)?;
        let affected_heads=sqlx::query("SELECT o.object_id,o.object_kind,o.scope,o.formation_class,o.formation_metadata,h.revision_id FROM memory_objects o JOIN memory_current_heads h USING(object_id) WHERE o.subject_id=$1 AND h.revision_id=ANY($2)")
            .bind(subject.0).bind(&revisions).fetch_all(&mut *tx).await.map_err(db)?;
        let mut deleted = BTreeSet::<Uuid>::new();
        let mut regenerating = vec![];
        for row in affected_heads {
            let object: Uuid = row.try_get("object_id").map_err(db)?;
            let remaining:Vec<Uuid>=sqlx::query_scalar("SELECT source_id FROM memory_revision_sources WHERE revision_id=$1 AND NOT(source_id=ANY($2)) ORDER BY source_id")
                .bind(row.try_get::<Uuid,_>("revision_id").map_err(db)?).bind(&sources).fetch_all(&mut *tx).await.map_err(db)?;
            if whole || direct == Some(object) || remaining.is_empty() {
                deleted.insert(object);
            } else {
                regenerating.push(object);
                let formation_class: String = row.try_get("formation_class").map_err(db)?;
                let formation_metadata: serde_json::Value =
                    row.try_get("formation_metadata").map_err(db)?;
                sqlx::query(
                    "UPDATE memory_objects SET availability='regenerating' WHERE object_id=$1",
                )
                .bind(object)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
                sqlx::query("INSERT INTO processing_obligations(obligation_id,subject_id,kind,payload_version,payload) VALUES($1,$2,'regenerate',1,$3)")
                    .bind(Uuid::now_v7()).bind(subject.0).bind(serde_json::json!({"object_id":object,"source_refs":remaining,"kind":row.try_get::<String,_>("object_kind").map_err(db)?,"scope":row.try_get::<String,_>("scope").map_err(db)?,"formation_class":formation_class,"formation_metadata":formation_metadata})).execute(&mut *tx).await.map_err(db)?;
            }
        }
        if whole {
            deleted.extend(
                sqlx::query_scalar::<_, Uuid>(
                    "SELECT object_id FROM memory_objects WHERE subject_id=$1",
                )
                .bind(subject.0)
                .fetch_all(&mut *tx)
                .await
                .map_err(db)?,
            );
        }
        let deleted: Vec<Uuid> = deleted.into_iter().collect();
        let all_affected_objects: Vec<Uuid> = sqlx::query_scalar(
            "SELECT DISTINCT object_id FROM memory_revisions WHERE revision_id=ANY($1)",
        )
        .bind(&revisions)
        .fetch_all(&mut *tx)
        .await
        .map_err(db)?;
        sqlx::query("DELETE FROM association_evidence WHERE subject_id=$1 AND ($2 OR source_id=ANY($3) OR (from_kind='memory' AND from_id=ANY($4)) OR (to_kind='memory' AND to_id=ANY($4)) OR from_id=ANY($5) OR to_id=ANY($5) OR use_event_id IN (SELECT event_id FROM cognitive_use_events WHERE subject_id=$1 AND object_refs && $6))")
            .bind(subject.0).bind(whole).bind(&sources).bind(&deleted).bind(&artifacts).bind(&all_affected_objects).execute(&mut *tx).await.map_err(db)?;
        sqlx::query(
            "DELETE FROM cognitive_use_events WHERE subject_id=$1 AND ($2 OR object_refs && $3)",
        )
        .bind(subject.0)
        .bind(whole)
        .bind(&all_affected_objects)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        sqlx::query("DELETE FROM episode_members WHERE subject_id=$1 AND ($2 OR episode_id=ANY($3) OR member_id=ANY($3))").bind(subject.0).bind(whole).bind(&deleted).execute(&mut *tx).await.map_err(db)?;
        sqlx::query("DELETE FROM processing_obligations WHERE subject_id=$1 AND ($2 OR source_id=ANY($3) OR payload->>'revision_id'=ANY($4::text[]) OR kind='projection' OR (kind<>'regenerate' AND payload->>'object_id'=ANY($5::text[])))")
            .bind(subject.0).bind(whole).bind(&sources).bind(revisions.iter().map(Uuid::to_string).collect::<Vec<_>>()).bind(deleted.iter().map(Uuid::to_string).collect::<Vec<_>>()).execute(&mut *tx).await.map_err(db)?;
        delete_revisions(&mut tx, &revisions).await?;
        sqlx::query("UPDATE memory_objects SET superseded_by=NULL WHERE superseded_by=ANY($1)")
            .bind(&deleted)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        for query in [
            "DELETE FROM suppression_state WHERE object_id=ANY($1)",
            "DELETE FROM accessibility_state WHERE object_id=ANY($1)",
        ] {
            sqlx::query(query)
                .bind(&deleted)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        sqlx::query("DELETE FROM memory_objects WHERE object_id=ANY($1)")
            .bind(&deleted)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("UPDATE accessibility_state a SET meaningful_uses=0,last_meaningful_use=NULL FROM memory_objects o WHERE o.object_id=a.object_id AND o.subject_id=$1").bind(subject.0).execute(&mut *tx).await.map_err(db)?;
        sqlx::query("WITH uses AS (SELECT unnest(object_refs) AS object_id,count(*) OVER () FROM cognitive_use_events WHERE subject_id=$1 AND kind IN ('FOLLOWED','REFERENCED_OR_ACTED_ON')), aggregate_use AS (SELECT object_id,count(*) AS n FROM uses GROUP BY object_id) UPDATE accessibility_state a SET meaningful_uses=u.n FROM aggregate_use u WHERE a.object_id=u.object_id").bind(subject.0).execute(&mut *tx).await.map_err(db)?;
        for query in [
            "DELETE FROM derivation_inputs WHERE derivation_id=ANY($1)",
            "DELETE FROM derivation_outputs WHERE derivation_id=ANY($1)",
        ] {
            sqlx::query(query)
                .bind(&derivations)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        sqlx::query("DELETE FROM derivations WHERE derivation_id=ANY($1)")
            .bind(&derivations)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query(
            "DELETE FROM document_sections WHERE artifact_id=ANY($1) OR source_artifact=ANY($1)",
        )
        .bind(&artifacts)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        sqlx::query("DELETE FROM artifact_sources WHERE subject_id=$1 AND (source_id=ANY($2) OR artifact_id=ANY($3))").bind(subject.0).bind(&sources).bind(&artifacts).execute(&mut *tx).await.map_err(db)?;
        if whole {
            sqlx::query("DELETE FROM character_seed_heads WHERE subject_id=$1")
                .bind(subject.0)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            sqlx::query("DELETE FROM character_seed_revisions WHERE subject_id=$1")
                .bind(subject.0)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        sqlx::query("DELETE FROM artifacts WHERE artifact_id=ANY($1)")
            .bind(&artifacts)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query(
            "DELETE FROM source_parents WHERE source_id=ANY($1) OR parent_source_id=ANY($1)",
        )
        .bind(&sources)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        sqlx::query("DELETE FROM source_records WHERE source_id=ANY($1)")
            .bind(&sources)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        if whole {
            sqlx::query("DELETE FROM cognitive_cycles WHERE subject_id=$1")
                .bind(subject.0)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            sqlx::query("DELETE FROM memory_entities WHERE subject_id=$1")
                .bind(subject.0)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            sqlx::query("DELETE FROM memory_operation_results WHERE subject_id=$1")
                .bind(subject.0)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            sqlx::query("DELETE FROM subjects WHERE subject_id=$1")
                .bind(subject.0)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        let result = PurgeResult {
            operation_id: input.operation_id,
            state: "physical_pending".into(),
            removed_sources: sources.len(),
            removed_artifacts: artifacts.len(),
            removed_revisions: revisions.len(),
            removed_objects: deleted.len(),
            regenerating_objects: regenerating,
            last_cleanup_deleted_hashes: 0,
        };
        sqlx::query("INSERT INTO memory_operation_results(operation_id,subject_id,kind,request,state,result) VALUES($1,$2,'purge',$3,'physical_pending',$4)").bind(input.operation_id).bind(subject.0).bind(request).bind(serde_json::to_value(&result).map_err(|e|Error::Infrastructure(e.to_string()))?).execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)?;
        Ok(result)
    }

    async fn remove_unreferenced_bytes(&self) -> Result<usize> {
        use futures::StreamExt;
        let stream = self.objects.hashes().await?;
        futures::pin_mut!(stream);
        let mut deleted = 0;
        loop {
            let mut batch = vec![];
            for _ in 0..256 {
                if let Some(hash) = stream.next().await {
                    batch.push(hash?);
                } else {
                    break;
                }
            }
            if batch.is_empty() {
                break;
            }
            let retained: Vec<String> = sqlx::query_scalar(
                "SELECT DISTINCT content_hash FROM artifacts WHERE content_hash=ANY($1)",
            )
            .bind(&batch)
            .fetch_all(self.store.pool())
            .await
            .map_err(db)?;
            let retained: BTreeSet<_> = retained.into_iter().collect();
            for hash in batch {
                if !retained.contains(&hash) {
                    self.objects.delete_unreferenced(&hash).await?;
                    deleted += 1;
                }
            }
        }
        Ok(deleted)
    }
}
async fn delete_revisions(tx: &mut Transaction<'_, Postgres>, revisions: &[Uuid]) -> Result<()> {
    sqlx::query("DELETE FROM memory_revision_parents WHERE revision_id=ANY($1) OR parent_revision_id=ANY($1)").bind(revisions).execute(&mut **tx).await.map_err(db)?;
    for query in [
        "DELETE FROM memory_current_heads WHERE revision_id=ANY($1)",
        "DELETE FROM memory_revision_sources WHERE revision_id=ANY($1)",
        "DELETE FROM memory_revision_artifacts WHERE revision_id=ANY($1)",
        "DELETE FROM memory_revision_derivations WHERE revision_id=ANY($1)",
        "DELETE FROM memory_entity_mentions WHERE revision_id=ANY($1)",
        "DELETE FROM retained_embeddings WHERE revision_id=ANY($1)",
    ] {
        sqlx::query(query)
            .bind(revisions)
            .execute(&mut **tx)
            .await
            .map_err(db)?;
    }
    sqlx::query("DELETE FROM memory_revisions WHERE revision_id=ANY($1)")
        .bind(revisions)
        .execute(&mut **tx)
        .await
        .map_err(db)?;
    Ok(())
}
