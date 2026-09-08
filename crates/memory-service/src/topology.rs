use super::support::*;
use super::*;
use nous_authority_store::ProjectionInvalidation;

impl MemoryService {
    pub async fn create_tag(&self, subject: SubjectId, input: CreateTagRequest) -> Result<Tag> {
        self.require_subject(subject).await?;
        if input.label.trim().is_empty() {
            return Err(Error::Invalid("tag label is required".into()));
        }
        let tag_id = TagId::new();
        let revision_id = Uuid::now_v7();
        let now = Utc::now();
        let mut tx = self.store.begin().await?;
        sqlx::query("INSERT INTO tags(tag_id,subject_id,current_revision_id,created_at,status) VALUES($1,$2,$3,$4,'active')")
            .bind(tag_id.0)
            .bind(subject.0)
            .bind(revision_id)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("INSERT INTO tag_revisions(tag_revision_id,tag_id,revision_no,label,description,kind_hint,origin,created_at) VALUES($1,$2,1,$3,$4,$5,$6,$7)")
            .bind(revision_id)
            .bind(tag_id.0)
            .bind(input.label)
            .bind(input.description)
            .bind(input.kind_hint)
            .bind(input.origin)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        tx.commit().await.map_err(db)?;
        self.store
            .invalidate(
                subject,
                ProjectionInvalidation {
                    topology: true,
                    ..ProjectionInvalidation::text()
                },
            )
            .await?;
        Ok(Tag {
            tag_id,
            subject_id: subject,
            current_revision_id: revision_id,
            status: TopologyStatus::Active,
            created_at: now,
        })
    }

    pub async fn create_anchor(
        &self,
        subject: SubjectId,
        input: CreateAnchorRequest,
    ) -> Result<Anchor> {
        self.require_subject(subject).await?;
        if input.description.trim().is_empty() {
            return Err(Error::Invalid("anchor description is required".into()));
        }
        if input.origin == "consolidation" && !input.confirmed {
            let mut roots = HashSet::new();
            for support in &input.supports {
                let root = match &support.reference {
                    CognitiveRef::Memory(memory) => Some(format!("memory:{}", memory.0)),
                    CognitiveRef::MemoryRevision(revision) => sqlx::query_scalar::<_, Uuid>(
                        "SELECT memory_id FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$2",
                    )
                    .bind(subject.0)
                    .bind(revision.0)
                    .fetch_optional(self.store.pool())
                    .await
                    .map_err(db)?
                    .map(|memory| format!("memory:{memory}")),
                    CognitiveRef::Occurrence(occurrence) => {
                        Some(format!("occurrence:{}", occurrence.0))
                    }
                    CognitiveRef::SourceRegion(region) => {
                        Some(format!("source_region:{}", region.0))
                    }
                    CognitiveRef::DerivedRepresentation(representation) => Some(format!(
                        "derived_representation:{}",
                        representation.0
                    )),
                    CognitiveRef::DerivedRegion(region) => {
                        Some(format!("derived_region:{}", region.0))
                    }
                    _ => None,
                };
                if let Some(root) = root {
                    roots.insert(root);
                }
            }
            if roots.len() < 2 {
                return Err(Error::Invalid("consolidation-derived Anchor needs two independent support roots or explicit confirmation".into()));
            }
        }
        for support in &input.supports {
            if !self
                .reference_in_subject(subject, &support.reference)
                .await?
            {
                return Err(Error::Invalid(
                    "Anchor support is outside Subject Authority".into(),
                ));
            }
        }
        let anchor_id = AnchorId::new();
        let revision_id = Uuid::now_v7();
        let now = Utc::now();
        let mut tx = self.store.begin().await?;
        sqlx::query("INSERT INTO anchors(anchor_id,subject_id,current_revision_id,created_at,status) VALUES($1,$2,$3,$4,'active')")
            .bind(anchor_id.0).bind(subject.0).bind(revision_id).bind(now).execute(&mut *tx).await.map_err(db)?;
        sqlx::query("INSERT INTO anchor_revisions(anchor_revision_id,anchor_id,revision_no,label,description,origin,created_at) VALUES($1,$2,1,$3,$4,$5,$6)")
            .bind(revision_id).bind(anchor_id.0).bind(input.label).bind(input.description).bind(input.origin).bind(now).execute(&mut *tx).await.map_err(db)?;
        for support in input.supports {
            let (kind, value) = reference_parts(&support.reference);
            sqlx::query("INSERT INTO anchor_support(anchor_revision_id,support_ref_kind,support_ref,support_role,provenance) VALUES($1,$2,$3,$4,'{}')")
                .bind(revision_id).bind(kind).bind(value).bind(support.role).execute(&mut *tx).await.map_err(db)?;
        }
        tx.commit().await.map_err(db)?;
        self.store
            .invalidate(
                subject,
                ProjectionInvalidation {
                    topology: true,
                    ..ProjectionInvalidation::text()
                },
            )
            .await?;
        Ok(Anchor {
            anchor_id,
            subject_id: subject,
            current_revision_id: revision_id,
            status: TopologyStatus::Active,
            created_at: now,
        })
    }

