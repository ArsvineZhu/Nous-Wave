//! Explicit per-family Authority watermarks for independent serving updates.
use crate::{AuthorityStore, database_error as db};
use nous_core::{Result, SubjectId};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum DenseInvalidation {
    #[default]
    None,
    All,
    Spaces(Vec<String>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectionInvalidation {
    pub exact: bool,
    pub lexical: bool,
    pub dense: DenseInvalidation,
    pub topology: bool,
    pub synopsis: bool,
}

impl ProjectionInvalidation {
    pub fn text() -> Self {
        Self { exact: true, lexical: true, dense: DenseInvalidation::All, ..Self::default() }
    }

    pub fn topology() -> Self { Self { topology: true, ..Self::default() } }

    pub fn identity() -> Self { Self { exact: true, topology: true, ..Self::default() } }

    pub fn all() -> Self {
        Self { exact: true, lexical: true, dense: DenseInvalidation::All, topology: true, synopsis: true }
    }

    fn families(&self) -> Vec<(&'static str, String)> {
        let mut result = Vec::new();
        for (family, changed) in [("exact", self.exact), ("lexical", self.lexical), ("topology", self.topology), ("synopsis", self.synopsis)] {
            if changed { result.push((family, String::new())); }
        }
        match &self.dense {
            DenseInvalidation::None => {},
            DenseInvalidation::All => result.push(("dense", "*".into())),
            DenseInvalidation::Spaces(spaces) => result.extend(spaces.iter().cloned().map(|space| ("dense", space))),
        }
        result
    }
}

impl AuthorityStore {
    pub async fn invalidate(&self, subject: SubjectId, changes: ProjectionInvalidation) -> Result<i64> {
        let mut tx = self.begin().await?;
        let revision = Self::invalidate_in(&mut tx, subject, changes).await?;
        tx.commit().await.map_err(db)?;
        Ok(revision)
    }

    pub async fn invalidate_in(tx: &mut Transaction<'_, Postgres>, subject: SubjectId, changes: ProjectionInvalidation) -> Result<i64> {
        let revision = sqlx::query_scalar::<_, i64>("UPDATE subjects SET state_revision=state_revision+1 WHERE subject_id=$1 RETURNING state_revision")
            .bind(subject.0).fetch_one(&mut **tx).await.map_err(db)?;
        for (family, space) in changes.families() {
            sqlx::query("INSERT INTO projection_watermarks(subject_id,family,space_signature,desired_revision) VALUES($1,$2,$3,$4) ON CONFLICT(subject_id,family,space_signature) DO UPDATE SET desired_revision=excluded.desired_revision")
                .bind(subject.0).bind(family).bind(space).bind(revision)
                .execute(&mut **tx).await.map_err(db)?;
        }
        Ok(revision)
    }
}
