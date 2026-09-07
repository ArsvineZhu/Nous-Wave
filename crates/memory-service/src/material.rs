use super::support::*;
use super::*;

impl LocalRuntime {
    pub async fn observe(&self, input: ObservationInput) -> Result<AcceptedObservation> {
        self.require_subject(input.subject).await?;
        if let Some(session) = input.session {
            self.require_session(input.subject, session).await?;
        }
        let now = Utc::now();
        let object_guard = if matches!(
            &input.material,
            ObservationMaterial::InlineText { .. } | ObservationMaterial::StructuredJson { .. }
        ) {
            Some(self.objects.reference_guard(false).await?)
        } else {
            None
        };
        let (artifact, _) = self
            .material_to_artifact(input.subject, &input.material, now)
            .await?;
        if let Some(artifact) = &artifact {
            artifact.validate()?;
        }
        let occurrence_id = OccurrenceId::new();
        let mut occurrence = ObservationOccurrence {
            occurrence_id,
            subject_id: input.subject,
            artifact_id: artifact.as_ref().map(|artifact| artifact.artifact_id),
            source_class: input.occurrence.source_class.clone(),
            external_object_ref: input.occurrence.external_object_ref.clone().or_else(|| {
                match &input.material {
                    ObservationMaterial::ExternalObjectRef { object_ref } => {
                        Some(object_ref.clone())
                    }
                    _ => None,
                }
            }),
            occurred_at: input.occurrence.occurred_at,
            observed_at: input.occurrence.observed_at,
            conversation_ref: input.occurrence.conversation_ref.clone(),
            actor_entity_ref: input.occurrence.actor_entity_ref.clone(),
            context: input.occurrence.context.clone(),
            created_at: now,
        };
        occurrence.validate()?;
        if let Some(entity) = &occurrence.actor_entity_ref {
            EntityRef::new(entity.as_str())?;
        }
        if let Some(object) = &occurrence.external_object_ref {
            ObjectRef::new(object.as_str())?;
        }
        let mut tx = self.store.begin().await?;
        let artifact = if let Some(mut artifact) = artifact {
            let existing = sqlx::query("SELECT artifact_id,byte_length,storage_key,created_at,metadata FROM artifacts WHERE subject_id=$1 AND content_hash=$2")
                .bind(input.subject.0).bind(&artifact.content_hash).fetch_optional(&mut *tx).await.map_err(db)?;
            if let Some(row) = existing {
                artifact.artifact_id = ArtifactId(row.try_get("artifact_id").map_err(db)?);
                artifact.byte_length = row.try_get::<i64, _>("byte_length").map_err(db)? as u64;
                artifact.storage_key = row.try_get("storage_key").map_err(db)?;
                artifact.created_at = row.try_get("created_at").map_err(db)?;
                artifact.metadata = row.try_get("metadata").map_err(db)?;
            } else {
                sqlx::query("INSERT INTO artifacts(artifact_id,subject_id,content_hash,byte_length,media_type,storage_key,created_at,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
                    .bind(artifact.artifact_id.0).bind(artifact.subject_id.0).bind(&artifact.content_hash).bind(artifact.byte_length as i64).bind(&artifact.media_type).bind(&artifact.storage_key).bind(artifact.created_at).bind(&artifact.metadata).execute(&mut *tx).await.map_err(db)?;
            }
            Some(artifact)
        } else {
            None
        };
        occurrence.artifact_id = artifact.as_ref().map(|artifact| artifact.artifact_id);
        sqlx::query("INSERT INTO observation_occurrences(occurrence_id,subject_id,artifact_id,source_class,external_object_ref,occurred_at,observed_at,conversation_ref,actor_entity_ref,context,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)")
            .bind(occurrence.occurrence_id.0).bind(occurrence.subject_id.0).bind(occurrence.artifact_id.map(|id| id.0)).bind(occurrence.source_class.as_str()).bind(occurrence.external_object_ref.as_ref().map(ObjectRef::as_str)).bind(occurrence.occurred_at).bind(occurrence.observed_at).bind(&occurrence.conversation_ref).bind(occurrence.actor_entity_ref.as_ref().map(EntityRef::as_str)).bind(&occurrence.context).bind(occurrence.created_at).execute(&mut *tx).await.map_err(db)?;
        let source_region = if let Some(artifact) = &artifact {
            let coordinate = serde_json::json!({});
            let coordinate_hash = blake3::hash(
                serde_json::to_string(&coordinate)
                    .unwrap_or_default()
                    .as_bytes(),
            )
            .to_hex()
            .to_string();
            let existing = sqlx::query("SELECT source_region_id,created_at FROM source_regions WHERE subject_id=$1 AND artifact_id=$2 AND coordinate_kind='whole_artifact' AND coordinate_hash=$3")
                .bind(input.subject.0).bind(artifact.artifact_id.0).bind(&coordinate_hash).fetch_optional(&mut *tx).await.map_err(db)?;
            let (id, region_created_at) = if let Some(row) = existing {
                (
                    SourceRegionId(row.try_get("source_region_id").map_err(db)?),
                    row.try_get("created_at").map_err(db)?,
                )
            } else {
                let id = SourceRegionId::new();
                sqlx::query("INSERT INTO source_regions(source_region_id,subject_id,artifact_id,coordinate_kind,coordinate,coordinate_hash,created_at) VALUES($1,$2,$3,'whole_artifact',$4,$5,$6)")
                    .bind(id.0).bind(input.subject.0).bind(artifact.artifact_id.0).bind(&coordinate).bind(&coordinate_hash).bind(now).execute(&mut *tx).await.map_err(db)?;
                (id, now)
            };
            let region = SourceRegion {
                source_region_id: id,
                subject_id: input.subject,
                artifact_id: artifact.artifact_id,
                coordinate_kind: "whole_artifact".into(),
                coordinate,
                coordinate_hash,
                parent_source_region_id: None,
                created_at: region_created_at,
            };
            region.validate()?;
            Some(region)
        } else {
            None
        };
        let mut mention_ids = Vec::new();
        for mention in &input.entities {
            if mention.surface.trim().is_empty() {
                return Err(Error::Invalid("entity mention surface is required".into()));
            }
            if let Some(entity) = &mention.entity_ref {
                EntityRef::new(entity.as_str())?;
            }
            let mention_id = Uuid::now_v7();
            mention_ids.push(mention_id);
            sqlx::query("INSERT INTO entity_mentions(mention_id,subject_id,occurrence_id,surface,semantic_role,created_at) VALUES($1,$2,$3,$4,$5,$6)")
                .bind(mention_id).bind(input.subject.0).bind(occurrence_id.0).bind(&mention.surface).bind(&mention.semantic_role).bind(now).execute(&mut *tx).await.map_err(db)?;
            sqlx::query("INSERT INTO entity_binding_revisions(binding_revision_id,mention_id,revision_no,entity_ref,binding_state,created_at) VALUES($1,$2,1,$3,$4,$5)")
                .bind(Uuid::now_v7()).bind(mention_id).bind(mention.entity_ref.as_ref().map(EntityRef::as_str)).bind(if mention.entity_ref.is_some(){"bound"}else{"unbound"}).bind(now).execute(&mut *tx).await.map_err(db)?;
        }
        if let Some(region) = &source_region {
            sqlx::query("INSERT INTO coverage_needs(coverage_need_id,subject_id,source_region_id,representation_kind,capability_operation,requirement,state,updated_at) VALUES($1,$2,$3,'extracted_text','document_extraction','preferred',$4,$5) ON CONFLICT(subject_id,source_region_id,representation_kind,capability_operation) DO UPDATE SET updated_at=excluded.updated_at")
                .bind(Uuid::now_v7()).bind(input.subject.0).bind(region.source_region_id.0).bind(if matches!(input.material, ObservationMaterial::InlineText{..}){"ready"}else{"missing"}).bind(now).execute(&mut *tx).await.map_err(db)?;
        }
        tx.commit().await.map_err(db)?;
        drop(object_guard);
        let mut memory_revisions = Vec::new();
        if matches!(input.formation, FormationDirective::ExplicitSpecific) {
            let text = match &input.material {
                ObservationMaterial::InlineText { text, .. } => text.clone(),
                _ => {
                    return Err(Error::Invalid(
                        "explicit formation requires inline text or a supplied representation"
                            .into(),
                    ));
                }
            };
            let evidence = vec![MemoryRevisionEvidence {
                evidence_no: 0,
                evidence: EvidenceRef::Occurrence { occurrence_id },
                support_role: SupportRole::Direct,
                weight: Some(1.0),
            }];
            let view = self
                .form_memory(ExplicitMemoryInput {
                    subject: input.subject,
                    memory_class: MemoryClass::Specific,
                    semantic_role: "episode".into(),
                    representation_text: text,
                    title: None,
                    evidence,
                    entity_refs: input
                        .entities
                        .iter()
                        .filter_map(|mention| mention.entity_ref.clone())
                        .collect(),
                    tags: Vec::new(),
                    occurred_at: input.occurrence.occurred_at,
                    observed_at: input.occurrence.observed_at,
                    valid_from: None,
                    valid_to: None,
                    epistemic_class: EpistemicClass::Reported,
                    confidence: None,
                })
                .await?;
            memory_revisions.push(view.revision.memory_revision_id);
        } else if matches!(input.formation, FormationDirective::ConsiderSpecific)
            && let Some(provider) = &self.memory_formation_provider
        {
            let text = match &input.material {
                ObservationMaterial::InlineText { text, .. } => Some(text.clone()),
                ObservationMaterial::ArtifactRef { artifact_id } => {
                    let artifact = self.artifact(input.subject, *artifact_id).await?;
                    self.objects
                        .get(&artifact.content_hash)
                        .await
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                }
                _ => None,
            };
            if let Some(text) = text.filter(|text| !text.trim().is_empty()) {
                let proposals = provider
                    .propose(MemoryFormationRequest {
                        subject: input.subject,
                        occurrence: occurrence_id,
                        source_class: input.occurrence.source_class.clone(),
                        text,
                        entity_refs: input
                            .entities
                            .iter()
                            .filter_map(|mention| mention.entity_ref.clone())
                            .collect(),
                        occurred_at: input.occurrence.occurred_at,
                        observed_at: input.occurrence.observed_at,
                    })
                    .await?;
                if proposals.len() > 32 {
                    return Err(Error::Invalid(
                        "memory formation proposal count exceeds the request bound".into(),
                    ));
                }
                let allowed_entities = input
                    .entities
                    .iter()
                    .filter_map(|mention| mention.entity_ref.clone())
                    .collect::<Vec<_>>();
                let mut proposal_keys = HashSet::new();
                for proposal in proposals {
                    let normalized = proposal
                        .representation_text
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .to_lowercase();
                    let evidence_key = serde_json::to_string(&proposal.evidence)
                        .map_err(|error| Error::Invalid(error.to_string()))?;
                    if !proposal_keys.insert(format!("{normalized}\0{evidence_key}")) {
                        continue;
                    }
                    let view = self
                        .commit_formation_proposal(input.subject, proposal, &allowed_entities)
                        .await?;
                    memory_revisions.push(view.revision.memory_revision_id);
                }
            }
        }
        if let Some(session) = input.session.filter(|_| input.runtime.admit) {
            self.admit(
                session,
                CognitiveRef::Occurrence(occurrence_id),
                "observed",
                input.runtime.hold_until,
            )
            .await?;
            for mention in &input.entities {
                if let Some(entity) = &mention.entity_ref {
                    self.admit(
                        session,
                        CognitiveRef::Entity(entity.clone()),
                        "observed",
                        input.runtime.hold_until,
                    )
                    .await?;
                }
            }
            if let ObservationMaterial::ResourceAvailability { resource } = &input.material {
                ResourceRef::new(resource.as_str())?;
                self.admit(
                    session,
                    CognitiveRef::Resource(resource.clone()),
                    "resource_awareness",
                    input.runtime.hold_until,
                )
                .await?;
            }
            for revision in &memory_revisions {
                self.admit(
                    session,
                    CognitiveRef::MemoryRevision(*revision),
                    "observed",
                    input.runtime.hold_until,
                )
                .await?;
            }
            self.evict_if_needed(session).await?;
        }
        Ok(AcceptedObservation {
            artifact,
            occurrence,
            source_region,
            mention_ids,
            resident: input.session.is_some() && input.runtime.admit,
            memory_revisions,
        })
    }

    async fn material_to_artifact(
        &self,
        subject: SubjectId,
        material: &ObservationMaterial,
        now: DateTime<Utc>,
    ) -> Result<(Option<Artifact>, Option<Vec<u8>>)> {
        match material {
            ObservationMaterial::InlineText { text, media_type } => {
                let bytes = text.as_bytes().to_vec();
                self.bytes_to_artifact(subject, bytes, media_type, serde_json::json!({}), now)
                    .await
            }
            ObservationMaterial::StructuredJson { value } => {
                let bytes =
                    serde_json::to_vec(value).map_err(|error| Error::Invalid(error.to_string()))?;
                self.bytes_to_artifact(
                    subject,
                    bytes,
                    "application/json",
                    serde_json::json!({}),
                    now,
                )
                .await
            }
            ObservationMaterial::ArtifactRef { artifact_id } => {
                let artifact = self.artifact(subject, *artifact_id).await?;
                Ok((Some(artifact), None))
            }
            ObservationMaterial::ExternalObjectRef { .. }
            | ObservationMaterial::ResourceAvailability { .. } => Ok((None, None)),
        }
    }

    async fn bytes_to_artifact(
        &self,
        subject: SubjectId,
        bytes: Vec<u8>,
        media_type: &str,
        metadata: serde_json::Value,
        now: DateTime<Utc>,
    ) -> Result<(Option<Artifact>, Option<Vec<u8>>)> {
        if bytes.len() as u64 > self.max_upload_bytes {
            return Err(Error::Invalid(
                "material exceeds configured byte bound".into(),
            ));
        }
        let (hash, size) = self
            .objects
            .put_chunks(
                futures::stream::iter([Ok::<Vec<u8>, Error>(bytes.clone())]),
                self.max_upload_bytes,
            )
            .await?;
        let artifact = Artifact {
            artifact_id: ArtifactId::new(),
            subject_id: subject,
            content_hash: hash.clone(),
            byte_length: size,
            media_type: media_type.to_owned(),
            storage_key: hash,
            created_at: now,
            metadata,
        };
        Ok((Some(artifact), Some(bytes)))
    }

    pub async fn ingest_stream<S>(
        &self,
        subject: SubjectId,
        metadata: UploadMetadata,
        chunks: S,
    ) -> Result<AcceptedObservation>
    where
        S: Stream<Item = Result<Vec<u8>>> + Send,
    {
        self.require_subject(subject).await?;
        let guard = self.objects.reference_guard(false).await?;
        let (hash, size) = self
            .objects
            .put_chunks(chunks, self.max_upload_bytes)
            .await?;
        let now = Utc::now();
        if !self.store.subject_exists(subject).await? {
            drop(guard);
            return Err(Error::NotFound("subject not found".into()));
        }
        let proposed_artifact_id = ArtifactId::new();
        sqlx::query("INSERT INTO artifacts(artifact_id,subject_id,content_hash,byte_length,media_type,storage_key,created_at,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT(subject_id,content_hash) DO NOTHING")
            .bind(proposed_artifact_id.0).bind(subject.0).bind(&hash).bind(size as i64).bind(&metadata.media_type).bind(&hash).bind(now).bind(&metadata.metadata)
            .execute(self.store.pool()).await.map_err(db)?;
        let artifact_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT artifact_id FROM artifacts WHERE subject_id=$1 AND content_hash=$2",
        )
        .bind(subject.0)
        .bind(&hash)
        .fetch_one(self.store.pool())
        .await
        .map_err(db)?;
        let artifact = self.artifact(subject, ArtifactId(artifact_id)).await?;
        let observed_at = metadata.observed_at;
        let result = self
            .observe(ObservationInput {
                subject,
                session: None,
                occurrence: OccurrenceDescriptor {
                    source_class: metadata.source_class,
                    external_object_ref: metadata.external_object_ref,
                    occurred_at: metadata.occurred_at,
                    observed_at,
                    conversation_ref: metadata.conversation_ref,
                    actor_entity_ref: metadata.actor_entity_ref,
                    context: serde_json::json!({}),
                },
                material: ObservationMaterial::ArtifactRef {
                    artifact_id: artifact.artifact_id,
                },
                entities: Vec::new(),
                formation: FormationDirective::None,
                runtime: RuntimeDirective::default(),
            })
            .await;
        drop(guard);
        result
    }

