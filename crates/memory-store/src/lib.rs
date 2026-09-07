//! PostgreSQL Authority repositories and migration ownership.

use nous_core::{Error, Result, SubjectId};
use sqlx::{PgPool, Postgres, Transaction, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct MemoryStore {
    pool: PgPool,
}

impl MemoryStore {
    pub async fn connect(url: &str, max_connections: u32) -> Result<Self> {
        if max_connections == 0 {
            return Err(Error::Invalid("max_connections must be positive".into()));
        }
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(std::time::Duration::from_secs(15))
            .connect(url)
            .await
            .map_err(database_error)?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|error| Error::Infrastructure(error.to_string()))
    }

    pub async fn check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(database_error)?;
        Ok(())
    }

    pub async fn subject_exists(&self, subject: SubjectId) -> Result<bool> {
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM subjects WHERE subject_id=$1)")
            .bind(subject.0)
            .fetch_one(&self.pool)
            .await
            .map_err(database_error)
    }

    pub async fn begin(&self) -> Result<Transaction<'_, Postgres>> {
        self.pool.begin().await.map_err(database_error)
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

fn database_error(error: sqlx::Error) -> Error {
    Error::Infrastructure(error.to_string())
}
