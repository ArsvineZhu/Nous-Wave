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
    pub readiness: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceView {
    pub descriptor: ResourceDescriptor,
}
