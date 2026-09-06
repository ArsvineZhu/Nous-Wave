use nous_core::{Error, Result};
use nous_material::ProcessorProvenance;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEndpoint {
    pub enabled: bool,
    /// Full HTTP operation URL. Provider gateways translate this versioned wire contract.
    pub endpoint: String,
    pub model: String,
    pub revision: String,
    pub preprocessing: String,
    pub timeout_seconds: u64,
    pub max_response_bytes: usize,
}
impl ModelEndpoint {
    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let url = reqwest::Url::parse(&self.endpoint)
            .map_err(|_| Error::Invalid("invalid model endpoint URL".into()))?;
        if !matches!(url.scheme(), "http" | "https")
            || self.model.trim().is_empty()
            || self.revision.trim().is_empty()
            || self.preprocessing.trim().is_empty()
            || self.timeout_seconds == 0
            || self.max_response_bytes == 0
        {
            return Err(Error::Invalid("enabled model role requires complete identity and positive timeout/response budget".into()));
        }
        Ok(())
    }
    pub fn provenance(&self) -> Result<ProcessorProvenance> {
        let config = serde_json::to_vec(self).map_err(|e| Error::Infrastructure(e.to_string()))?;
        Ok(ProcessorProvenance {
            identity: self.model.clone(),
            revision: self.revision.clone(),
            preprocessing: self.preprocessing.clone(),
            config_digest: blake3::hash(&config).to_hex().to_string(),
        })
    }
}

