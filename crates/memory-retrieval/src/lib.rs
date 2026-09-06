use nous_core::{Error, Result};
pub mod association;
pub mod dense;
pub mod residual;

#[derive(Clone)]
pub struct RetrievalProjection {
    connection: lancedb::Connection,
}

impl RetrievalProjection {
    pub async fn open(uri: &str) -> Result<Self> {
        let connection = lancedb::connect(uri)
            .execute()
            .await
            .map_err(projection_error)?;
        let projection = Self { connection };
        projection.check().await?;
        Ok(projection)
    }

    pub async fn check(&self) -> Result<()> {
        self.connection
            .table_names()
            .execute()
            .await
            .map_err(projection_error)?;
        Ok(())
    }
}

fn projection_error(error: lancedb::Error) -> Error {
    Error::Infrastructure(error.to_string())
}
