use nous_core::{Error, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};

#[derive(Clone)]
pub struct MemoryStore {
    pool: PgPool,
}

impl MemoryStore {
    pub async fn connect(url: &str, max_connections: u32) -> Result<Self> {
        if max_connections == 0 {
            return Err(Error::Invalid(
                "postgres.max_connections must be positive".into(),
            ));
        }
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(url)
            .await
            .map_err(database_error)?;
        let version: String = sqlx::query_scalar("SHOW server_version_num")
            .fetch_one(&pool)
            .await
            .map_err(database_error)?;
        let version: u32 = version
            .parse()
            .map_err(|_| Error::Infrastructure("invalid PostgreSQL version".into()))?;
        if !(180000..190000).contains(&version) {
            pool.close().await;
            return Err(Error::Unavailable("PostgreSQL 18.x is required".into()));
        }
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| Error::Infrastructure(e.to_string()))
    }

    pub async fn check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(database_error)?;
        Ok(())
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
