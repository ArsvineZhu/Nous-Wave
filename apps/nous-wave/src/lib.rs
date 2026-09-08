//! Static composition of Subject Core, Cognitive Runtime and optional Memory.
mod observation;
mod context;

use nous_authority_store::AuthorityStore;
use nous_cognitive_runtime::CognitiveRuntimeService;
use nous_core::*;
use nous_material_service::MaterialService;
use nous_memory_service::MemoryService;
use nous_object_store::ObjectStore;
use nous_subject_core::SubjectCoreService;
use nous_serving::{ServingService, ServingOptions, TextEmbeddingProvider};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

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
    pub postgres_url: String,
    pub max_connections: u32,
    pub object_root: String,
    pub max_upload_bytes: u64,
    pub resident_limit: usize,
    pub memory_enabled: bool,
    pub serving_options: ServingOptions,
    pub embedding: Option<Arc<dyn TextEmbeddingProvider>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub api_version: u32,
    pub ready: bool,
    pub capabilities: Vec<CapabilityStatus>,
}

impl NousRuntime {
    pub async fn materialize(&self, subject: SubjectId, request: nous_material_service::MaterializeRequest) -> Result<nous_material_service::MaterializedEvidence> {
        self.store.validate_reference(subject,&request.reference).await?;
        let resource=match &request.reference {CognitiveRef::Resource(resource)=>Some(resource.clone()),CognitiveRef::ExternalObject(_)=>request.resource.clone(),_=>None};
        let Some(resource)=resource else{return self.material.materialize(subject,request).await;};
        let handle=request.resource_handle.as_deref().ok_or_else(||Error::Invalid("Resource materialization handle is required".into()))?;
        let source=self.cognition.materialize_resource(subject,&resource,handle).await?;
        let total=source.bytes.len() as u64;
        let range=request.byte_range.unwrap_or(nous_material_service::ByteRange {start:0,end:total});
        if request.max_bytes==0 || range.end<range.start || range.end>total {return Err(Error::Invalid("invalid resource byte range/bound".into()));}
        let end=range.end.min(range.start.saturating_add(request.max_bytes));
        Ok(nous_material_service::MaterializedEvidence {reference:request.reference.clone(),media_type:source.media_type,bytes:source.bytes[range.start as usize..end as usize].to_vec(),byte_range:nous_material_service::ByteRange {start:range.start,end},total_bytes:total,partial:range.start>0||end<total,provenance:vec![request.reference,CognitiveRef::Resource(resource)],producer:None,selection:None})
    }
    pub async fn open(options: RuntimeOptions) -> Result<Self> {
        let store = AuthorityStore::connect(&options.postgres_url, options.max_connections).await?;
        store.migrate().await?;
        let objects = ObjectStore::open(&options.object_root).await?;
        let subjects = SubjectCoreService::new(store.clone(), objects.clone());
        let cognition = CognitiveRuntimeService::new(store.clone(), options.resident_limit)?;
        let material = MaterialService::new(store.clone(), objects.clone(), cognition.clone(), options.max_upload_bytes)?;
        let mut serving_options = options.serving_options;
        serving_options.memory_enabled = options.memory_enabled;
        let serving = ServingService::new(store.clone(), objects.clone(), serving_options, options.embedding)?;
        let memory = options.memory_enabled.then(|| MemoryService::new(store.clone(), objects, cognition.clone(), serving.clone()));
        for subject in store.active_subjects().await? {
            let status = serving.refresh(subject).await?;
            for degradation in status.degradation { tracing::warn!(code=%degradation.code, detail=?degradation.detail, "serving projection degraded"); }
        }
        Ok(Self { store, subjects, cognition, memory, material, serving })
    }

    pub fn require_memory(&self) -> Result<&MemoryService> {
        self.memory.as_ref().ok_or_else(|| Error::Unavailable("Memory MicroSystem is disabled".into()))
    }

    pub async fn query(&self, query: CognitiveQuery) -> Result<CognitiveQueryResult> {
        let projection = self.serving.refresh(query.subject).await?;
        let mut result = self.cognition.query(query, self.memory.as_ref().map(|memory| memory as &dyn nous_cognitive_runtime::CognitiveContributor)).await?;
        result.degradation.extend(projection.degradation);
        if !result.degradation.is_empty() { result.status = QueryStatus::Degraded; }
        Ok(result)
    }

    pub async fn status(&self) -> RuntimeStatus {
        let mut capabilities = vec![
            CapabilityStatus { capability_id: "subject_core".into(), status: Readiness::Ready, reason: None },
            CapabilityStatus { capability_id: "cognitive_runtime".into(), status: Readiness::Ready, reason: None },
            CapabilityStatus { capability_id: "memory".into(), status: if self.memory.is_some() { Readiness::Ready } else { Readiness::Unavailable }, reason: self.memory.is_none().then(|| "disabled in composition".into()) },
        ];
        if let Some(memory) = &self.memory { capabilities.extend(memory.status().await.capabilities); }
        RuntimeStatus { api_version: API_VERSION, ready: self.store.check().await.is_ok(), capabilities }
    }
}
