use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMetadata {
    pub media_type: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationClaim {
    pub derivation_id: DerivationId,
    pub subject_id: SubjectId,
    pub source_region_id: SourceRegionId,
    pub representation_kind: String,
    pub producer_signature_id: Uuid,
    pub attempt_id: Uuid,
    pub attempt_no: i32,
    pub lease_owner: String,
    pub lease_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExtractionRequest {
    pub subject: SubjectId,
    pub source_region: SourceRegionId,
    pub media_type: String,
    pub input: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentExtractionOutput {
    pub representation_kind: RepresentationKind,
    pub text: String,
    pub producer: ProducerSignature,
    pub derived_regions: Vec<DerivedRegion>,
    pub warnings: Vec<String>,
    pub coverage: serde_json::Value,
}

#[async_trait::async_trait]
pub trait DocumentExtractionProvider: Send + Sync {
    async fn extract(&self, request: DocumentExtractionRequest)
    -> Result<DocumentExtractionOutput>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationRunResult {
    pub claimed: usize,
    pub succeeded: usize,
    pub failed: usize,
}
