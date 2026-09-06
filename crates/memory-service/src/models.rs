use nous_core::{Error, Result};
use nous_material::ProcessorProvenance;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct LocalEmbeddingModel {
    model: Arc<Mutex<fastembed::TextEmbedding>>,
    gate: Arc<tokio::sync::Semaphore>,
    provenance: ProcessorProvenance,
    e5_prefix: bool,
}

impl LocalEmbeddingModel {
    pub async fn load(cache_dir: PathBuf, identity: &str) -> Result<Self> {
        let selected = match identity {
            "BAAI/bge-m3" => fastembed::EmbeddingModel::BGEM3,
            "intfloat/multilingual-e5-base" => fastembed::EmbeddingModel::MultilingualE5Base,
            "intfloat/multilingual-e5-small" => fastembed::EmbeddingModel::MultilingualE5Small,
            _ => return Err(Error::Invalid("unsupported local embedding model".into())),
        };
        let identity = identity.to_owned();
        tokio::task::spawn_blocking(move || {
            let model_dir = cache_dir.join(format!("models--{}", identity.replace('/', "--")));
            if !model_dir.is_dir() {
                return Err(Error::Unavailable(format!(
                    "bundled local model assets are missing at {}",
                    model_dir.display()
                )));
            }
            let options = fastembed::TextInitOptions::new(selected.clone())
                .with_cache_dir(cache_dir.clone())
                .with_show_download_progress(false)
                .with_intra_threads(2);
            let model = fastembed::TextEmbedding::try_new(options).map_err(|error| {
                Error::Unavailable(format!("local embedding model failed to load: {error}"))
            })?;
            let revision = std::fs::read_to_string(
                cache_dir
                    .join(format!("models--{}", identity.replace('/', "--")))
                    .join("refs/main"),
            )
            .map_err(|e| {
                Error::Unavailable(format!("local model asset identity unavailable: {e}"))
            })?;
            let e5_prefix = identity.starts_with("intfloat/");
            let preprocessing = if e5_prefix {
                "e5-query-passage-l2-v1"
            } else {
                "identity-l2-v1"
            }
            .to_string();
            let dimension = fastembed::TextEmbedding::get_model_info(&selected)
                .map_err(|e| Error::Invalid(e.to_string()))?
                .dim;
            let config_digest = blake3::hash(
                format!("fastembed-6.0.2:{identity}:{revision}:{preprocessing}:{dimension}")
                    .as_bytes(),
            )
            .to_hex()
            .to_string();
            Ok(Self {
                model: Arc::new(Mutex::new(model)),
                gate: Arc::new(tokio::sync::Semaphore::new(1)),
                provenance: ProcessorProvenance {
                    identity,
                    revision: revision.trim().into(),
                    preprocessing,
                    config_digest,
                },
                e5_prefix,
            })
        })
        .await
        .map_err(|error| Error::Infrastructure(error.to_string()))?
    }

    pub async fn embed(
        &self,
        texts: &[String],
        query: bool,
    ) -> Result<(EmbeddingResult, ProcessorProvenance)> {
        let permit = self
            .gate
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Error::Unavailable("local embedding closed".into()))?;
        let model = self.model.clone();
        let inputs: Vec<String> = texts
            .iter()
            .map(|text| {
                if self.e5_prefix {
                    format!("{}: {text}", if query { "query" } else { "passage" })
                } else {
                    text.clone()
                }
            })
            .collect();
        let provenance = self.provenance.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let mut guard = model
                .lock()
                .map_err(|_| Error::Infrastructure("local embedding lock poisoned".into()))?;
            let vectors = guard
                .embed(inputs, None)
                .map_err(|error| Error::Unavailable(format!("local embedding failed: {error}")))?;
            let vectors = vectors
                .into_iter()
                .map(|mut vector| {
                    normalize_vector(&mut vector)?;
                    Ok(vector)
                })
                .collect::<Result<Vec<_>>>()?;
            Ok((
                EmbeddingResult {
                    model: provenance.identity.clone(),
                    revision: provenance.revision.clone(),
                    vectors,
                },
                provenance,
            ))
        })
        .await
        .map_err(|error| Error::Infrastructure(error.to_string()))?
    }
}

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
    pub local_embedding: Option<LocalEmbeddingModel>,
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
            local_embedding: None,
        })
    }

    pub fn with_local_embedding(mut self, model: LocalEmbeddingModel) -> Self {
        self.local_embedding = Some(model);
        self
    }

    pub fn embedding_available(&self) -> bool {
        self.local_embedding.is_some() || self.embedding.is_some()
    }

    pub async fn embed(&self, texts: &[String]) -> Result<(EmbeddingResult, ProcessorProvenance)> {
        if let Some(model) = &self.local_embedding {
            return model.embed(texts, false).await;
        }
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
        let mut result: EmbeddingResult = serde_json::from_slice(&bytes)
            .map_err(|_| Error::Invalid("invalid embedding response schema".into()))?;
        validate_embedding(&result, config, texts.len())?;
        for vector in &mut result.vectors {
            normalize_vector(vector)?;
        }
        Ok((result, config.provenance()?))
    }

    pub async fn embed_query(&self, text: &str) -> Result<(EmbeddingResult, ProcessorProvenance)> {
        if let Some(model) = &self.local_embedding {
            return model.embed(&[text.to_owned()], true).await;
        }
        self.embed(&[text.to_owned()]).await
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

fn normalize_vector(vector: &mut [f32]) -> Result<()> {
    let norm = vector
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm <= f64::EPSILON {
        return Err(Error::Invalid(
            "embedding must have a finite nonzero norm".into(),
        ));
    }
    for value in vector {
        *value = (*value as f64 / norm) as f32;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn endpoint(enabled: bool) -> ModelEndpoint {
        ModelEndpoint {
            enabled,
            endpoint: "http://127.0.0.1:9/model".into(),
            model: "test-model".into(),
            revision: "r1".into(),
            preprocessing: "identity".into(),
            timeout_seconds: 1,
            max_response_bytes: 1024,
        }
    }
    #[tokio::test]
    async fn unconfigured_provider_is_truthfully_unavailable() {
        let services = ModelServices::new(None, None, None).unwrap();
        let error = services.embed(&["text".into()]).await.unwrap_err();
        assert!(matches!(error, Error::Unavailable(_)));
    }
    #[test]
    fn disabled_endpoint_does_not_require_a_url() {
        let mut config = endpoint(false);
        config.endpoint.clear();
        config.model.clear();
        assert!(config.validate().is_ok());
        assert!(endpoint(true).provenance().is_ok());
    }
}
