//! Subject identity, initialization material, configuration and lifecycle.

mod seed;

use chrono::{DateTime, Utc};
use nous_authority_store::AuthorityStore;
use nous_core::{Error, Result, SubjectId};
use nous_object_store::ObjectStore;
use serde::{Deserialize, Serialize};
use sqlx::Row;

pub use seed::{CharacterSeedInput, CharacterSeedView};

#[derive(Clone)]
pub struct SubjectCoreService {
    pub store: AuthorityStore,
    pub objects: ObjectStore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubject {
    pub subject_id: Option<SubjectId>,
    pub character_seed: CharacterSeedInput,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectView {
    pub subject_id: SubjectId,
    pub created_at: DateTime<Utc>,
    pub state_revision: i64,
    pub status: String,
    pub config: serde_json::Value,
}

impl SubjectCoreService {
    pub fn new(store: AuthorityStore, objects: ObjectStore) -> Self {
        Self { store, objects }
    }

    pub async fn create_subject(&self, input: CreateSubject) -> Result<SubjectView> {
        input.character_seed.validate()?;
        let subject = input.subject_id.unwrap_or_default();
        let guard = self.objects.reference_guard(false).await?;
        let hash = self.objects.put(input.character_seed.text.as_bytes().to_vec()).await?;
        let mut tx = self.store.begin().await?;
        sqlx::query("INSERT INTO subjects(subject_id,created_at,metadata) VALUES($1,$2,$3)")
            .bind(subject.0).bind(Utc::now()).bind(input.config)
            .execute(&mut *tx).await.map_err(nous_authority_store::database_error)?;
        seed::insert_seed(&mut tx, subject, input.character_seed, hash, 1).await?;
        tx.commit().await.map_err(nous_authority_store::database_error)?;
        drop(guard);
        self.subject(subject).await
    }

    pub async fn subject(&self, subject: SubjectId) -> Result<SubjectView> {
        let row = sqlx::query("SELECT subject_id,created_at,state_revision,status,metadata FROM subjects WHERE subject_id=$1")
            .bind(subject.0).fetch_optional(self.store.pool()).await
            .map_err(nous_authority_store::database_error)?
            .ok_or_else(|| Error::NotFound("subject not found".into()))?;
        Ok(SubjectView {
            subject_id: subject,
            created_at: row.try_get("created_at").map_err(nous_authority_store::database_error)?,
            state_revision: row.try_get("state_revision").map_err(nous_authority_store::database_error)?,
            status: row.try_get("status").map_err(nous_authority_store::database_error)?,
            config: row.try_get("metadata").map_err(nous_authority_store::database_error)?,
        })
    }
}