    pub async fn create_association(
        &self,
        subject: SubjectId,
        input: CreateAssociationRequest,
    ) -> Result<AssociationEvidence> {
        self.require_subject(subject).await?;
        if input.support_value < 0.0 || !input.support_value.is_finite() {
            return Err(Error::Invalid(
                "association support_value must be finite and non-negative".into(),
            ));
        }
        if input.support_class == AssociationSupportClass::MeaningfulUse
            && input.occurrence_id.is_none()
            && input.memory_revision_id.is_none()
        {
            return Err(Error::Invalid(
                "meaningful-use association needs an evidence reference".into(),
            ));
        }
        if input.support_class == AssociationSupportClass::MeaningfulUse {
            let Some(meaningful_ref) = input
                .memory_revision_id
                .map(|revision| ("memory_revision", revision.0.to_string()))
                .or_else(|| {
                    input
                        .occurrence_id
                        .map(|occurrence| ("occurrence", occurrence.0.to_string()))
                })
            else {
                return Err(Error::Invalid(
                    "meaningful-use association needs an evidence reference".into(),
                ));
            };
            let valid_use: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM cognitive_use_events WHERE subject_id=$1 AND ((ref_kind=$2 AND ref_value=$3) OR ($2='memory_revision' AND ref_kind='memory' AND ref_value=(SELECT memory_id::text FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$3::uuid))) AND use_kind IN ('referenced','acted_on','corroborated'))",
            )
            .bind(subject.0)
            .bind(meaningful_ref.0)
            .bind(meaningful_ref.1)
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
            if !valid_use {
                return Err(Error::Invalid(
                    "meaningful-use association needs a referenced, acted_on, or corroborated use event".into(),
                ));
            }
        }
        if !self.reference_in_subject(subject, &input.from).await?
            || !self.reference_in_subject(subject, &input.to).await?
        {
            return Err(Error::Invalid(
                "association endpoint is outside Subject Authority".into(),
            ));
        }
        if let Some(occurrence) = input.occurrence_id {
            let valid: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM observation_occurrences WHERE subject_id=$1 AND occurrence_id=$2)",
            )
            .bind(subject.0)
            .bind(occurrence.0)
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
            if !valid {
                return Err(Error::Invalid(
                    "association occurrence is outside Subject".into(),
                ));
            }
        }
        if let Some(revision) = input.memory_revision_id {
            let valid: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$2)",
            )
            .bind(subject.0)
            .bind(revision.0)
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
            if !valid {
                return Err(Error::Invalid(
                    "association memory revision is outside Subject".into(),
                ));
            }
        }
        let (from_kind, from_ref) = reference_parts(&input.from);
        let (to_kind, to_ref) = reference_parts(&input.to);
        let id = Uuid::now_v7();
        let now = Utc::now();
        sqlx::query("INSERT INTO association_evidence(association_evidence_id,subject_id,from_ref_kind,from_ref,to_ref_kind,to_ref,association_kind,polarity,support_class,support_value,occurrence_id,memory_revision_id,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)")
            .bind(id).bind(subject.0).bind(&from_kind).bind(&from_ref).bind(&to_kind).bind(&to_ref).bind(&input.association_kind).bind(input.polarity.as_str()).bind(input.support_class.as_str()).bind(input.support_value).bind(input.occurrence_id.map(|id| id.0)).bind(input.memory_revision_id.map(|id| id.0)).bind(now).execute(self.store.pool()).await.map_err(db)?;
        let result = AssociationEvidence {
            association_evidence_id: id,
            subject_id: subject,
            from_ref_kind: from_kind,
            from_ref,
            to_ref_kind: to_kind,
            to_ref,
            association_kind: input.association_kind,
            polarity: input.polarity,
            support_class: input.support_class,
            support_value: input.support_value,
            occurrence_id: input.occurrence_id,
            memory_revision_id: input.memory_revision_id,
            producer_signature_id: None,
            valid_from: None,
            valid_to: None,
            created_at: now,
            revoked_at: None,
        };
        self.store
            .invalidate(subject, ProjectionInvalidation::topology())
            .await?;
        Ok(result)
    }

    pub async fn rebind_entity(
        &self,
        subject: SubjectId,
        input: RebindEntityRequest,
    ) -> Result<()> {
        if !matches!(
            input.binding_state.as_str(),
            "bound" | "unbound" | "disputed"
        ) {
            return Err(Error::Invalid("invalid entity binding state".into()));
        }
        let mention_subject: Option<Uuid> =
            sqlx::query_scalar("SELECT subject_id FROM entity_mentions WHERE mention_id=$1")
                .bind(input.mention_id)
                .fetch_optional(self.store.pool())
                .await
                .map_err(db)?;
        if mention_subject != Some(subject.0) {
            return Err(Error::NotFound("entity mention not found".into()));
        }
        if let Some(entity) = &input.entity_ref {
            EntityRef::new(entity.as_str())?;
        }
        if input.binding_state == "bound" && input.entity_ref.is_none() {
            return Err(Error::Invalid(
                "bound entity binding requires an EntityRef".into(),
            ));
        }
        if input.binding_state != "bound" && input.entity_ref.is_some() {
            return Err(Error::Invalid(
                "unbound/disputed binding cannot carry an EntityRef".into(),
            ));
        }
        let next: i32 = sqlx::query_scalar("SELECT COALESCE(max(revision_no),0)+1 FROM entity_binding_revisions WHERE mention_id=$1")
            .bind(input.mention_id).fetch_one(self.store.pool()).await.map_err(db)?;
        sqlx::query("INSERT INTO entity_binding_revisions(binding_revision_id,mention_id,revision_no,entity_ref,binding_state,host_resolution_ref,reason,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(Uuid::now_v7()).bind(input.mention_id).bind(next).bind(input.entity_ref.as_ref().map(EntityRef::as_str)).bind(input.binding_state).bind(input.host_resolution_ref).bind(input.reason).bind(Utc::now()).execute(self.store.pool()).await.map_err(db)?;
        self.store
            .invalidate(subject, ProjectionInvalidation::identity())
            .await?;
        Ok(())
    }
}
