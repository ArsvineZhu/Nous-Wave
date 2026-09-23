use super::*;
use sqlx::{Postgres, Row, Transaction};

impl MemoryService {
    pub(crate) async fn validate_evidence(
        &self,
        subject: SubjectId,
        evidence: &[MemoryRevisionEvidence],
    ) -> Result<()> {
        if evidence.is_empty() {
            return Err(Error::Invalid("evidence is required".into()));
        }
        for item in evidence {
            let valid: bool = match item.evidence {
                EvidenceRef::Occurrence { occurrence_id } => sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM observation_occurrences WHERE subject_id=$1 AND occurrence_id=$2)").bind(subject.0).bind(occurrence_id.0).fetch_one(self.store.pool()).await.map_err(db)?,
                EvidenceRef::SourceRegion { source_region_id } => sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM source_regions WHERE subject_id=$1 AND source_region_id=$2)").bind(subject.0).bind(source_region_id.0).fetch_one(self.store.pool()).await.map_err(db)?,
                EvidenceRef::DerivedRepresentation { derived_representation_id } => sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM derived_representations WHERE subject_id=$1 AND derived_representation_id=$2)").bind(subject.0).bind(derived_representation_id.0).fetch_one(self.store.pool()).await.map_err(db)?,
                EvidenceRef::DerivedRegion { derived_region_id } => sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM derived_regions WHERE subject_id=$1 AND derived_region_id=$2)").bind(subject.0).bind(derived_region_id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            };
            if !valid {
                return Err(Error::Invalid(
                    "evidence reference does not belong to Subject".into(),
                ));
            }
        }
        Ok(())
    }

    pub(crate) async fn require_subject(&self, subject: SubjectId) -> Result<()> {
        if self.store.subject_exists(subject).await? {
            Ok(())
        } else {
            Err(Error::NotFound("subject not found".into()))
        }
    }

    pub(crate) async fn memory_has_entities(
        &self,
        revision: Uuid,
        entities: &[EntityRef],
    ) -> Result<bool> {
        if entities.is_empty() {
            return Ok(false);
        }
        let values = entities
            .iter()
            .map(|entity| entity.as_str())
            .collect::<Vec<_>>();
        let found: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_revision_entities WHERE memory_revision_id=$1 AND entity_ref=ANY($2)")
            .bind(revision).bind(values).fetch_one(self.store.pool()).await.map_err(db)?;
        Ok(found > 0)
    }

    pub(crate) async fn memory_has_all_entities(
        &self,
        revision: Uuid,
        entities: &[&EntityRef],
    ) -> Result<bool> {
        for entity in entities {
            if !self
                .memory_has_entities(revision, &[(*entity).clone()])
                .await?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) async fn memory_source_classes(&self, revision: Uuid) -> Result<Vec<String>> {
        sqlx::query_scalar::<_, String>("SELECT DISTINCT source_class FROM memory_evidence_occurrences WHERE memory_revision_id=$1")
            .bind(revision).fetch_all(self.store.pool()).await.map_err(db)
    }

    pub(crate) async fn memory_has_evidence_classes(
        &self,
        revision: Uuid,
        classes: &[String],
    ) -> Result<bool> {
        let rows = sqlx::query("SELECT support_role,occurrence_id,source_region_id,derived_representation_id,derived_region_id FROM memory_revision_evidence WHERE memory_revision_id=$1")
            .bind(revision).fetch_all(self.store.pool()).await.map_err(db)?;
        Ok(classes.iter().any(|class| {
            rows.iter().any(|row| {
                let support: String = row.try_get("support_role").unwrap_or_default();
                let kind = if row
                    .try_get::<Option<Uuid>, _>("occurrence_id")
                    .unwrap_or(None)
                    .is_some()
                {
                    "occurrence"
                } else if row
                    .try_get::<Option<Uuid>, _>("source_region_id")
                    .unwrap_or(None)
                    .is_some()
                {
                    "source_region"
                } else if row
                    .try_get::<Option<Uuid>, _>("derived_representation_id")
                    .unwrap_or(None)
                    .is_some()
                {
                    "derived_representation"
                } else {
                    "derived_region"
                };
                class.eq_ignore_ascii_case(&support) || class.eq_ignore_ascii_case(kind)
            })
        }))
    }

    pub(crate) async fn memory_has_modalities(
        &self,
        revision: Uuid,
        modalities: &[Modality],
    ) -> Result<bool> {
        let media_types = sqlx::query_scalar::<_, String>("SELECT DISTINCT a.media_type FROM memory_revision_evidence e JOIN observation_occurrences o ON o.occurrence_id=e.occurrence_id JOIN artifacts a ON a.artifact_id=o.artifact_id WHERE e.memory_revision_id=$1 AND a.media_type IS NOT NULL")
            .bind(revision).fetch_all(self.store.pool()).await.map_err(db)?;
        Ok(modalities.iter().any(|modality| {
            media_types.iter().any(|media_type| match modality {
                Modality::Text => {
                    media_type.starts_with("text/")
                        || media_type.contains("json")
                        || media_type.contains("xml")
                }
                Modality::Image => media_type.starts_with("image/"),
                Modality::Audio => media_type.starts_with("audio/"),
                Modality::Video => media_type.starts_with("video/"),
                Modality::Structured => media_type.contains("json") || media_type.contains("xml"),
                Modality::Binary => true,
            })
        }))
    }

    pub(crate) async fn revision_entities(&self, revision: Uuid) -> Result<Vec<String>> {
        sqlx::query_scalar::<_, String>("SELECT DISTINCT entity_ref FROM memory_revision_entities WHERE memory_revision_id=$1 ORDER BY entity_ref")
            .bind(revision).fetch_all(self.store.pool()).await.map_err(db)
    }

    pub(crate) async fn revision_tags(&self, revision: Uuid) -> Result<Vec<TagId>> {
        Ok(sqlx::query_scalar::<_, Uuid>("SELECT tag_id FROM memory_revision_tags WHERE memory_revision_id=$1 ORDER BY ordinal NULLS LAST,tag_id")
            .bind(revision).fetch_all(self.store.pool()).await.map_err(db)?.into_iter().map(TagId).collect())
    }

    pub(crate) async fn revision_anchors(&self, revision: Uuid) -> Result<Vec<AnchorId>> {
        Ok(sqlx::query_scalar::<_, Uuid>("SELECT DISTINCT a.anchor_id FROM anchors a JOIN anchor_revisions ar ON ar.anchor_revision_id=a.current_revision_id JOIN anchor_support s ON s.anchor_revision_id=ar.anchor_revision_id WHERE a.status='active' AND ((s.support_ref_kind='memory_revision' AND s.support_ref=$1) OR (s.support_ref_kind='memory' AND s.support_ref=(SELECT memory_id::text FROM memory_revisions WHERE memory_revision_id=$2))) ORDER BY a.anchor_id")
            .bind(revision.to_string()).bind(revision).fetch_all(self.store.pool()).await.map_err(db)?.into_iter().map(AnchorId).collect())
    }

    pub(crate) async fn candidate_trail(
        &self,
        revision: Uuid,
        memory: CognitiveRef,
        wave: &WaveGraphGeneration,
    ) -> Result<CandidateSemanticTrail> {
        let rows = sqlx::query("SELECT tag_id,ordinal,order_provenance FROM memory_revision_tags WHERE memory_revision_id=$1 ORDER BY ordinal NULLS LAST,tag_id")
            .bind(revision).fetch_all(self.store.pool()).await.map_err(db)?;
        let ordinals = rows
            .iter()
            .map(|row| row.try_get::<Option<i32>, _>("ordinal").unwrap_or(None))
            .collect::<Vec<_>>();
        let provenances = rows
            .iter()
            .map(|row| {
                row.try_get::<Option<serde_json::Value>, _>("order_provenance")
                    .unwrap_or(None)
            })
            .collect::<Vec<_>>();
        let ordered = semantic_order_is_declared(&ordinals, &provenances);
        let mut nodes = Vec::new();
        if ordered {
            for row in rows {
                let tag = TagId(row.try_get("tag_id").map_err(db)?);
                if let Some(node) = wave.node_id(&CognitiveRef::Tag(tag)) {
                    nodes.push(node);
                }
            }
        } else {
            for tag in self.revision_tags(revision).await? {
                if let Some(node) = wave.node_id(&CognitiveRef::Tag(tag)) {
                    nodes.push(node);
                }
            }
            for entity in self.revision_entities(revision).await? {
                let entity = EntityRef::new(entity)?;
                if let Some(node) = wave.node_id(&CognitiveRef::Entity(entity)) {
                    nodes.push(node);
                }
            }
            for anchor in self.revision_anchors(revision).await? {
                if let Some(node) = wave.node_id(&CognitiveRef::Anchor(anchor)) {
                    nodes.push(node);
                }
            }
        }
        if nodes.is_empty()
            && let Some(node) = wave.node_id(&memory)
        {
            nodes.push(node);
        }
        let order = if ordered {
            TrailOrder::Ordered
        } else if nodes.is_empty() {
            TrailOrder::Unavailable
        } else {
            TrailOrder::Unordered
        };
        Ok(CandidateSemanticTrail {
            memory,
            nodes,
            order,
            provenance: Some(if ordered {
                "memory_revision_tags explicit order_provenance".into()
            } else {
                "current Authority topology without declared order".into()
            }),
        })
    }

    pub(crate) async fn reference_in_subject(
        &self,
        subject: SubjectId,
        reference: &CognitiveRef,
    ) -> Result<bool> {
        let valid = match reference {
            CognitiveRef::Memory(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memory_objects WHERE subject_id=$1 AND memory_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::MemoryRevision(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::Artifact(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM artifacts WHERE subject_id=$1 AND artifact_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::SourceRegion(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM source_regions WHERE subject_id=$1 AND source_region_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::DerivedRepresentation(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM derived_representations WHERE subject_id=$1 AND derived_representation_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::DerivedRegion(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM derived_regions WHERE subject_id=$1 AND derived_region_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::Tag(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM tags WHERE subject_id=$1 AND tag_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::Anchor(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM anchors WHERE subject_id=$1 AND anchor_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::Resource(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM resources WHERE subject_id=$1 AND resource_ref=$2)").bind(subject.0).bind(id.as_str()).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::ExternalObject(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM observation_occurrences WHERE subject_id=$1 AND external_object_ref=$2)").bind(subject.0).bind(id.as_str()).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::Occurrence(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM observation_occurrences WHERE subject_id=$1 AND occurrence_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
            CognitiveRef::Entity(id) => {
                EntityRef::new(id.as_str())?;
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM observation_occurrences WHERE subject_id=$1 AND actor_entity_ref=$2) OR EXISTS(SELECT 1 FROM entity_mentions m JOIN entity_binding_revisions b ON b.mention_id=m.mention_id WHERE m.subject_id=$1 AND b.revision_no=(SELECT max(b2.revision_no) FROM entity_binding_revisions b2 WHERE b2.mention_id=b.mention_id) AND b.binding_state='bound' AND b.entity_ref=$2)")
                    .bind(subject.0)
                    .bind(id.as_str())
                    .fetch_one(self.store.pool())
                    .await
                    .map_err(db)?
            }
            CognitiveRef::Session(id) => sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM cognitive_sessions WHERE subject_id=$1 AND session_id=$2)").bind(subject.0).bind(id.0).fetch_one(self.store.pool()).await.map_err(db)?,
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

fn semantic_order_is_declared(
    ordinals: &[Option<i32>],
    provenances: &[Option<serde_json::Value>],
) -> bool {
    if ordinals.len() < 2 || ordinals.len() != provenances.len() {
        return false;
    }
    let mut values = ordinals.iter().copied().flatten().collect::<Vec<_>>();
    values.sort_unstable();
    values.len() == ordinals.len()
        && values.windows(2).all(|pair| pair[0] < pair[1])
        && provenances
            .iter()
            .all(|value| value.as_ref().is_some_and(|value| !value.is_null()))
}

pub(crate) async fn insert_evidence(
    tx: &mut Transaction<'_, Postgres>,
    revision: MemoryRevisionId,
    evidence: &MemoryRevisionEvidence,
) -> Result<()> {
    let (occurrence, source, derived, derived_region) = match evidence.evidence {
        EvidenceRef::Occurrence { occurrence_id } => (Some(occurrence_id.0), None, None, None),
        EvidenceRef::SourceRegion { source_region_id } => {
            (None, Some(source_region_id.0), None, None)
        }
        EvidenceRef::DerivedRepresentation {
            derived_representation_id,
        } => (None, None, Some(derived_representation_id.0), None),
        EvidenceRef::DerivedRegion { derived_region_id } => {
            (None, None, None, Some(derived_region_id.0))
        }
    };
    sqlx::query("INSERT INTO memory_revision_evidence(memory_revision_id,evidence_no,occurrence_id,source_region_id,derived_representation_id,derived_region_id,support_role) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(revision.0).bind(evidence.evidence_no).bind(occurrence).bind(source).bind(derived).bind(derived_region)
        .bind(format!("{:?}",evidence.support_role).to_lowercase())
        .execute(&mut **tx).await.map_err(db)?;
    Ok(())
}

pub(crate) fn decode_evidence(row: sqlx::postgres::PgRow) -> Result<MemoryRevisionEvidence> {
    let evidence = if let Some(id) = row
        .try_get::<Option<Uuid>, _>("occurrence_id")
        .map_err(db)?
    {
        EvidenceRef::Occurrence {
            occurrence_id: OccurrenceId(id),
        }
    } else if let Some(id) = row
        .try_get::<Option<Uuid>, _>("source_region_id")
        .map_err(db)?
    {
        EvidenceRef::SourceRegion {
            source_region_id: SourceRegionId(id),
        }
    } else if let Some(id) = row
        .try_get::<Option<Uuid>, _>("derived_representation_id")
        .map_err(db)?
    {
        EvidenceRef::DerivedRepresentation {
            derived_representation_id: DerivedRepresentationId(id),
        }
    } else {
        EvidenceRef::DerivedRegion {
            derived_region_id: DerivedRegionId(row.try_get("derived_region_id").map_err(db)?),
        }
    };
    Ok(MemoryRevisionEvidence {
        evidence_no: row.try_get("evidence_no").map_err(db)?,
        evidence,
        support_role: parse_support(&row.try_get::<String, _>("support_role").map_err(db)?)?,
    })
}

pub(crate) fn parse_memory_class(value: &str) -> Result<MemoryClass> {
    match value {
        "specific" => Ok(MemoryClass::Specific),
        "integrative" => Ok(MemoryClass::Integrative),
        "procedural" => Ok(MemoryClass::Procedural),
        _ => Err(Error::Infrastructure(
            "invalid memory class in Authority".into(),
        )),
    }
}
pub(crate) fn parse_memory_status(value: &str) -> Result<MemoryStatus> {
    match value {
        "active" => Ok(MemoryStatus::Active),
        "suppressed" => Ok(MemoryStatus::Suppressed),
        "purging" => Ok(MemoryStatus::Purging),
        _ => Err(Error::Infrastructure(
            "invalid memory status in Authority".into(),
        )),
    }
}
pub(crate) fn parse_revision_lifecycle(value: &str) -> Result<RevisionLifecycle> {
    match value {
        "current" => Ok(RevisionLifecycle::Current),
        "superseded" => Ok(RevisionLifecycle::Superseded),
        "revoked" => Ok(RevisionLifecycle::Revoked),
        _ => Err(Error::Infrastructure(
            "invalid revision state in Authority".into(),
        )),
    }
}
pub(crate) fn parse_epistemic(value: &str) -> Result<EpistemicClass> {
    match value {
        "observed" => Ok(EpistemicClass::Observed),
        "reported" => Ok(EpistemicClass::Reported),
        "derived" => Ok(EpistemicClass::Derived),
        "inferred" => Ok(EpistemicClass::Inferred),
        "narrative" => Ok(EpistemicClass::Narrative),
        "simulated" => Ok(EpistemicClass::Simulated),
        _ => Err(Error::Infrastructure(
            "invalid epistemic class in Authority".into(),
        )),
    }
}
pub(crate) fn parse_support(value: &str) -> Result<SupportRole> {
    match value {
        "direct" => Ok(SupportRole::Direct),
        "corroborating" => Ok(SupportRole::Corroborating),
        "interpretation" => Ok(SupportRole::Interpretation),
        "contradiction" => Ok(SupportRole::Contradiction),
        "contextual" => Ok(SupportRole::Contextual),
        _ => Err(Error::Infrastructure(
            "invalid support role in Authority".into(),
        )),
    }
}
pub(crate) fn subject_id_from_row(row: &sqlx::postgres::PgRow) -> Result<SubjectId> {
    Ok(SubjectId(row.try_get("subject_id").map_err(db)?))
}
pub(crate) fn interval_overlaps(
    requested: Option<TimeInterval>,
    candidate_start: Option<DateTime<Utc>>,
    candidate_end: Option<DateTime<Utc>>,
) -> bool {
    let Some(requested) = requested else {
        return true;
    };
    let Some(candidate_start) = candidate_start else {
        return false;
    };
    let candidate_end = candidate_end.unwrap_or(candidate_start);
    requested.end.is_none_or(|end| candidate_start <= end)
        && requested.start.is_none_or(|start| candidate_end >= start)
}
pub(crate) use nous_authority_store::database_error as db;

#[cfg(test)]
mod ordering_tests {
    use super::semantic_order_is_declared;

    #[test]
    fn ordinal_without_provenance_is_unordered() {
        assert!(!semantic_order_is_declared(
            &[Some(0), Some(1)],
            &[None, None]
        ));
    }

    #[test]
    fn explicit_provenance_and_distinct_ordinals_are_ordered() {
        let provenance = serde_json::json!({"source":"host"});
        assert!(semantic_order_is_declared(
            &[Some(0), Some(1)],
            &[Some(provenance.clone()), Some(provenance)]
        ));
    }

    #[test]
    fn duplicate_ordinals_are_not_ordered() {
        let provenance = serde_json::json!({"source":"host"});
        assert!(!semantic_order_is_declared(
            &[Some(0), Some(0)],
            &[Some(provenance.clone()), Some(provenance)]
        ));
    }
}

pub(crate) fn parse_accessibility_mode(value:&str)->Result<AccessibilityMode>{
    serde_json::from_value(serde_json::Value::String(value.into())).map_err(|_|Error::Infrastructure("invalid accessibility mode".into()))
}
