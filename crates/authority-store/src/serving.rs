use crate::{AuthorityStore, database_error as db};
use chrono::{DateTime, Utc};
use nous_core::*;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServingRecord {
    pub generation_id: ServingGenerationId,
    pub subject: SubjectId,
    pub family: String,
    pub space: String,
    pub watermark: i64,
    pub artifact_location: String,
    pub artifact_hash: String,
    pub built_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

fn decode(row: sqlx::postgres::PgRow) -> Result<ServingRecord> {
    Ok(ServingRecord {
        generation_id: ServingGenerationId(row.try_get("generation_id").map_err(db)?), subject: SubjectId(row.try_get("subject_id").map_err(db)?),
        family: row.try_get("kind").map_err(db)?, space: row.try_get::<Option<String>,_>("space_signature").map_err(db)?.unwrap_or_default(), watermark: row.try_get("input_generation").map_err(db)?,
        artifact_location: row.try_get("artifact_location").map_err(db)?, artifact_hash: row.try_get("artifact_hash").map_err(db)?, built_at: row.try_get("built_at").map_err(db)?, metadata: row.try_get("metadata").map_err(db)?,
    })
}

impl AuthorityStore {
    pub async fn serving_current(&self, subject: SubjectId) -> Result<Vec<ServingRecord>> {
        sqlx::query("SELECT g.* FROM serving_current c JOIN serving_generations g USING(generation_id) WHERE c.subject_id=$1 AND g.state='ready' ORDER BY c.kind,c.space_signature")
            .bind(subject.0).fetch_all(self.pool()).await.map_err(db)?.into_iter().map(decode).collect()
    }

    pub async fn publish_generation(&self, record: ServingRecord) -> Result<ServingRecord> {
        let mut tx = self.begin().await?;
        let key = format!("serving:{}:{}:{}", record.subject.0, record.family, record.space);
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))").bind(key).execute(&mut *tx).await.map_err(db)?;
        let current = sqlx::query("SELECT g.* FROM serving_current c JOIN serving_generations g USING(generation_id) WHERE c.subject_id=$1 AND c.kind=$2 AND c.space_signature=$3")
            .bind(record.subject.0).bind(&record.family).bind(&record.space).fetch_optional(&mut *tx).await.map_err(db)?.map(decode).transpose()?;
        if let Some(current) = current.as_ref().filter(|current| current.watermark > record.watermark) {
            return Ok(current.clone());
        }
        sqlx::query("INSERT INTO serving_generations(generation_id,subject_id,kind,space_signature,input_generation,artifact_location,artifact_hash,state,built_at,published_at,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,'ready',$8,$9,$10)")
            .bind(record.generation_id.0).bind(record.subject.0).bind(&record.family).bind(&record.space).bind(record.watermark).bind(&record.artifact_location).bind(&record.artifact_hash).bind(record.built_at).bind(Utc::now()).bind(&record.metadata)
            .execute(&mut *tx).await.map_err(db)?;
        sqlx::query("INSERT INTO serving_current(subject_id,kind,space_signature,generation_id) VALUES($1,$2,$3,$4) ON CONFLICT(subject_id,kind,space_signature) DO UPDATE SET generation_id=excluded.generation_id")
            .bind(record.subject.0).bind(&record.family).bind(&record.space).bind(record.generation_id.0).execute(&mut *tx).await.map_err(db)?;
        if let Some(current) = current {
            sqlx::query("UPDATE serving_generations SET state='retired' WHERE generation_id=$1").bind(current.generation_id.0).execute(&mut *tx).await.map_err(db)?;
        }
        tx.commit().await.map_err(db)?;
        Ok(record)
    }
}