    pub async fn artifact(&self, subject: SubjectId, artifact: ArtifactId) -> Result<Artifact> {
        let row = sqlx::query("SELECT artifact_id,subject_id,content_hash,byte_length,media_type,storage_key,created_at,metadata FROM artifacts WHERE subject_id=$1 AND artifact_id=$2")
            .bind(subject.0).bind(artifact.0).fetch_optional(self.store.pool()).await.map_err(db)?.ok_or_else(||Error::NotFound("artifact not found".into()))?;
        Ok(Artifact {
            artifact_id: ArtifactId(row.try_get("artifact_id").map_err(db)?),
            subject_id: SubjectId(row.try_get("subject_id").map_err(db)?),
            content_hash: row.try_get("content_hash").map_err(db)?,
            byte_length: row.try_get::<i64, _>("byte_length").map_err(db)? as u64,
            media_type: row.try_get("media_type").map_err(db)?,
            storage_key: row.try_get("storage_key").map_err(db)?,
            created_at: row.try_get("created_at").map_err(db)?,
            metadata: row.try_get("metadata").map_err(db)?,
        })
    }

    pub async fn artifact_stream(
        &self,
        subject: SubjectId,
        artifact: ArtifactId,
    ) -> Result<(Artifact, impl Stream<Item = Result<Vec<u8>>> + Send + use<>)> {
        let artifact = self.artifact(subject, artifact).await?;
        let stream = self.objects.stream(&artifact.content_hash).await?;
        Ok((artifact, stream))
    }
}
