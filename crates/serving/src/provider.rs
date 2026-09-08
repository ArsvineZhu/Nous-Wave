use nous_core::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEmbeddingRequest {
    pub subject: SubjectId,
    pub text: String,
    pub query: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEmbeddingOutput {
    pub vector: Vec<f32>,
    pub space: EmbeddingSpaceSignature,
    pub producer: ProducerSignature,
}

#[async_trait::async_trait]
pub trait TextEmbeddingProvider: Send + Sync {
    fn space(&self) -> EmbeddingSpaceSignature;
    fn producer(&self) -> ProducerSignature;
    async fn embed(&self, request: TextEmbeddingRequest) -> Result<TextEmbeddingOutput>;
}
