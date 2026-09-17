//! Versioned persistence for Host-owned runtime payloads.
use crate::*;
use nous_authority_store::database_error as db;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCheckpoint {
    pub owner_kind: String,
    pub owner_key: String,
    pub schema_version: i32,
    pub revision: i64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointWrite {
    pub owner_kind: String,
    pub owner_key: String,
    pub schema_version: i32,
    pub expected_revision: i64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMutation {
    pub subject: SubjectId,
    pub session: SessionId,
    pub expected_runtime_revision: i64,
    pub checkpoints: Vec<CheckpointWrite>,
    /// None preserves foreground; Some(None) clears it.
    pub foreground: Option<Option<String>>,
}

pub struct RuntimeSnapshot {
    pub revision: i64,
    pub active_focus_key: Option<String>,
    pub closed: bool,
    pub checkpoints: Vec<RuntimeCheckpoint>,
}

impl CognitiveRuntimeService {
    pub async fn runtime_snapshot(
        &self,
        subject: SubjectId,
        session: SessionId,
        owner: &str,
    ) -> Result<RuntimeSnapshot> {
        let rows=sqlx::query("SELECT s.state_revision,s.active_focus_key,s.closed_at,c.owner_kind,c.owner_key,c.schema_version,c.revision,c.payload FROM cognitive_sessions s LEFT JOIN runtime_checkpoints c ON c.session_id=s.session_id AND c.subject_id=s.subject_id AND c.owner_kind=$3 WHERE s.subject_id=$1 AND s.session_id=$2 ORDER BY c.owner_key LIMIT 257")
            .bind(subject.0).bind(session.0).bind(owner).fetch_all(self.store.pool()).await.map_err(db)?;
        let first = rows
            .first()
            .ok_or_else(|| Error::NotFound("Session not found".into()))?;
        if rows.len() > 256 {
            return Err(Error::Invalid("checkpoint capacity exceeded".into()));
        }
        let revision = first.try_get("state_revision").map_err(db)?;
        let active_focus_key = first.try_get("active_focus_key").map_err(db)?;
        let closed = first
            .try_get::<Option<DateTime<Utc>>, _>("closed_at")
            .map_err(db)?
            .is_some();
        let mut checkpoints = vec![];
        for row in rows {
            let Some(owner_key) = row.try_get::<Option<String>, _>("owner_key").map_err(db)? else {
                continue;
            };
            checkpoints.push(RuntimeCheckpoint {
                owner_kind: row.try_get("owner_kind").map_err(db)?,
                owner_key,
                schema_version: row.try_get("schema_version").map_err(db)?,
                revision: row.try_get("revision").map_err(db)?,
                payload: row.try_get("payload").map_err(db)?,
            });
        }
        Ok(RuntimeSnapshot {
            revision,
            active_focus_key,
            closed,
            checkpoints,
        })
    }

    pub async fn mutate_runtime(&self, input: RuntimeMutation) -> Result<i64> {
        validate_mutation(&input)?;
        let mut tx = self.store.begin().await?;
        let current: i64 = sqlx::query_scalar("SELECT state_revision FROM cognitive_sessions WHERE subject_id=$1 AND session_id=$2 AND closed_at IS NULL FOR UPDATE")
            .bind(input.subject.0).bind(input.session.0).fetch_one(&mut *tx).await.map_err(db)?;
        if current != input.expected_runtime_revision {
            return Err(Error::Conflict("stale Session runtime revision".into()));
        }
        for checkpoint in input.checkpoints {
            let updated = sqlx::query("INSERT INTO runtime_checkpoints(subject_id,session_id,owner_kind,owner_key,schema_version,revision,payload) SELECT $1,$2,$3,$4,$5,1,$6 WHERE $7=0 ON CONFLICT(session_id,owner_kind,owner_key) DO NOTHING")
                .bind(input.subject.0).bind(input.session.0).bind(&checkpoint.owner_kind).bind(&checkpoint.owner_key)
                .bind(checkpoint.schema_version).bind(&checkpoint.payload).bind(checkpoint.expected_revision)
                .execute(&mut *tx).await.map_err(db)?.rows_affected();
            if updated == 0 {
                let updated = sqlx::query("UPDATE runtime_checkpoints SET schema_version=$5,revision=revision+1,payload=$6,updated_at=now() WHERE subject_id=$1 AND session_id=$2 AND owner_kind=$3 AND owner_key=$4 AND revision=$7")
                    .bind(input.subject.0).bind(input.session.0).bind(&checkpoint.owner_kind).bind(&checkpoint.owner_key)
                    .bind(checkpoint.schema_version).bind(&checkpoint.payload).bind(checkpoint.expected_revision)
                    .execute(&mut *tx).await.map_err(db)?.rows_affected();
                if updated != 1 {
                    return Err(Error::Conflict("stale runtime checkpoint revision".into()));
                }
            }
        }
        if let Some(foreground) = input.foreground {
            if let Some(key) = &foreground {
                let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM runtime_checkpoints WHERE subject_id=$1 AND session_id=$2 AND owner_kind='focus' AND owner_key=$3)")
                    .bind(input.subject.0).bind(input.session.0).bind(key).fetch_one(&mut *tx).await.map_err(db)?;
                if !exists {
                    return Err(Error::Invalid(
                        "foreground Focus checkpoint does not exist".into(),
                    ));
                }
            }
            sqlx::query("UPDATE cognitive_sessions SET active_focus_key=$3 WHERE subject_id=$1 AND session_id=$2")
                .bind(input.subject.0).bind(input.session.0).bind(foreground).execute(&mut *tx).await.map_err(db)?;
        }
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM runtime_checkpoints WHERE session_id=$1")
                .bind(input.session.0)
                .fetch_one(&mut *tx)
                .await
                .map_err(db)?;
        if count > 256 {
            return Err(Error::Invalid(
                "Session checkpoint capacity exceeded".into(),
            ));
        }
        sqlx::query("UPDATE cognitive_sessions SET state_revision=state_revision+1,last_activity_at=now() WHERE session_id=$1")
            .bind(input.session.0).execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)?;
        Ok(current + 1)
    }
}

fn validate_mutation(input: &RuntimeMutation) -> Result<()> {
    if input.checkpoints.len() > 16 || input.expected_runtime_revision < 0 {
        return Err(Error::Invalid("invalid runtime mutation bounds".into()));
    }
    let mut keys = HashSet::new();
    for item in &input.checkpoints {
        if !matches!(item.owner_kind.as_str(), "focus" | "context" | "steward")
            || item.owner_key.is_empty()
            || item.owner_key.len() > 256
            || item.schema_version <= 0
            || item.expected_revision < 0
            || item.payload.len() > 1_048_576
            || !keys.insert((&item.owner_kind, &item.owner_key))
        {
            return Err(Error::Invalid(
                "invalid or duplicate runtime checkpoint".into(),
            ));
        }
    }
    Ok(())
}
