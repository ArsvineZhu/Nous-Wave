use super::*;
use nous_authority_store::database_error as db;
use nous_core::Result;
use sqlx::Row;
use uuid::Uuid;
impl KernelService {
    pub(super) async fn get_occurrence(&self, input: p::ObjectRequest) -> Result<p::Occurrence> {
        let r = sqlx::query(
            "SELECT * FROM observation_occurrences WHERE subject_id=$1 AND occurrence_id=$2",
        )
        .bind(id(&input.subject_id)?)
        .bind(id(&input.id)?)
        .fetch_one(self.0.store.pool())
        .await
        .map_err(db)?;
        Ok(p::Occurrence {
            occurrence_id: input.id,
            subject_id: input.subject_id,
            artifact_id: r
                .try_get::<Option<Uuid>, _>("artifact_id")
                .map_err(db)?
                .map(|i| i.to_string()),
            source_class: r.try_get("source_class").map_err(db)?,
            external_object_ref: r.try_get("external_object_ref").map_err(db)?,
            occurred_at: r
                .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("occurred_at")
                .map_err(db)?
                .map(timestamp),
            observed_at: Some(timestamp(r.try_get("observed_at").map_err(db)?)),
            conversation_ref: r.try_get("conversation_ref").map_err(db)?,
            actor_entity_ref: r.try_get("actor_entity_ref").map_err(db)?,
            context: to_object(r.try_get("context").map_err(db)?),
        })
    }
    pub(super) async fn get_source_region(
        &self,
        input: p::ObjectRequest,
    ) -> Result<p::SourceRegion> {
        let r =
            sqlx::query("SELECT * FROM source_regions WHERE subject_id=$1 AND source_region_id=$2")
                .bind(id(&input.subject_id)?)
                .bind(id(&input.id)?)
                .fetch_one(self.0.store.pool())
                .await
                .map_err(db)?;
        Ok(p::SourceRegion {
            source_region_id: input.id,
            artifact_id: r.try_get::<Uuid, _>("artifact_id").map_err(db)?.to_string(),
            coordinate_kind: r.try_get("coordinate_kind").map_err(db)?,
            coordinate: to_object(r.try_get("coordinate").map_err(db)?),
            coordinate_hash: r.try_get("coordinate_hash").map_err(db)?,
            parent_source_region_id: r
                .try_get::<Option<Uuid>, _>("parent_source_region_id")
                .map_err(db)?
                .map(|i| i.to_string()),
        })
    }
    pub(super) async fn get_derived_representation(
        &self,
        input: p::ObjectRequest,
    ) -> Result<p::DerivedRepresentation> {
        let r=sqlx::query("SELECT d.*,p.signature_hash FROM derived_representations d JOIN producer_signatures p ON p.producer_signature_id=d.producer_signature_id WHERE d.subject_id=$1 AND d.derived_representation_id=$2").bind(id(&input.subject_id)?).bind(id(&input.id)?).fetch_one(self.0.store.pool()).await.map_err(db)?;
        Ok(p::DerivedRepresentation {
            representation_id: input.id,
            subject_id: input.subject_id,
            source_region_id: Some(
                r.try_get::<Uuid, _>("source_region_id")
                    .map_err(db)?
                    .to_string(),
            ),
            kind: r.try_get("representation_kind").map_err(db)?,
            producer_signature: r.try_get("signature_hash").map_err(db)?,
            revision: r.try_get("revision").map_err(db)?,
            text: r.try_get("payload_text").map_err(db)?,
            artifact_id: r
                .try_get::<Option<Uuid>, _>("payload_artifact_id")
                .map_err(db)?
                .map(|i| i.to_string()),
        })
    }
}
