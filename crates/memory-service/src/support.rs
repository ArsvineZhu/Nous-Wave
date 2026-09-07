use super::*;
use sqlx::{Postgres, Row, Transaction};

impl LocalRuntime {
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

    pub(crate) async fn require_session(
        &self,
        subject: SubjectId,
        session: SessionId,
    ) -> Result<()> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM cognitive_sessions WHERE subject_id=$1 AND session_id=$2 AND closed_at IS NULL)")
            .bind(subject.0)
            .bind(session.0)
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
        if exists {
            Ok(())
        } else {
            Err(Error::NotFound("open session not found".into()))
        }
    }

    pub(crate) async fn admit(
        &self,
        session: SessionId,
        reference: CognitiveRef,
        reason: &str,
        hold_until: Option<DateTime<Utc>>,
    ) -> Result<()> {
        self.admit_with_state(session, reference, reason, hold_until, "resident")
            .await
    }

    pub(crate) async fn admit_with_state(
        &self,
        session: SessionId,
        reference: CognitiveRef,
        reason: &str,
        hold_until: Option<DateTime<Utc>>,
        state: &str,
    ) -> Result<()> {
        if !matches!(state, "resident" | "provisional") {
            return Err(Error::Invalid("invalid resident state".into()));
        }
        let (kind, value) = reference_parts(&reference);
        let now = Utc::now();
        sqlx::query("INSERT INTO resident_refs(session_id,ref_kind,ref_value,entered_at,entry_reason,hold_until,state,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,'{}') ON CONFLICT(session_id,ref_kind,ref_value) DO UPDATE SET state=excluded.state,entry_reason=excluded.entry_reason,hold_until=COALESCE(excluded.hold_until,resident_refs.hold_until)")
            .bind(session.0).bind(kind).bind(value).bind(now).bind(reason).bind(hold_until).bind(state)
            .execute(self.store.pool()).await.map_err(db)?;
        sqlx::query("UPDATE cognitive_sessions SET last_activity_at=$2,state_revision=state_revision+1 WHERE session_id=$1")
            .bind(session.0).bind(now).execute(self.store.pool()).await.map_err(db)?;
        Ok(())
    }

    pub(crate) async fn evict_if_needed(&self, session: SessionId) -> Result<()> {
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM resident_refs WHERE session_id=$1 AND state IN ('resident','provisional')",
        )
        .bind(session.0)
        .fetch_one(self.store.pool())
        .await
        .map_err(db)?;
        if count as usize <= self.resident_limit {
            return Ok(());
        }
        let excess = count as usize - self.resident_limit;
        sqlx::query("WITH victims AS (SELECT session_id,ref_kind,ref_value FROM resident_refs WHERE session_id=$1 AND state IN ('resident','provisional') AND (hold_until IS NULL OR hold_until < now()) ORDER BY (state='provisional') DESC,(last_meaningful_use_at IS NOT NULL) ASC,last_meaningful_use_at ASC NULLS FIRST,entered_at ASC LIMIT $2) UPDATE resident_refs r SET state='evicted' FROM victims v WHERE r.session_id=v.session_id AND r.ref_kind=v.ref_kind AND r.ref_value=v.ref_value")
            .bind(session.0).bind(excess as i64).execute(self.store.pool()).await.map_err(db)?;
        Ok(())
    }

    pub(crate) async fn resident_reference_strings(
        &self,
        session: SessionId,
    ) -> Result<HashSet<String>> {
        Ok(sqlx::query(
            "SELECT ref_kind,ref_value FROM resident_refs WHERE session_id=$1 AND state IN ('resident','provisional')",
        )
        .bind(session.0)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?
        .into_iter()
        .map(|row| {
            format!(
                "{}:{}",
                row.get::<String, _>("ref_kind"),
                row.get::<String, _>("ref_value")
            )
        })
        .collect())
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
        let found: i64 = sqlx::query_scalar("SELECT count(*) FROM entity_mentions m JOIN entity_binding_revisions b ON b.mention_id=m.mention_id WHERE (m.occurrence_id IN (SELECT occurrence_id FROM memory_revision_evidence WHERE memory_revision_id=$1 AND occurrence_id IS NOT NULL) OR m.source_region_id IN (SELECT source_region_id FROM memory_revision_evidence WHERE memory_revision_id=$1 AND source_region_id IS NOT NULL) OR m.derived_region_id IN (SELECT derived_region_id FROM memory_revision_evidence WHERE memory_revision_id=$1 AND derived_region_id IS NOT NULL)) AND b.revision_no=(SELECT max(b2.revision_no) FROM entity_binding_revisions b2 WHERE b2.mention_id=b.mention_id) AND b.entity_ref=ANY($2) AND b.binding_state='bound'")
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
        sqlx::query_scalar::<_, String>("SELECT DISTINCT o.source_class FROM memory_revision_evidence e JOIN observation_occurrences o ON o.occurrence_id=e.occurrence_id WHERE e.memory_revision_id=$1 UNION SELECT DISTINCT o.source_class FROM memory_revision_evidence e JOIN source_regions sr ON sr.source_region_id=e.source_region_id JOIN observation_occurrences o ON o.artifact_id=sr.artifact_id WHERE e.memory_revision_id=$1")
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
        sqlx::query_scalar::<_, String>("SELECT DISTINCT b.entity_ref FROM memory_revision_evidence e JOIN entity_mentions m ON (m.occurrence_id=e.occurrence_id OR m.source_region_id=e.source_region_id OR m.derived_region_id=e.derived_region_id) JOIN entity_binding_revisions b ON b.mention_id=m.mention_id WHERE e.memory_revision_id=$1 AND b.revision_no=(SELECT max(b2.revision_no) FROM entity_binding_revisions b2 WHERE b2.mention_id=b.mention_id) AND b.binding_state='bound' AND b.entity_ref IS NOT NULL ORDER BY b.entity_ref")
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
        let rows = sqlx::query("SELECT tag_id,ordinal FROM memory_revision_tags WHERE memory_revision_id=$1 ORDER BY ordinal NULLS LAST,tag_id")
            .bind(revision).fetch_all(self.store.pool()).await.map_err(db)?;
        let ordered = rows.len() >= 2
            && rows.iter().all(|row| {
                row.try_get::<Option<i32>, _>("ordinal")
                    .unwrap_or(None)
                    .is_some()
            });
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
                "memory_revision_tags ordinal/order_provenance".into()
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
    sqlx::query("INSERT INTO memory_revision_evidence(memory_revision_id,evidence_no,occurrence_id,source_region_id,derived_representation_id,derived_region_id,support_role,weight) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(revision.0).bind(evidence.evidence_no).bind(occurrence).bind(source).bind(derived).bind(derived_region)
        .bind(format!("{:?}",evidence.support_role).to_lowercase()).bind(evidence.weight)
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
        weight: row.try_get("weight").map_err(db)?,
    })
}

