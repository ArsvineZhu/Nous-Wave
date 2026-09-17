//! Host-supplied model material. This adapter never invokes a model or network provider.
use crate::*;
use nous_authority_store::{DenseInvalidation, ProjectionInvalidation, database_error as db};
use std::future::Future;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEmbeddingConfig {
    pub space: EmbeddingSpaceSignature,
    pub producer: ProducerSignature,
}
#[derive(Debug, Clone)]
pub struct QueryEmbedding {
    pub text: String,
    pub output: TextEmbeddingOutput,
}
tokio::task_local! { static QUERY_MATERIAL: Vec<QueryEmbedding>; }
pub async fn with_query_material<T>(
    material: Vec<QueryEmbedding>,
    operation: impl Future<Output = T>,
) -> T {
    QUERY_MATERIAL.scope(material, operation).await
}

pub struct StoredEmbeddingProvider {
    store: AuthorityStore,
    config: StoredEmbeddingConfig,
}
impl StoredEmbeddingProvider {
    pub fn new(store: AuthorityStore, config: StoredEmbeddingConfig) -> Result<Self> {
        validate_embedding_config(&config)?;
        Ok(Self { store, config })
    }
}
pub fn validate_embedding_config(config: &StoredEmbeddingConfig) -> Result<()> {
    if config.space.dimension == 0
        || config.space.dimension > 8192
        || config.space.space_hash.is_empty()
        || config.producer.signature_hash.is_empty()
        || config.producer.model_identity.as_deref() != Some(config.space.model_identity.as_str())
        || config.producer.operation != CapabilityOperation::TextEmbedding
    {
        return Err(Error::Invalid(
            "invalid embedding space/producer configuration".into(),
        ));
    }
    Ok(())
}
#[async_trait::async_trait]
impl TextEmbeddingProvider for StoredEmbeddingProvider {
    fn space(&self) -> EmbeddingSpaceSignature {
        self.config.space.clone()
    }
    fn producer(&self) -> ProducerSignature {
        self.config.producer.clone()
    }
    async fn embed(&self, request: TextEmbeddingRequest) -> Result<TextEmbeddingOutput> {
        if request.query {
            let output = QUERY_MATERIAL
                .try_with(|items| {
                    items
                        .iter()
                        .find(|i| {
                            i.text == request.text
                                && i.output.space.compatible_with(&self.config.space)
                                && i.output.producer.signature_hash
                                    == self.config.producer.signature_hash
                        })
                        .map(|i| i.output.clone())
                })
                .ok()
                .flatten();
            return output.ok_or_else(|| {
                Error::Unavailable("Host did not supply compatible query embedding".into())
            });
        }
        let vector=sqlx::query_scalar::<_,Vec<f32>>("SELECT vector FROM embedding_materials WHERE subject_id=$1 AND content_digest=$2 AND space_hash=$3 AND producer_hash=$4")
            .bind(request.subject.0).bind(blake3::hash(request.text.as_bytes()).to_hex().to_string()).bind(&self.config.space.space_hash).bind(&self.config.producer.signature_hash)
            .fetch_optional(self.store.pool()).await.map_err(db)?.ok_or_else(||Error::Unavailable("Host embedding material is not ready".into()))?;
        Ok(TextEmbeddingOutput {
            vector,
            space: self.space(),
            producer: self.producer(),
        })
    }
}
#[derive(Debug, Clone)]
pub struct EmbeddingNeed {
    pub reference: CognitiveRef,
    pub text: String,
    pub digest: String,
}
impl ServingService {
    pub async fn embedding_needs(
        &self,
        subject: SubjectId,
        limit: usize,
    ) -> Result<Vec<EmbeddingNeed>> {
        if limit == 0 || limit > 256 {
            return Err(Error::Invalid(
                "embedding batch limit must be 1..256".into(),
            ));
        }
        self.store.require_subject(subject).await?;
        let provider = self
            .embedding
            .as_ref()
            .ok_or_else(|| Error::Unavailable("embedding space not configured".into()))?;
        let config = provider.space();
        let producer = provider.producer();
        let input = self
            .store
            .text_projection_input(
                subject,
                "dense",
                &config.space_hash,
                self.options.memory_enabled,
            )
            .await?;
        let mut needs = vec![];
        for doc in self.documents(input.sources).await? {
            let digest = blake3::hash(doc.representation_text.as_bytes())
                .to_hex()
                .to_string();
            let exists:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM embedding_materials WHERE subject_id=$1 AND content_digest=$2 AND space_hash=$3 AND producer_hash=$4)")
                .bind(subject.0).bind(&digest).bind(&config.space_hash).bind(&producer.signature_hash).fetch_one(self.store.pool()).await.map_err(db)?;
            if !exists {
                needs.push(EmbeddingNeed {
                    reference: doc.reference,
                    text: doc.representation_text,
                    digest,
                });
            }
            if needs.len() >= limit {
                break;
            }
        }
        Ok(needs)
    }
    pub async fn commit_embedding(
        &self,
        subject: SubjectId,
        reference: CognitiveRef,
        text: String,
        space: &str,
        producer: &str,
        vector: Vec<f32>,
    ) -> Result<()> {
        self.store.validate_reference(subject, &reference).await?;
        let configured = self
            .embedding
            .as_ref()
            .ok_or_else(|| Error::Unavailable("embedding not configured".into()))?;
        if configured.space().space_hash != space
            || configured.producer().signature_hash != producer
            || vector.len() != configured.space().dimension as usize
            || vector.iter().any(|v| !v.is_finite())
        {
            return Err(Error::Invalid(
                "embedding material disagrees with configured space/producer".into(),
            ));
        }
        let input = self
            .store
            .text_projection_input(subject, "dense", space, self.options.memory_enabled)
            .await?;
        let exists = self
            .documents(input.sources)
            .await?
            .into_iter()
            .any(|d| d.reference == reference && d.representation_text == text);
        if !exists {
            return Err(Error::Conflict("embedding source changed".into()));
        }
        let mut tx = self.store.begin().await?;
        let inserted=sqlx::query("INSERT INTO embedding_materials(subject_id,content_digest,space_hash,producer_hash,vector) VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
            .bind(subject.0).bind(blake3::hash(text.as_bytes()).to_hex().to_string()).bind(space).bind(producer).bind(vector).execute(&mut *tx).await.map_err(db)?;
        if inserted.rows_affected() > 0 {
            AuthorityStore::invalidate_in(
                &mut tx,
                subject,
                ProjectionInvalidation {
                    dense: DenseInvalidation::Spaces(vec![space.into()]),
                    ..Default::default()
                },
            )
            .await?;
        }
        tx.commit().await.map_err(db)
    }
}
