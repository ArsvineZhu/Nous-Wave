use super::*;
use nous_core::{Result, SubjectId};
use nous_serving::{QueryEmbedding, TextEmbeddingOutput};

impl KernelService {
    fn embedding_config(&self) -> Result<k::EmbeddingConfig> {
        let provider = self
            .0
            .serving
            .embedding
            .as_ref()
            .ok_or_else(|| Error::Unavailable("embedding space not configured".into()))?;
        let space = provider.space();
        Ok(k::EmbeddingConfig {
            space_hash: space.space_hash,
            producer_hash: provider.producer().signature_hash,
            model: space.model_identity,
            dimension: space.dimension,
        })
    }
    pub(super) async fn query_with_material(
        &self,
        input: k::KernelQueryRequest,
    ) -> Result<p::QueryResponse> {
        if input.embeddings.len() > 64 {
            return Err(Error::Invalid("too many query embeddings".into()));
        }
        let mut materials = vec![];
        for material in input.embeddings {
            let provider = self
                .0
                .serving
                .embedding
                .as_ref()
                .ok_or_else(|| Error::Unavailable("embedding space not configured".into()))?;
            let space = provider.space();
            let producer = provider.producer();
            if material.text.len() > 32768
                || material.space_hash != space.space_hash
                || material.producer_hash != producer.signature_hash
                || material.vector.len() != space.dimension as usize
                || material.vector.iter().any(|v| !v.is_finite())
            {
                return Err(Error::Invalid(
                    "query embedding is incompatible with selected space/producer".into(),
                ));
            }
            materials.push(QueryEmbedding {
                text: material.text,
                output: TextEmbeddingOutput {
                    vector: material.vector,
                    space,
                    producer,
                },
            });
        }
        nous_serving::with_query_material(materials, self.query(required(input.query, "query")?))
            .await
    }
}
#[tonic::async_trait]
impl k::model_material_service_server::ModelMaterialService for KernelService {
    async fn commit_interpretation(
        &self,
        request: Request<k::CommitInterpretationRequest>,
    ) -> std::result::Result<Response<p::DerivedRepresentation>, Status> {
        let input = request.into_inner();
        let result:Result<_>=async{
            if input.model.is_empty()||input.implementation.is_empty()||input.text.trim().is_empty()||input.text.len()>1_048_576{return Err(Error::Invalid("invalid interpretation proposal bounds".into()));}
            let subject=SubjectId(id(&input.subject_id)?);
            let source=nous_core::SourceRegionId(id(&input.source_region_id)?);
            let kind:nous_core::RepresentationKind=enum_value(&input.kind)?;
            let operation=match kind {
                nous_core::RepresentationKind::ImageDescription=>nous_core::CapabilityOperation::ImageInterpretation,
                nous_core::RepresentationKind::Transcript=>nous_core::CapabilityOperation::SpeechTranscription,
                _=>nous_core::CapabilityOperation::DocumentExtraction,
            };
            let mut producer=nous_core::ProducerSignature{signature_hash:String::new(),provider_class:"host-model".into(),operation,implementation:input.implementation,model_identity:Some(input.model),model_revision:Some(input.model_revision),preprocessing_identity:"bounded-source".into(),preprocessing_revision:"1".into(),config_digest:"host-interpretation-v1".into()};
            producer.signature_hash=blake3::hash(&serde_json::to_vec(&producer).map_err(|e|Error::Invalid(e.to_string()))?).to_hex().to_string();
            let revision:i32=sqlx::query_scalar("SELECT COALESCE(max(d.revision),0)+1 FROM derived_representations d JOIN producer_signatures p ON p.producer_signature_id=d.producer_signature_id WHERE d.subject_id=$1 AND d.source_region_id=$2 AND d.representation_kind=$3 AND p.signature_hash=$4")
                .bind(subject.0).bind(source.0).bind(kind.as_str()).bind(&producer.signature_hash).fetch_one(self.0.store.pool()).await.map_err(nous_authority_store::database_error)?;
            let representation=self.0.material.persist_derived_representation(nous_material::DerivedRepresentation{derived_representation_id:nous_core::DerivedRepresentationId::new(),subject_id:subject,source_region_id:source,representation_kind:kind,producer,revision,payload_text:Some(input.text),payload_artifact_id:None,quality:serde_json::json!({"status":"model_interpretation"}),created_at:chrono::Utc::now(),supersedes:None}).await?;
            self.get_derived_representation(p::ObjectRequest{subject_id:subject.0.to_string(),id:representation.derived_representation_id.0.to_string()}).await
        }.await;
        result.map(Response::new).map_err(status)
    }
    async fn get_embedding_config(
        &self,
        _: Request<()>,
    ) -> std::result::Result<Response<k::EmbeddingConfig>, Status> {
        self.embedding_config().map(Response::new).map_err(status)
    }
    async fn list_embedding_needs(
        &self,
        request: Request<k::EmbeddingNeedsRequest>,
    ) -> std::result::Result<Response<k::EmbeddingNeedsResponse>, Status> {
        let input = request.into_inner();
        let result: Result<_> = async {
            let config = self.embedding_config()?;
            let needs = self
                .0
                .serving
                .embedding_needs(SubjectId(id(&input.subject_id)?), input.limit as usize)
                .await?
                .into_iter()
                .map(|n| k::EmbeddingNeed {
                    reference: Some(to_ref(n.reference)),
                    text: n.text,
                    digest: n.digest,
                })
                .collect();
            Ok(k::EmbeddingNeedsResponse {
                config: Some(config),
                needs,
            })
        }
        .await;
        result.map(Response::new).map_err(status)
    }
    async fn commit_embedding(
        &self,
        request: Request<k::CommitEmbeddingRequest>,
    ) -> std::result::Result<Response<()>, Status> {
        let input = request.into_inner();
        let result: Result<_> = async {
            let material = required(input.material, "material")?;
            self.0
                .serving
                .commit_embedding(
                    SubjectId(id(&input.subject_id)?),
                    from_ref(required(input.reference, "reference")?)?,
                    material.text,
                    &material.space_hash,
                    &material.producer_hash,
                    material.vector,
                )
                .await
        }
        .await;
        result.map(Response::new).map_err(status)
    }
}