pub(crate) fn decode_resource(
    subject: SubjectId,
    row: sqlx::postgres::PgRow,
) -> Result<ResourceDescriptor> {
    Ok(ResourceDescriptor {
        subject_id: subject,
        resource_ref: ResourceRef::new(row.try_get::<String, _>("resource_ref").map_err(db)?)?,
        display_label: row.try_get("display_label").map_err(db)?,
        authority_class: row.try_get("authority_class").map_err(db)?,
        coverage: row.try_get("coverage").map_err(db)?,
        query_dimensions: row.try_get("query_dimensions").map_err(db)?,
        modalities: row.try_get("modalities").map_err(db)?,
        freshness_policy: row.try_get("freshness_policy").map_err(db)?,
        access_cost_class: row.try_get("access_cost_class").map_err(db)?,
        resolver_key: row.try_get("resolver_key").map_err(db)?,
        readiness: row.try_get("readiness").map_err(db)?,
        updated_at: row.try_get("updated_at").map_err(db)?,
    })
}

pub(crate) fn reference_parts(reference: &CognitiveRef) -> (String, String) {
    match reference {
        CognitiveRef::Memory(id) => ("memory".into(), id.0.to_string()),
        CognitiveRef::MemoryRevision(id) => ("memory_revision".into(), id.0.to_string()),
        CognitiveRef::Artifact(id) => ("artifact".into(), id.0.to_string()),
        CognitiveRef::SourceRegion(id) => ("source_region".into(), id.0.to_string()),
        CognitiveRef::DerivedRepresentation(id) => {
            ("derived_representation".into(), id.0.to_string())
        }
        CognitiveRef::DerivedRegion(id) => ("derived_region".into(), id.0.to_string()),
        CognitiveRef::Entity(id) => ("entity".into(), id.as_str().into()),
        CognitiveRef::Tag(id) => ("tag".into(), id.0.to_string()),
        CognitiveRef::Anchor(id) => ("anchor".into(), id.0.to_string()),
        CognitiveRef::Resource(id) => ("resource".into(), id.as_str().into()),
        CognitiveRef::ExternalObject(id) => ("external_object".into(), id.as_str().into()),
        CognitiveRef::Occurrence(id) => ("occurrence".into(), id.0.to_string()),
        CognitiveRef::Session(id) => ("session".into(), id.0.to_string()),
    }
}

