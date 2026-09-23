use crate::*;
use database_error as db;
use nous_core::*;

impl AuthorityStore {
    pub async fn require_subject(&self, subject: SubjectId) -> Result<()> {
        if self.subject_exists(subject).await? {
            Ok(())
        } else {
            Err(Error::NotFound("subject not found".into()))
        }
    }

    pub async fn reference_in_subject(
        &self,
        subject: SubjectId,
        reference: &CognitiveRef,
    ) -> Result<bool> {
        let valid = match reference {
            CognitiveRef::Memory(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memory_objects WHERE subject_id=$1 AND memory_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::MemoryRevision(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::Artifact(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM artifacts WHERE subject_id=$1 AND artifact_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::SourceRegion(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM source_regions WHERE subject_id=$1 AND source_region_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::DerivedRepresentation(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM derived_representations WHERE subject_id=$1 AND derived_representation_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::DerivedRegion(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM derived_regions WHERE subject_id=$1 AND derived_region_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::Tag(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM tags WHERE subject_id=$1 AND tag_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::Anchor(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM anchors WHERE subject_id=$1 AND anchor_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::Resource(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM resources WHERE subject_id=$1 AND resource_ref=$2)").bind(subject.0).bind(id.as_str()).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::ExternalObject(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM observation_occurrences WHERE subject_id=$1 AND external_object_ref=$2)").bind(subject.0).bind(id.as_str()).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::Occurrence(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM observation_occurrences WHERE subject_id=$1 AND occurrence_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
            CognitiveRef::Entity(id) => {
                EntityRef::new(id.as_str())?;
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM observation_occurrences WHERE subject_id=$1 AND actor_entity_ref=$2) OR EXISTS(SELECT 1 FROM entity_mentions m JOIN entity_binding_revisions b ON b.mention_id=m.mention_id WHERE m.subject_id=$1 AND b.revision_no=(SELECT max(b2.revision_no) FROM entity_binding_revisions b2 WHERE b2.mention_id=b.mention_id) AND b.binding_state='bound' AND b.entity_ref=$2) OR EXISTS(SELECT 1 FROM lexical_visibility v JOIN lexical_bindings l ON l.lexical_ref=v.lexical_ref WHERE v.subject_id=$1 AND l.object_kind='entity' AND l.canonical_ref=$2 AND l.tombstoned_at IS NULL) OR EXISTS(SELECT 1 FROM memory_revision_entities e JOIN memory_revisions r ON r.memory_revision_id=e.memory_revision_id WHERE r.subject_id=$1 AND e.entity_ref=$2)")
                    .bind(subject.0)
                    .bind(id.as_str())
                    .fetch_one(self.pool())
                    .await
                    .map_err(db)?
            }
            CognitiveRef::Session(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM cognitive_sessions WHERE subject_id=$1 AND session_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.pool()).await.map_err(db)?,
        };
        Ok(valid)
    }

    pub async fn validate_reference(
        &self,
        subject: SubjectId,
        reference: &CognitiveRef,
    ) -> Result<()> {
        self.require_subject(subject).await?;
        if self.reference_in_subject(subject, reference).await? {
            Ok(())
        } else {
            Err(Error::NotFound("cognitive reference not found".into()))
        }
    }
}
