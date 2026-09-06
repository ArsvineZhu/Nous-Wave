use crate::{
    LocalRuntime,
    material::{decode_enum, enum_name},
    subjects::{check_version, db},
};
use chrono::{DateTime, Utc};
use nous_core::{Error, Result, SubjectId};
use nous_material::{Classification, TemporalRange};
use nous_memory_domain::{MemoryKind, RevisionRelation};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDraft {
    pub kind: MemoryKind,
    pub scope: String,
    pub title: String,
    pub text: String,
    pub classification: Classification,
    pub occurred: Option<TemporalRange>,
    pub observed_at: Option<DateTime<Utc>>,
    pub source_refs: Vec<Uuid>,
    pub artifact_refs: Vec<Uuid>,
    #[serde(default)]
    pub derivation_refs: Vec<Uuid>,
    #[serde(default)]
    pub entities: Vec<Uuid>,
}
impl MemoryDraft {
    pub fn validate(&self) -> Result<()> {
        self.classification.validate()?;
        if let Some(time) = &self.occurred {
            time.validate()?;
        }
        if self.scope.is_empty()
            || self.scope.len() > 1024
            || self.title.is_empty()
            || self.title.len() > 4096
            || self.text.len() > 262144
            || self.source_refs.is_empty()
            || self.source_refs.len() > 1000
            || self.artifact_refs.len() > 1000
            || self.derivation_refs.len() > 1000
            || self.entities.len() > 1000
        {
            return Err(Error::Invalid(
                "memory representation or evidence bounds are invalid".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryView {
    pub object_id: Uuid,
    pub revision_id: Uuid,
    pub subject_id: SubjectId,
    pub content: MemoryDraft,
    pub recorded_at: DateTime<Utc>,
    pub parent_revisions: Vec<Uuid>,
    pub reason: String,
    pub suppressed: bool,
    pub superseded_by: Option<Uuid>,
    pub availability: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectMemory {
    pub api_version: u32,
    pub expected_revision: Uuid,
    pub replacement: MemoryDraft,
    pub reason: String,
    pub authority_metadata: Option<serde_json::Value>,
    pub relation: RevisionRelation,
}

impl LocalRuntime {
    pub async fn memory(
        &self,
        subject: SubjectId,
        object: Uuid,
        revision: Option<Uuid>,
    ) -> Result<MemoryView> {
        let row = sqlx::query("SELECT o.object_kind,o.scope,o.availability,o.superseded_by,r.*,COALESCE(s.suppressed,false) AS suppressed,lower(r.occurred) AS occurred_start,upper(r.occurred) AS occurred_end,r.occurred IS NOT NULL AS has_occurred FROM memory_objects o JOIN memory_revisions r ON r.object_id=o.object_id LEFT JOIN memory_current_heads h ON h.object_id=o.object_id LEFT JOIN suppression_state s ON s.object_id=o.object_id WHERE o.subject_id=$1 AND o.object_id=$2 AND r.revision_id=COALESCE($3,h.revision_id)")
            .bind(subject.0).bind(object).bind(revision).fetch_optional(self.store.pool()).await.map_err(db)?
            .ok_or_else(|| Error::NotFound("memory revision not found or unavailable during regeneration".into()))?;
        let revision: Uuid = row.try_get("revision_id").map_err(db)?;
        let source_refs = sqlx::query_scalar(
            "SELECT source_id FROM memory_revision_sources WHERE revision_id=$1 ORDER BY source_id",
        )
        .bind(revision)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?;
        let artifact_refs = sqlx::query_scalar("SELECT artifact_id FROM memory_revision_artifacts WHERE revision_id=$1 ORDER BY artifact_id").bind(revision).fetch_all(self.store.pool()).await.map_err(db)?;
        let derivation_refs = sqlx::query_scalar("SELECT derivation_id FROM memory_revision_derivations WHERE revision_id=$1 ORDER BY derivation_id").bind(revision).fetch_all(self.store.pool()).await.map_err(db)?;
        let entities = sqlx::query_scalar(
            "SELECT entity_id FROM memory_entity_mentions WHERE revision_id=$1 ORDER BY entity_id",
        )
        .bind(revision)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?;
        let parents = sqlx::query_scalar("SELECT parent_revision_id FROM memory_revision_parents WHERE revision_id=$1 ORDER BY parent_revision_id").bind(revision).fetch_all(self.store.pool()).await.map_err(db)?;
        Ok(MemoryView {
            object_id: object,
            revision_id: revision,
            subject_id: subject,
            content: MemoryDraft {
                kind: decode_enum(row.try_get("object_kind").map_err(db)?)?,
                scope: row.try_get("scope").map_err(db)?,
                title: row.try_get("title").map_err(db)?,
                text: row.try_get("representation_text").map_err(db)?,
                classification: Classification {
                    origin: decode_enum(row.try_get("origin_class").map_err(db)?)?,
                    semantic: row.try_get("semantic_class").map_err(db)?,
                    epistemic: decode_enum(row.try_get("epistemic_class").map_err(db)?)?,
                },
                occurred: if row.try_get::<bool, _>("has_occurred").map_err(db)? {
                    Some(TemporalRange {
                        start: row.try_get("occurred_start").map_err(db)?,
                        end: row.try_get("occurred_end").map_err(db)?,
                        approximate: row.try_get("approximate_time").map_err(db)?,
                    })
                } else {
                    None
                },
                observed_at: row.try_get("observed_at").map_err(db)?,
                source_refs,
                artifact_refs,
                derivation_refs,
                entities,
            },
            recorded_at: row.try_get("recorded_at").map_err(db)?,
            parent_revisions: parents,
            reason: row.try_get("reason").map_err(db)?,
            suppressed: row.try_get("suppressed").map_err(db)?,
            superseded_by: row.try_get("superseded_by").map_err(db)?,
            availability: row.try_get("availability").map_err(db)?,
        })
    }

    pub async fn memory_history(
        &self,
        subject: SubjectId,
        object: Uuid,
    ) -> Result<Vec<MemoryView>> {
        let revisions: Vec<Uuid> = sqlx::query_scalar("SELECT revision_id FROM memory_revisions WHERE subject_id=$1 AND object_id=$2 ORDER BY recorded_at,revision_id")
            .bind(subject.0).bind(object).fetch_all(self.store.pool()).await.map_err(db)?;
        if revisions.is_empty() {
            return Err(Error::NotFound("memory not found".into()));
        }
        let mut history = vec![];
        for revision in revisions {
            history.push(self.memory(subject, object, Some(revision)).await?);
        }
        Ok(history)
    }

    pub async fn correct_memory(
        &self,
        subject: SubjectId,
        object: Uuid,
        input: CorrectMemory,
    ) -> Result<MemoryView> {
        check_version(input.api_version)?;
        input.replacement.validate()?;
        if input.reason.trim().is_empty()
            || input.reason.len() > 4096
            || input
                .authority_metadata
                .as_ref()
                .is_some_and(|v| v.to_string().len() > 65536)
            || input.relation == RevisionRelation::Consolidation
        {
            return Err(Error::Invalid("correction requires bounded reason/authority and correction or world evolution semantics".into()));
        }
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        lock_subject(&mut tx, subject).await?;
        let current: Option<Uuid> = sqlx::query_scalar("SELECT h.revision_id FROM memory_objects o JOIN memory_current_heads h USING(object_id) WHERE o.subject_id=$1 AND o.object_id=$2 FOR UPDATE OF o")
            .bind(subject.0).bind(object).fetch_optional(&mut *tx).await.map_err(db)?;
        if current != Some(input.expected_revision) {
            return Err(Error::Conflict("memory head changed or was removed".into()));
        }
        let previous =
            sqlx::query("SELECT object_kind,scope FROM memory_objects WHERE object_id=$1")
                .bind(object)
                .fetch_one(&mut *tx)
                .await
                .map_err(db)?;
        if previous.try_get::<String, _>("object_kind").map_err(db)?
            != enum_name(&input.replacement.kind)?
            || previous.try_get::<String, _>("scope").map_err(db)? != input.replacement.scope
        {
            return Err(Error::Invalid(
                "correction cannot change logical object kind or scope".into(),
            ));
        }
        let revision = write_revision(
            &mut tx,
            subject,
            object,
            &input.replacement,
            &input.reason,
            input.authority_metadata.as_ref(),
        )
        .await?;
        sqlx::query("INSERT INTO memory_revision_parents(subject_id,revision_id,parent_revision_id,relation) VALUES($1,$2,$3,$4)")
            .bind(subject.0).bind(revision).bind(input.expected_revision).bind(enum_name(&input.relation)?).execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)?;
        self.memory(subject, object, None).await
    }

    pub async fn suppress(
        &self,
        subject: SubjectId,
        object: Uuid,
        suppressed: bool,
        reason: &str,
    ) -> Result<()> {
        if reason.is_empty() || reason.len() > 4096 {
            return Err(Error::Invalid("suppression needs a bounded reason".into()));
        }
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        lock_subject(&mut tx, subject).await?;
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM memory_objects WHERE subject_id=$1 AND object_id=$2)",
        )
        .bind(subject.0)
        .bind(object)
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
        if !exists {
            return Err(Error::NotFound("memory not found".into()));
        }
        sqlx::query("INSERT INTO suppression_state(object_id,suppressed,reason) VALUES($1,$2,$3) ON CONFLICT(object_id) DO UPDATE SET suppressed=excluded.suppressed,reason=excluded.reason,changed_at=clock_timestamp()")
            .bind(object).bind(suppressed).bind(reason).execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)
    }

    pub(crate) async fn require_memory(&self, subject: SubjectId) -> Result<()> {
        if !self.subject(subject).await?.memory_enabled {
            return Err(Error::Unavailable("memory disabled for subject".into()));
        }
        Ok(())
    }
}

pub(crate) async fn lock_subject(
    tx: &mut Transaction<'_, Postgres>,
    subject: SubjectId,
) -> Result<()> {
    sqlx::query("SELECT subject_id FROM subjects WHERE subject_id=$1 FOR UPDATE")
        .bind(subject.0)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db)?
        .ok_or_else(|| Error::NotFound("subject not found".into()))?;
    Ok(())
}

pub(crate) async fn form_memory(
    tx: &mut Transaction<'_, Postgres>,
    subject: SubjectId,
    draft: &MemoryDraft,
    formation_key: &str,
) -> Result<(Uuid, Uuid)> {
    draft.validate()?;
    if let Some(object) = sqlx::query_scalar::<_, Uuid>(
        "SELECT object_id FROM memory_objects WHERE subject_id=$1 AND formation_key=$2",
    )
    .bind(subject.0)
    .bind(formation_key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db)?
    {
        let head: Uuid =
            sqlx::query_scalar("SELECT revision_id FROM memory_current_heads WHERE object_id=$1")
                .bind(object)
                .fetch_one(&mut **tx)
                .await
                .map_err(db)?;
        return Ok((object, head));
    }
    let object = Uuid::now_v7();
    sqlx::query("INSERT INTO memory_objects(object_id,subject_id,object_kind,scope,formation_key) VALUES($1,$2,$3,$4,$5)")
        .bind(object).bind(subject.0).bind(enum_name(&draft.kind)?).bind(&draft.scope).bind(formation_key).execute(&mut **tx).await.map_err(db)?;
    sqlx::query("INSERT INTO accessibility_state(object_id) VALUES($1)")
        .bind(object)
        .execute(&mut **tx)
        .await
        .map_err(db)?;
    let revision = write_revision(tx, subject, object, draft, "memory formation", None).await?;
    Ok((object, revision))
}

pub(crate) async fn write_revision(
    tx: &mut Transaction<'_, Postgres>,
    subject: SubjectId,
    object: Uuid,
    draft: &MemoryDraft,
    reason: &str,
    authority: Option<&serde_json::Value>,
) -> Result<Uuid> {
    draft.validate()?;
    let revision = Uuid::now_v7();
    let digest =
        blake3::hash(&serde_json::to_vec(draft).map_err(|e| Error::Invalid(e.to_string()))?)
            .to_hex()
            .to_string();
    sqlx::query("INSERT INTO memory_revisions(revision_id,subject_id,object_id,title,representation_text,origin_class,semantic_class,epistemic_class,occurred,approximate_time,observed_at,content_digest,reason,authority_metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8,CASE WHEN $9 THEN tstzrange($10,$11,'[]') ELSE NULL END,$12,$13,$14,$15,$16)")
        .bind(revision).bind(subject.0).bind(object).bind(&draft.title).bind(&draft.text).bind(enum_name(&draft.classification.origin)?).bind(&draft.classification.semantic).bind(enum_name(&draft.classification.epistemic)?)
        .bind(draft.occurred.is_some()).bind(draft.occurred.as_ref().and_then(|r| r.start)).bind(draft.occurred.as_ref().and_then(|r| r.end)).bind(draft.occurred.as_ref().is_some_and(|r| r.approximate))
        .bind(draft.observed_at).bind(digest).bind(reason).bind(authority).execute(&mut **tx).await.map_err(db)?;
    for source in &draft.source_refs {
        sqlx::query("INSERT INTO memory_revision_sources(subject_id,revision_id,source_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(subject.0).bind(revision).bind(source).execute(&mut **tx).await.map_err(db)?;
    }
    for artifact in &draft.artifact_refs {
        sqlx::query("INSERT INTO memory_revision_artifacts(subject_id,revision_id,artifact_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(subject.0).bind(revision).bind(artifact).execute(&mut **tx).await.map_err(db)?;
    }
    for derivation in &draft.derivation_refs {
        sqlx::query("INSERT INTO memory_revision_derivations(subject_id,revision_id,derivation_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(subject.0).bind(revision).bind(derivation).execute(&mut **tx).await.map_err(db)?;
    }
    for entity in &draft.entities {
        sqlx::query("INSERT INTO memory_entity_mentions(subject_id,entity_id,revision_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(subject.0).bind(entity).bind(revision).execute(&mut **tx).await.map_err(db)?;
    }
    sqlx::query("INSERT INTO memory_current_heads(object_id,revision_id) VALUES($1,$2) ON CONFLICT(object_id) DO UPDATE SET revision_id=excluded.revision_id").bind(object).bind(revision).execute(&mut **tx).await.map_err(db)?;
    sqlx::query("INSERT INTO processing_obligations(obligation_id,subject_id,kind,payload_version,payload) VALUES($1,$2,'embedding',1,$3)")
        .bind(Uuid::now_v7()).bind(subject.0).bind(serde_json::json!({"object_id":object,"revision_id":revision})).execute(&mut **tx).await.map_err(db)?;
    Ok(revision)
}