pub(crate) fn parse_reference(kind: &str, value: &str) -> Result<CognitiveRef> {
    Ok(match kind {
        "memory" => CognitiveRef::Memory(MemoryId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid memory ref".into()))?,
        )),
        "memory_revision" => CognitiveRef::MemoryRevision(MemoryRevisionId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid memory revision ref".into()))?,
        )),
        "artifact" => CognitiveRef::Artifact(ArtifactId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid artifact ref".into()))?,
        )),
        "source_region" => CognitiveRef::SourceRegion(SourceRegionId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid source region ref".into()))?,
        )),
        "derived_representation" => CognitiveRef::DerivedRepresentation(DerivedRepresentationId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid derived ref".into()))?,
        )),
        "derived_region" => CognitiveRef::DerivedRegion(DerivedRegionId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid derived region ref".into()))?,
        )),
        "entity" => CognitiveRef::Entity(EntityRef::new(value)?),
        "tag" => CognitiveRef::Tag(TagId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid tag ref".into()))?,
        )),
        "anchor" => CognitiveRef::Anchor(AnchorId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid anchor ref".into()))?,
        )),
        "resource" => CognitiveRef::Resource(ResourceRef::new(value)?),
        "external_object" => CognitiveRef::ExternalObject(ObjectRef::new(value)?),
        "occurrence" => CognitiveRef::Occurrence(OccurrenceId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid occurrence ref".into()))?,
        )),
        "session" => CognitiveRef::Session(SessionId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid session ref".into()))?,
        )),
        _ => return Err(Error::Invalid("unknown reference kind".into())),
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
pub(crate) fn parse_supersession(value: &str) -> Result<SupersessionState> {
    match value {
        "current" => Ok(SupersessionState::Current),
        "superseded" => Ok(SupersessionState::Superseded),
        "contradicted" => Ok(SupersessionState::Contradicted),
        "revoked" => Ok(SupersessionState::Revoked),
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
pub(crate) fn interval_contains(interval: Option<TimeInterval>, value: DateTime<Utc>) -> bool {
    interval.is_none_or(|interval| {
        interval.start.is_none_or(|start| value >= start)
            && interval.end.is_none_or(|end| value <= end)
    })
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
pub(crate) fn wave_node_kind(reference: &CognitiveRef) -> WaveNodeKind {
    match reference {
        CognitiveRef::Memory(_) | CognitiveRef::MemoryRevision(_) => WaveNodeKind::Memory,
        CognitiveRef::Tag(_) => WaveNodeKind::Tag,
        CognitiveRef::Anchor(_) => WaveNodeKind::Anchor,
        CognitiveRef::Entity(_) => WaveNodeKind::Entity,
        CognitiveRef::Resource(_) => WaveNodeKind::Resource,
        _ => WaveNodeKind::Memory,
    }
}
pub(crate) fn wave_reference_allowed(reference: &CognitiveRef) -> bool {
    matches!(
        reference,
        CognitiveRef::Memory(_)
            | CognitiveRef::Tag(_)
            | CognitiveRef::Anchor(_)
            | CognitiveRef::Entity(_)
            | CognitiveRef::Resource(_)
    )
}
pub(crate) fn topology_edge(
    from: &CognitiveRef,
    to: &CognitiveRef,
    support_class: &str,
    association_kind: &str,
) -> WaveEdgeEvidence {
    WaveEdgeEvidence {
        from: from.clone(),
        to: to.clone(),
        support_class: support_class.into(),
        association_kind: association_kind.into(),
        polarity: "positive".into(),
        support_value: 1.0,
        bridge_hint: false,
    }
}
pub(crate) fn db(error: sqlx::Error) -> Error {
    Error::Infrastructure(error.to_string())
}
