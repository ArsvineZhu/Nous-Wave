use super::*;
use nous_authority_store::database_error as db;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ByteRange { pub start: u64, pub end: u64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializeRequest {
    pub reference: CognitiveRef,
    pub byte_range: Option<ByteRange>,
    #[serde(default = "default_read_bound")]
    pub max_bytes: u64,
    pub resource_handle: Option<String>,
    pub resource: Option<ResourceRef>,
}

fn default_read_bound() -> u64 { 1024 * 1024 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterializedEvidence {
    pub reference: CognitiveRef,
    pub media_type: String,
    pub bytes: Vec<u8>,
    pub byte_range: ByteRange,
    pub total_bytes: u64,
    pub partial: bool,
    pub provenance: Vec<CognitiveRef>,
    pub producer: Option<serde_json::Value>,
    pub selection: Option<serde_json::Value>,
}

struct Payload {
    text: Option<String>, hash: Option<String>, media_type: String, length: u64,
    selected: Option<ByteRange>, provenance: Vec<CognitiveRef>, producer: Option<serde_json::Value>, selection: Option<serde_json::Value>,
}

impl MaterialService {
    pub async fn materialize(&self, subject: SubjectId, request: MaterializeRequest) -> Result<MaterializedEvidence> {
        if request.max_bytes==0 || request.max_bytes > self.max_upload_bytes { return Err(Error::Invalid("materialization byte bound is invalid".into())); }
        self.store.validate_reference(subject, &request.reference).await?;
        let mut payload = self.payload(subject, &request.reference).await?;
        let selected = payload.selected.unwrap_or(ByteRange { start:0, end:payload.length });
        let requested = request.byte_range.unwrap_or(ByteRange { start:0, end:selected.end-selected.start });
        if requested.end < requested.start || requested.end > selected.end-selected.start { return Err(Error::Invalid("requested range is outside selected source".into())); }
        let start = selected.start + requested.start;
        let end = (selected.start + requested.end).min(start.saturating_add(request.max_bytes));
        let bytes = if let Some(text) = payload.text.take() { text.as_bytes()[start as usize..end as usize].to_vec() }
            else if let Some(hash) = payload.hash { self.objects.read_range(&hash, start, end).await? }
            else { return Err(Error::Unavailable("material has no locally available payload".into())); };
        Ok(MaterializedEvidence { reference:request.reference, media_type:payload.media_type, bytes, byte_range:ByteRange { start,end }, total_bytes:payload.length, partial:start>0 || end<payload.length, provenance:payload.provenance, producer:payload.producer, selection:payload.selection })
    }

    async fn payload(&self, subject: SubjectId, reference: &CognitiveRef) -> Result<Payload> {
        let mut provenance = vec![reference.clone()];
        let mut selection = None;
        let mut selected = None;
        let mut reference = reference.clone();
        if let CognitiveRef::Occurrence(id) = reference {
            let artifact: Option<Uuid> = sqlx::query_scalar("SELECT artifact_id FROM observation_occurrences WHERE subject_id=$1 AND occurrence_id=$2").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?;
            reference = CognitiveRef::Artifact(ArtifactId(artifact.ok_or_else(|| Error::Unavailable("external occurrence requires its Host resolver".into()))?));
            provenance.push(reference.clone());
        }
        if let CognitiveRef::SourceRegion(id) = reference {
            let row = sqlx::query("SELECT artifact_id,coordinate_kind,coordinate FROM source_regions WHERE subject_id=$1 AND source_region_id=$2").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?;
            let coordinate: serde_json::Value = row.try_get("coordinate").map_err(db)?;
            selected = coordinate_range(&row.try_get::<String,_>("coordinate_kind").map_err(db)?, &coordinate)?;
            selection=Some(coordinate);
            reference=CognitiveRef::Artifact(ArtifactId(row.try_get("artifact_id").map_err(db)?)); provenance.push(reference.clone());
        }
        if let CognitiveRef::DerivedRegion(id) = reference {
            let row = sqlx::query("SELECT derived_representation_id,coordinate_kind,coordinate FROM derived_regions WHERE subject_id=$1 AND derived_region_id=$2").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?;
            let coordinate: serde_json::Value = row.try_get("coordinate").map_err(db)?;
            selected=coordinate_range(&row.try_get::<String,_>("coordinate_kind").map_err(db)?,&coordinate)?;
            selection=Some(coordinate);
            reference=CognitiveRef::DerivedRepresentation(DerivedRepresentationId(row.try_get("derived_representation_id").map_err(db)?)); provenance.push(reference.clone());
        }
        let mut payload = match reference {
            CognitiveRef::Artifact(id) => {
                let artifact=self.artifact(subject,id).await?;
                Payload { text:None, hash:Some(artifact.content_hash), media_type:artifact.media_type, length:artifact.byte_length, selected:None, provenance, producer:None, selection }
            },
            CognitiveRef::DerivedRepresentation(id) => self.derived_payload(subject,id,provenance,selection).await?,
            _ => return Err(Error::Unavailable("reference needs its semantic owner or Host materializer".into())),
        };
        if let Some(range)=selected {
            if range.end<range.start || range.end>payload.length { return Err(Error::Invalid("source coordinate is outside payload".into())); }
            payload.selected=Some(range);
        }
        Ok(payload)
    }

    async fn derived_payload(&self, subject: SubjectId, id: DerivedRepresentationId, mut provenance: Vec<CognitiveRef>, selection: Option<serde_json::Value>) -> Result<Payload> {
        let row=sqlx::query("SELECT d.payload_text,d.payload_artifact_id,d.source_region_id,to_jsonb(p) AS producer FROM derived_representations d JOIN producer_signatures p USING(producer_signature_id) WHERE d.subject_id=$1 AND d.derived_representation_id=$2")
            .bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?;
        provenance.push(CognitiveRef::SourceRegion(SourceRegionId(row.try_get("source_region_id").map_err(db)?)));
        let text: Option<String>=row.try_get("payload_text").map_err(db)?;
        let (hash,media_type,length)=if let Some(text)=&text { (None,"text/plain; charset=utf-8".into(),text.len() as u64) }
            else { let artifact=self.artifact(subject,ArtifactId(row.try_get("payload_artifact_id").map_err(db)?)).await?; provenance.push(CognitiveRef::Artifact(artifact.artifact_id)); (Some(artifact.content_hash),artifact.media_type,artifact.byte_length) };
        Ok(Payload { text,hash,media_type,length,selected:None,provenance,producer:Some(row.try_get("producer").map_err(db)?),selection })
    }
}

fn coordinate_range(kind: &str, coordinate: &serde_json::Value) -> Result<Option<ByteRange>> {
    match kind {
        "whole_artifact" => Ok(None),
        "byte_range" | "text_span" => {
            let start=coordinate.get("start").and_then(|v|v.as_u64()).ok_or_else(|| Error::Invalid("coordinate start is required".into()))?;
            let end=coordinate.get("end").and_then(|v|v.as_u64()).ok_or_else(|| Error::Invalid("coordinate end is required".into()))?;
            Ok(Some(ByteRange { start,end }))
        },
        _ => Err(Error::Unavailable(format!("coordinate materializer for '{kind}' is unavailable"))),
    }
}
