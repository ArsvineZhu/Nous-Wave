use nous_core::{CapabilityStatus, Readiness, Result};
use nous_memory_retrieval::RetrievalProjection;
use nous_memory_store::MemoryStore;
use nous_object_store::ObjectStore;
use serde::Serialize;

pub mod cycles;
pub mod material;
pub mod memory;
pub mod models;
pub mod operations;
pub mod processing;
pub mod purge;
pub mod recall;
pub mod subjects;

#[derive(Clone)]
pub struct LocalRuntime {
    pub store: MemoryStore,
    pub objects: ObjectStore,
    pub projection: Option<RetrievalProjection>,
    projection_failure: bool,
    pub models: std::sync::Arc<models::ModelServices>,
    pub(crate) process_id: uuid::Uuid,
    pub(crate) cycles: std::sync::Arc<
        tokio::sync::Mutex<std::collections::HashMap<uuid::Uuid, cycles::WorkingState>>,
    >,
}

#[derive(Serialize)]
pub struct RuntimeStatus {
    pub api_version: u32,
    pub dependencies: Vec<CapabilityStatus>,
    pub capabilities: Vec<CapabilityStatus>,
}

impl LocalRuntime {
    pub(crate) fn projection_failed(&self) -> bool {
        self.projection_failure
    }
    pub async fn open(
        postgres_url: &str,
        max_connections: u32,
        object_root: &str,
        lancedb_uri: Option<&str>,
    ) -> Result<Self> {
        let store = MemoryStore::connect(postgres_url, max_connections).await?;
        store.migrate().await?;
        let objects = ObjectStore::open(object_root).await?;
        objects.check().await?;
        let (projection, projection_failure) = match lancedb_uri {
            Some(uri) => match RetrievalProjection::open(uri).await {
                Ok(projection) => (Some(projection), false),
                Err(_) => (None, true),
            },
            None => (None, false),
        };
        Ok(Self {
            store,
            objects,
            projection,
            projection_failure,
            models: std::sync::Arc::new(models::ModelServices::new(None, None, None)?),
            process_id: uuid::Uuid::now_v7(),
            cycles: Default::default(),
        })
    }

    pub async fn status(&self) -> RuntimeStatus {
        let postgres = self.store.check().await.is_ok();
        let objects = self.objects.check().await.is_ok();
        let projection = match &self.projection {
            Some(projection) => status("lancedb", projection.check().await.is_ok()),
            None => CapabilityStatus {
                capability_id: "lancedb".into(),
                status: if self.projection_failure {
                    Readiness::Failed
                } else {
                    Readiness::Unavailable
                },
                reason: Some(
                    if self.projection_failure {
                        "projection open failed"
                    } else {
                        "memory disabled"
                    }
                    .into(),
                ),
            },
        };
        RuntimeStatus {
            api_version: nous_core::API_VERSION,
            dependencies: vec![
                status("postgres", postgres),
                status("object_store", objects),
                projection,
            ],
            // Storage connectivity does not establish semantic capability readiness.
            capabilities: vec!["subject.core", "memory"]
                .into_iter()
                .map(|id| CapabilityStatus {
                    capability_id: id.into(),
                    status: Readiness::Unavailable,
                    reason: Some("semantic service implementation not complete".into()),
                })
                .collect(),
        }
    }
}

fn status(id: &str, ready: bool) -> CapabilityStatus {
    CapabilityStatus {
        capability_id: id.into(),
        status: if ready {
            Readiness::Ready
        } else {
            Readiness::Failed
        },
        reason: if ready {
            None
        } else {
            Some("dependency check failed".into())
        },
    }
}
