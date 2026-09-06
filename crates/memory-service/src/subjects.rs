use crate::LocalRuntime;
use chrono::{DateTime, Utc};
use nous_core::{Error, Result, SeedRevision, Subject, SubjectId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SeedContent {
    Inline { content: String, media_type: String },
    InlineBytes { bytes: Vec<u8>, media_type: String },
    Artifact { artifact_id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedInput {
    pub content: SeedContent,
    pub authored_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubject {
    pub api_version: u32,
    pub label: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: serde_json::Value,
    pub character_seed: SeedInput,
    #[serde(default = "memory_default")]
    pub memory_enabled: bool,
}
fn memory_default() -> bool {
    true
}
fn empty_metadata() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceSeed {
    pub api_version: u32,
    pub expected_subject_revision: i64,
    pub character_seed: SeedInput,
}

impl LocalRuntime {
    pub async fn create_subject(&self, input: CreateSubject) -> Result<Subject> {
        let _objects = self.objects.reference_guard(false).await?;
        check_version(input.api_version)?;
        if !input.metadata.is_object() || input.metadata.to_string().len() > 65536 {
            return Err(Error::Invalid(
                "subject metadata must be an object of at most 65536 bytes".into(),
            ));
        }
        if input.label.as_ref().is_some_and(|label| label.len() > 1024) {
            return Err(Error::Invalid("subject label exceeds 1024 bytes".into()));
        }
        if matches!(&input.character_seed.content, SeedContent::Artifact { .. }) {
            return Err(Error::Invalid(
                "CreateSubject Character Seed must be supplied inline".into(),
            ));
        }
        let subject = SubjectId::default();
        let prepared = self.prepare_seed(subject, &input.character_seed).await?;
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        sqlx::query(
            "INSERT INTO subjects(subject_id,label,metadata,memory_enabled) VALUES($1,$2,$3,$4)",
        )
        .bind(subject.0)
        .bind(input.label)
        .bind(input.metadata)
        .bind(input.memory_enabled)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        write_seed(&mut tx, subject, &input.character_seed, prepared, None).await?;
        tx.commit().await.map_err(db)?;
        self.subject(subject).await
    }

    pub async fn subject(&self, subject: SubjectId) -> Result<Subject> {
        let row = sqlx::query("SELECT s.*, h.revision_id AS seed_revision FROM subjects s JOIN character_seed_heads h USING(subject_id) WHERE s.subject_id=$1")
            .bind(subject.0).fetch_optional(self.store.pool()).await.map_err(db)?
            .ok_or_else(|| Error::NotFound("subject not found".into()))?;
        decode_subject(&row)
    }

    pub async fn subjects(&self, after: Option<Uuid>, limit: u32) -> Result<Vec<Subject>> {
        if limit == 0 || limit > 1000 {
            return Err(Error::Invalid("list limit must be 1..1000".into()));
        }
        let rows = sqlx::query("SELECT s.*, h.revision_id AS seed_revision FROM subjects s JOIN character_seed_heads h USING(subject_id) WHERE ($1::uuid IS NULL OR s.subject_id > $1) ORDER BY s.subject_id LIMIT $2")
            .bind(after).bind(i64::from(limit)).fetch_all(self.store.pool()).await.map_err(db)?;
        rows.iter().map(decode_subject).collect()
    }

    pub async fn replace_seed(&self, subject: SubjectId, input: ReplaceSeed) -> Result<Subject> {
        let _objects = self.objects.reference_guard(false).await?;
        check_version(input.api_version)?;
        self.subject(subject).await?;
        let prepared = self.prepare_seed(subject, &input.character_seed).await?;
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        let changed = sqlx::query(
            "UPDATE subjects SET revision=revision+1 WHERE subject_id=$1 AND revision=$2",
        )
        .bind(subject.0)
        .bind(input.expected_subject_revision)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        if changed.rows_affected() != 1 {
            return Err(Error::Conflict("subject revision changed".into()));
        }
        let parent = sqlx::query_scalar::<_, Uuid>(
            "SELECT revision_id FROM character_seed_heads WHERE subject_id=$1",
        )
        .bind(subject.0)
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
        write_seed(
            &mut tx,
            subject,
            &input.character_seed,
            prepared,
            Some(parent),
        )
        .await?;
        tx.commit().await.map_err(db)?;
        self.subject(subject).await
    }

    pub async fn seed_history(&self, subject: SubjectId) -> Result<Vec<SeedRevision>> {
        self.subject(subject).await?;
        let rows = sqlx::query("SELECT * FROM character_seed_revisions WHERE subject_id=$1 ORDER BY created_at,revision_id")
            .bind(subject.0).fetch_all(self.store.pool()).await.map_err(db)?;
        rows.iter()
            .map(|row| {
                Ok(SeedRevision {
                    subject_id: subject,
                    revision_id: row.try_get("revision_id").map_err(db)?,
                    artifact_id: row.try_get("artifact_id").map_err(db)?,
                    authored_by: row.try_get("authored_by").map_err(db)?,
                    parent_revision: row.try_get("parent_revision").map_err(db)?,
                    created_at: row.try_get("created_at").map_err(db)?,
                })
            })
            .collect()
    }

    async fn prepare_seed(&self, subject: SubjectId, seed: &SeedInput) -> Result<PreparedSeed> {
        if seed.authored_by.trim().is_empty() || seed.authored_by.len() > 1024 {
            return Err(Error::Invalid(
                "seed authored_by must be nonempty and at most 1024 bytes".into(),
            ));
        }
        match &seed.content {
            SeedContent::Inline {
                content,
                media_type,
            } => {
                if !matches!(media_type.as_str(), "text/plain" | "text/markdown")
                    || content.trim().is_empty()
                {
                    return Err(Error::Invalid(
                        "inline Character Seed requires nonempty plain text or Markdown".into(),
                    ));
                }
                let hash = self.objects.put(content.as_bytes().to_vec()).await?;
                Ok(PreparedSeed::New {
                    artifact: Uuid::now_v7(),
                    hash,
                    size: content.len() as i64,
                    media_type: media_type.clone(),
                })
            }
            SeedContent::InlineBytes { bytes, media_type } => {
                if bytes.is_empty()
                    || bytes.len() as u64 > self.max_upload_bytes
                    || media_type.is_empty()
                    || media_type.len() > 256
                {
                    return Err(Error::Invalid(
                        "inline Character Seed bytes or media type are invalid".into(),
                    ));
                }
                let hash = self.objects.put(bytes.clone()).await?;
                Ok(PreparedSeed::New {
                    artifact: Uuid::now_v7(),
                    hash,
                    size: bytes.len() as i64,
                    media_type: media_type.clone(),
                })
            }
            SeedContent::Artifact { artifact_id } => {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM artifacts WHERE subject_id=$1 AND artifact_id=$2)",
                )
                .bind(subject.0)
                .bind(artifact_id)
                .fetch_one(self.store.pool())
                .await
                .map_err(db)?;
                if !exists {
                    return Err(Error::NotFound(
                        "seed artifact must belong to this subject".into(),
                    ));
                }
                Ok(PreparedSeed::Existing(*artifact_id))
            }
        }
    }
}

enum PreparedSeed {
    New {
        artifact: Uuid,
        hash: String,
        size: i64,
        media_type: String,
    },
    Existing(Uuid),
}

async fn write_seed(
    tx: &mut Transaction<'_, Postgres>,
    subject: SubjectId,
    input: &SeedInput,
    prepared: PreparedSeed,
    parent: Option<Uuid>,
) -> Result<()> {
    let artifact = match prepared {
        PreparedSeed::Existing(artifact) => artifact,
        PreparedSeed::New {
            artifact,
            hash,
            size,
            media_type,
        } => {
            let source = Uuid::now_v7();
            sqlx::query("INSERT INTO source_records(source_id,subject_id,source_kind,origin_class,semantic_class,epistemic_class,actor) VALUES($1,$2,'subject:character_seed','HUMAN','DOCUMENT','NARRATIVE',$3)")
                .bind(source).bind(subject.0).bind(&input.authored_by).execute(&mut **tx).await.map_err(db)?;
            sqlx::query("INSERT INTO artifacts(artifact_id,subject_id,content_hash,media_type,byte_size,origin_class,semantic_class,epistemic_class) VALUES($1,$2,$3,$4,$5,'HUMAN','DOCUMENT','NARRATIVE')")
                .bind(artifact).bind(subject.0).bind(hash).bind(media_type).bind(size).execute(&mut **tx).await.map_err(db)?;
            sqlx::query(
                "INSERT INTO artifact_sources(subject_id,artifact_id,source_id) VALUES($1,$2,$3)",
            )
            .bind(subject.0)
            .bind(artifact)
            .bind(source)
            .execute(&mut **tx)
            .await
            .map_err(db)?;
            artifact
        }
    };
    let revision = Uuid::now_v7();
    sqlx::query("INSERT INTO character_seed_revisions(revision_id,subject_id,artifact_id,authored_by,parent_revision) VALUES($1,$2,$3,$4,$5)")
        .bind(revision).bind(subject.0).bind(artifact).bind(&input.authored_by).bind(parent).execute(&mut **tx).await.map_err(db)?;
    sqlx::query("INSERT INTO character_seed_heads(subject_id,revision_id) VALUES($1,$2) ON CONFLICT(subject_id) DO UPDATE SET revision_id=excluded.revision_id")
        .bind(subject.0).bind(revision).execute(&mut **tx).await.map_err(db)?;
    Ok(())
}

fn decode_subject(row: &sqlx::postgres::PgRow) -> Result<Subject> {
    Ok(Subject {
        subject_id: SubjectId(row.try_get("subject_id").map_err(db)?),
        label: row.try_get("label").map_err(db)?,
        metadata: row.try_get("metadata").map_err(db)?,
        revision: row.try_get("revision").map_err(db)?,
        seed_revision: row.try_get("seed_revision").map_err(db)?,
        memory_enabled: row.try_get("memory_enabled").map_err(db)?,
        created_at: row.try_get::<DateTime<Utc>, _>("created_at").map_err(db)?,
    })
}

pub(crate) fn check_version(version: u32) -> Result<()> {
    if version != nous_core::API_VERSION {
        return Err(Error::Invalid("unsupported API version".into()));
    }
    Ok(())
}
pub(crate) fn db(error: sqlx::Error) -> Error {
    Error::Infrastructure(error.to_string())
}
