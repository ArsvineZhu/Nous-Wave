use chrono::{DateTime, Utc};
use nous_authority_store::database_error as db;
use nous_core::{ArtifactId, Error, Result, SubjectId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::SubjectCoreService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSeedInput {
    pub text: String,
    #[serde(default = "text_media_type")]
    pub media_type: String,
    #[serde(default)]
    pub provenance: serde_json::Value,
}

fn text_media_type() -> String {
    "text/plain; charset=utf-8".into()
}

impl CharacterSeedInput {
    pub fn validate(&self) -> Result<()> {
        if self.text.trim().is_empty() || self.media_type.trim().is_empty() {
            return Err(Error::Invalid(
                "Character Seed text and media type are required".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSeedView {
    pub subject: SubjectId,
    pub revision_id: Uuid,
    pub revision_no: i32,
    pub artifact_id: ArtifactId,
    pub text: String,
    pub media_type: String,
    pub provenance: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

pub(super) async fn insert_seed(
    tx: &mut Transaction<'_, Postgres>,
    subject: SubjectId,
    input: CharacterSeedInput,
    hash: String,
    revision_no: i32,
) -> Result<Uuid> {
    let artifact = sqlx::query_scalar::<_, Uuid>("INSERT INTO artifacts(artifact_id,subject_id,content_hash,byte_length,media_type,storage_key,created_at,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,'{}') ON CONFLICT(subject_id,content_hash) DO UPDATE SET content_hash=excluded.content_hash RETURNING artifact_id")
        .bind(Uuid::now_v7()).bind(subject.0).bind(&hash).bind(input.text.len() as i64)
        .bind(&input.media_type).bind(format!("{}/{}", &hash[..2], hash)).bind(Utc::now())
        .fetch_one(&mut **tx).await.map_err(db)?;
    let revision = Uuid::now_v7();
    sqlx::query("INSERT INTO character_seeds(seed_revision_id,subject_id,revision_no,artifact_id,media_type,provenance,created_at) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(revision).bind(subject.0).bind(revision_no).bind(artifact)
        .bind(input.media_type).bind(input.provenance).bind(Utc::now())
        .execute(&mut **tx).await.map_err(db)?;
    Ok(revision)
}

impl SubjectCoreService {
    pub async fn revise_character_seed(
        &self,
        subject: SubjectId,
        input: CharacterSeedInput,
    ) -> Result<CharacterSeedView> {
        input.validate()?;
        self.subject(subject).await?;
        let guard = self.objects.reference_guard(false).await?;
        let hash = self.objects.put(input.text.as_bytes().to_vec()).await?;
        let mut tx = self.store.begin().await?;
        sqlx::query("SELECT subject_id FROM subjects WHERE subject_id=$1 FOR UPDATE")
            .bind(subject.0)
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?;
        let next: i32 = sqlx::query_scalar(
            "SELECT COALESCE(max(revision_no),0)+1 FROM character_seeds WHERE subject_id=$1",
        )
        .bind(subject.0)
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
        let revision = insert_seed(&mut tx, subject, input, hash, next).await?;
        sqlx::query("UPDATE subjects SET state_revision=state_revision+1 WHERE subject_id=$1")
            .bind(subject.0)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        tx.commit().await.map_err(db)?;
        drop(guard);
        self.character_seed(subject, Some(revision)).await
    }

    pub async fn character_seed(
        &self,
        subject: SubjectId,
        revision: Option<Uuid>,
    ) -> Result<CharacterSeedView> {
        let row = sqlx::query("SELECT s.seed_revision_id,s.revision_no,s.artifact_id,s.media_type,s.provenance,s.created_at,a.content_hash FROM character_seeds s JOIN artifacts a USING(artifact_id) WHERE s.subject_id=$1 AND ($2::uuid IS NULL OR s.seed_revision_id=$2) ORDER BY s.revision_no DESC LIMIT 1")
            .bind(subject.0).bind(revision).fetch_optional(self.store.pool()).await.map_err(db)?
            .ok_or_else(|| Error::NotFound("Character Seed revision not found".into()))?;
        let hash: String = row.try_get("content_hash").map_err(db)?;
        let text = String::from_utf8(self.objects.get(&hash).await?)
            .map_err(|e| Error::Infrastructure(e.to_string()))?;
        Ok(CharacterSeedView {
            subject,
            revision_id: row.try_get("seed_revision_id").map_err(db)?,
            revision_no: row.try_get("revision_no").map_err(db)?,
            artifact_id: ArtifactId(row.try_get("artifact_id").map_err(db)?),
            text,
            media_type: row.try_get("media_type").map_err(db)?,
            provenance: row.try_get("provenance").map_err(db)?,
            created_at: row.try_get("created_at").map_err(db)?,
        })
    }
}
