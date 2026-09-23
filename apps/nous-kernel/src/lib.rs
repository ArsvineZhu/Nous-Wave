//! Static composition of Subject Core, Cognitive Runtime and optional Memory.
mod context;
pub mod transport;

use nous_authority_store::AuthorityStore;
use nous_cognitive_runtime::CognitiveRuntimeService;
use nous_core::*;
use nous_material_service::MaterialService;
use nous_memory_service::MemoryService;
use nous_object_store::ObjectStore;
use nous_serving::{ServingOptions, ServingService, TextEmbeddingProvider};
use nous_subject_core::SubjectCoreService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct NousRuntime {
    pub store: AuthorityStore,
    pub subjects: SubjectCoreService,
    pub cognition: CognitiveRuntimeService,
    pub memory: Option<MemoryService>,
    pub material: MaterialService,
    pub serving: ServingService,
}

pub struct RuntimeOptions {
    pub accessibility_policy:nous_memory_service::AccessibilityPolicy,
    pub postgres_url: String,
    pub max_connections: u32,
    pub object_root: String,
    pub max_upload_bytes: u64,
    pub resident_limit: usize,
    pub memory_enabled: bool,
    pub serving_options: ServingOptions,
    pub embedding: Option<Arc<dyn TextEmbeddingProvider>>,
    pub stored_embedding: Option<nous_serving::StoredEmbeddingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub api_version: u32,
    pub ready: bool,
    pub capabilities: Vec<CapabilityStatus>,
}

impl NousRuntime {
    pub async fn materialize(
        &self,
        subject: SubjectId,
        request: nous_material_service::MaterializeRequest,
    ) -> Result<nous_material_service::MaterializedEvidence> {
        self.material.materialize(subject, request).await
    }
    pub async fn open(options: RuntimeOptions) -> Result<Self> {
        let store = AuthorityStore::connect(&options.postgres_url, options.max_connections).await?;
        store.migrate().await?;
        let objects = ObjectStore::open(&options.object_root).await?;
        let subjects = SubjectCoreService::new(store.clone(), objects.clone());
        let cognition = CognitiveRuntimeService::new(store.clone(), options.resident_limit)?;
        let material = MaterialService::new(
            store.clone(),
            objects.clone(),
            cognition.clone(),
            options.max_upload_bytes,
        )?;
        let mut serving_options = options.serving_options;
        serving_options.memory_enabled = options.memory_enabled;
        let serving = ServingService::new(
            store.clone(),
            objects.clone(),
            serving_options,
            match options.stored_embedding {
                Some(config) => Some(Arc::new(nous_serving::StoredEmbeddingProvider::new(
                    store.clone(),
                    config,
                )?)),
                None => options.embedding,
            },
        )?;
        let accessibility_policy=options.accessibility_policy.validate()?;
        let memory = options.memory_enabled.then(|| {
            let mut memory=MemoryService::new(store.clone(), objects, cognition.clone(), serving.clone());
            memory.accessibility_policy=accessibility_policy;
            memory
        });
        for subject in store.active_subjects().await? {
            let status = serving.refresh(subject).await?;
            for degradation in status.degradation {
                tracing::warn!(code=%degradation.code, detail=?degradation.detail, "serving projection degraded");
            }
        }
        Ok(Self {
            store,
            subjects,
            cognition,
            memory,
            material,
            serving,
        })
    }

    pub fn require_memory(&self) -> Result<&MemoryService> {
        self.memory
            .as_ref()
            .ok_or_else(|| Error::Unavailable("Memory MicroSystem is disabled".into()))
    }

    pub async fn query(&self, query: CognitiveQuery) -> Result<CognitiveQueryResult> {
        query.validate()?;
        let plan = nous_cognitive_runtime::QueryPlan::for_query(&query);
        let projection = self
            .serving
            .prepare(query.subject, plan.serving_need(&query))
            .await?;
        let mut result = self
            .cognition
            .query_with_plan(
                query,
                self.memory
                    .as_ref()
                    .map(|memory| memory as &dyn nous_cognitive_runtime::CognitiveContributor),
                plan,
            )
            .await?;
        result.degradation.extend(projection.degradation);
        if !result.degradation.is_empty() {
            result.status = if result
                .degradation
                .iter()
                .any(|d| d.code == "required_external_authority_unresolved")
            {
                QueryStatus::Partial
            } else {
                QueryStatus::Degraded
            };
        }
        Ok(result)
    }

    pub async fn status(&self) -> RuntimeStatus {
        let mut capabilities = vec![
            CapabilityStatus {
                capability_id: "subject_core".into(),
                status: Readiness::Ready,
                reason: None,
            },
            CapabilityStatus {
                capability_id: "cognitive_runtime".into(),
                status: Readiness::Ready,
                reason: None,
            },
            CapabilityStatus {
                capability_id: "memory".into(),
                status: if self.memory.is_some() {
                    Readiness::Ready
                } else {
                    Readiness::Unavailable
                },
                reason: self
                    .memory
                    .is_none()
                    .then(|| "disabled in composition".into()),
            },
        ];
        if let Some(memory) = &self.memory {
            capabilities.extend(memory.status().await.capabilities);
        }
        RuntimeStatus {
            api_version: API_VERSION,
            ready: self.store.check().await.is_ok(),
            capabilities,
        }
    }
}
