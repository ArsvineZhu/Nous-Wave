use chrono::{DateTime, Utc};
use nous_core::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDescriptor {
    pub subject_id: SubjectId,
    pub resource_ref: nous_core::ResourceRef,
    pub display_label: Option<String>,
    pub authority_class: String,
    pub coverage: serde_json::Value,
    pub query_dimensions: serde_json::Value,
    pub modalities: serde_json::Value,
    pub freshness_policy: serde_json::Value,
    pub access_cost_class: String,
    pub resolver_key: String,
    pub readiness: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpsert {
    pub resource_ref: ResourceRef,
    pub display_label: Option<String>,
    pub authority_class: String,
    #[serde(default)]
    pub coverage: serde_json::Value,
    #[serde(default)]
    pub query_dimensions: serde_json::Value,
    #[serde(default)]
    pub modalities: serde_json::Value,
    #[serde(default)]
    pub freshness_policy: serde_json::Value,
    pub access_cost_class: String,
    pub resolver_key: String,
    pub readiness: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceView {
    pub descriptor: ResourceDescriptor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuery {
    pub dimensions: serde_json::Value,
    pub synopsis_only: bool,
    #[serde(default)]
    pub prefer_synopsis: bool,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQueryResult {
    pub resource: ResourceRef,
    pub current_authority: bool,
    pub records: Vec<serde_json::Value>,
    pub evidence: Vec<EvidenceHandle>,
    pub degraded: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMaterial {
    pub handle: String,
    pub media_type: String,
    pub bytes: Vec<u8>,
}

#[async_trait::async_trait]
pub trait ResourceResolver: Send + Sync {
    async fn describe(&self, resource: &ResourceRef) -> Result<ResourceDescriptor>;
    async fn query(
        &self,
        resource: &ResourceRef,
        request: ResourceQuery,
    ) -> Result<ResourceQueryResult>;
    async fn materialize(&self, handle: &str) -> Result<ResourceMaterial>;
}