#[derive(Clone)]
pub struct ModelServices {
    client: reqwest::Client,
    pub embedding: Option<ModelEndpoint>,
    pub rerank: Option<ModelEndpoint>,
    pub generation: Option<ModelEndpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResult {
    pub model: String,
    pub revision: String,
    pub vectors: Vec<Vec<f32>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RerankResult {
    pub model: String,
    pub revision: String,
    pub ranked: Vec<RankedIndex>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RankedIndex {
    pub index: usize,
    pub score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMaterial {
    pub source_id: Uuid,
    pub artifact_id: Uuid,
    pub text: String,
    pub epistemic_class: nous_material::EpistemicClass,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedMemory {
    pub kind: nous_memory_domain::MemoryKind,
    pub title: String,
    pub text: String,
    pub source_refs: Vec<Uuid>,
    pub artifact_refs: Vec<Uuid>,
    pub entities: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub model: String,
    pub revision: String,
    pub memories: Vec<ProposedMemory>,
}

impl ModelServices {
    pub fn new(
        embedding: Option<ModelEndpoint>,
        rerank: Option<ModelEndpoint>,
        generation: Option<ModelEndpoint>,
    ) -> Result<Self> {
        for config in [&embedding, &rerank, &generation].into_iter().flatten() {
            config.validate()?;
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| Error::Infrastructure("model HTTP client initialization failed".into()))?;
        Ok(Self {
            client,
            embedding: embedding.filter(|m| m.enabled),
            rerank: rerank.filter(|m| m.enabled),
            generation: generation.filter(|m| m.enabled),
        })
    }

    pub async fn embed(&self, texts: &[String]) -> Result<(EmbeddingResult, ProcessorProvenance)> {
        let config = self
            .embedding
            .as_ref()
            .ok_or_else(|| Error::Unavailable("embedding service is not configured".into()))?;
        let mut response = self.client.post(&config.endpoint).timeout(Duration::from_secs(config.timeout_seconds))
            .json(&serde_json::json!({"api_version":1,"model":config.model,"revision":config.revision,"preprocessing":config.preprocessing,"texts":texts}))
            .send().await.map_err(|_| Error::Unavailable("embedding request failed".into()))?.error_for_status().map_err(|_| Error::Unavailable("embedding provider rejected request".into()))?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| Error::Unavailable("embedding response interrupted".into()))?
        {
            if chunk.len() > config.max_response_bytes.saturating_sub(bytes.len()) {
                return Err(Error::Invalid("embedding response exceeds budget".into()));
            }
            bytes.extend_from_slice(&chunk);
        }
        let result: EmbeddingResult = serde_json::from_slice(&bytes)
            .map_err(|_| Error::Invalid("invalid embedding response schema".into()))?;
        validate_embedding(&result, config, texts.len())?;
        Ok((result, config.provenance()?))
    }

    pub async fn rerank(
        &self,
        query: &str,
        documents: &[String],
    ) -> Result<(RerankResult, ProcessorProvenance)> {
        let config = self
            .rerank
            .as_ref()
            .ok_or_else(|| Error::Unavailable("rerank service is not configured".into()))?;
        let mut response = self.client.post(&config.endpoint).timeout(Duration::from_secs(config.timeout_seconds))
            .json(&serde_json::json!({"api_version":1,"model":config.model,"revision":config.revision,"query":query,"documents":documents}))
            .send().await.map_err(|_| Error::Unavailable("rerank request failed".into()))?.error_for_status().map_err(|_| Error::Unavailable("rerank provider rejected request".into()))?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| Error::Unavailable("rerank response interrupted".into()))?
        {
            if chunk.len() > config.max_response_bytes.saturating_sub(bytes.len()) {
                return Err(Error::Invalid("rerank response exceeds budget".into()));
            }
            bytes.extend_from_slice(&chunk);
        }
        let result: RerankResult = serde_json::from_slice(&bytes)
            .map_err(|_| Error::Invalid("invalid rerank response schema".into()))?;
        let mut indices = std::collections::HashSet::new();
        if result.model != config.model
            || result.revision != config.revision
            || result.ranked.len() != documents.len()
            || result.ranked.iter().any(|item| {
                item.index >= documents.len()
                    || !item.score.is_finite()
                    || !indices.insert(item.index)
            })
        {
            return Err(Error::Invalid(
                "rerank identity, indices or scores are invalid".into(),
            ));
        }
        Ok((result, config.provenance()?))
    }

    pub async fn extract(
        &self,
        material: &[ExtractionMaterial],
    ) -> Result<(ExtractionResult, ProcessorProvenance)> {
        let config = self.generation.as_ref().ok_or_else(|| {
            Error::Unavailable("structured generation service is not configured".into())
        })?;
        let mut response = self.client.post(&config.endpoint).timeout(Duration::from_secs(config.timeout_seconds))
            .json(&serde_json::json!({"api_version":1,"model":config.model,"revision":config.revision,"operation":"memory.extract","material":material}))
            .send().await.map_err(|_| Error::Unavailable("structured generation request failed".into()))?.error_for_status().map_err(|_| Error::Unavailable("structured generation provider rejected request".into()))?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| Error::Unavailable("generation response interrupted".into()))?
        {
            if chunk.len() > config.max_response_bytes.saturating_sub(bytes.len()) {
                return Err(Error::Invalid("generation response exceeds budget".into()));
            }
            bytes.extend_from_slice(&chunk);
        }
        let result: ExtractionResult = serde_json::from_slice(&bytes)
            .map_err(|_| Error::Invalid("invalid generation response schema".into()))?;
        if result.model != config.model
            || result.revision != config.revision
            || result.memories.len() > 128
        {
            return Err(Error::Invalid(
                "generation identity or proposal count is invalid".into(),
            ));
        }
        for memory in &result.memories {
            if memory.title.is_empty()
                || memory.title.len() > 4096
                || memory.text.len() > 65536
                || memory.source_refs.is_empty()
                || memory.artifact_refs.is_empty()
                || memory.entities.len() > 128
                || memory
                    .source_refs
                    .iter()
                    .any(|id| !material.iter().any(|item| item.source_id == *id))
                || memory
                    .artifact_refs
                    .iter()
                    .any(|id| !material.iter().any(|item| item.artifact_id == *id))
            {
                return Err(Error::Invalid(
                    "model proposal violates source or content boundaries".into(),
                ));
            }
        }
        Ok((result, config.provenance()?))
    }
}

fn validate_embedding(
    result: &EmbeddingResult,
    config: &ModelEndpoint,
    expected: usize,
) -> Result<()> {
    let dimension = result.vectors.first().map_or(0, Vec::len);
    if result.model != config.model
        || result.revision != config.revision
        || result.vectors.len() != expected
        || dimension == 0
        || dimension > 65536
        || result
            .vectors
            .iter()
            .any(|v| v.len() != dimension || v.iter().any(|value| !value.is_finite()))
    {
        return Err(Error::Invalid(
            "embedding identity, count, dimensions or values are invalid".into(),
        ));
    }
    Ok(())
}
