use super::support::*;
use super::*;

impl LocalRuntime {
    pub async fn use_feedback(&self, input: UseFeedback) -> Result<()> {
        self.require_subject(input.subject).await?;
        if let Some(session) = input.session_id {
            self.require_session(input.subject, session).await?;
        }
        for event in input.events {
            if !self
                .reference_in_subject(input.subject, &event.reference)
                .await?
            {
                return Err(Error::Invalid(
                    "use feedback reference is outside Subject".into(),
                ));
            }
            let (kind, value) = reference_parts(&event.reference);
            let now = Utc::now();
            sqlx::query("INSERT INTO cognitive_use_events(use_event_id,subject_id,session_id,ref_kind,ref_value,use_kind,consumer_ref,occurred_at,context) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)")
                .bind(Uuid::now_v7()).bind(input.subject.0).bind(input.session_id.map(|id|id.0)).bind(&kind).bind(&value).bind(event.use_kind.as_str()).bind(&input.consumer).bind(now).bind(event.context).execute(self.store.pool()).await.map_err(db)?;
            if let Some(session) = input.session_id.filter(|_| event.use_kind.meaningful()) {
                let updated = sqlx::query("UPDATE resident_refs SET last_meaningful_use_at=$4,state='resident' WHERE session_id=$1 AND ref_kind=$2 AND ref_value=$3")
                    .bind(session.0).bind(&kind).bind(&value).bind(now).execute(self.store.pool()).await.map_err(db)?;
                if updated.rows_affected() == 0 {
                    self.admit_with_state(
                        session,
                        event.reference.clone(),
                        "recalled",
                        None,
                        "provisional",
                    )
                    .await?;
                    sqlx::query("UPDATE resident_refs SET last_meaningful_use_at=$4 WHERE session_id=$1 AND ref_kind=$2 AND ref_value=$3")
                        .bind(session.0)
                        .bind(&kind)
                        .bind(&value)
                        .bind(now)
                        .execute(self.store.pool())
                        .await
                        .map_err(db)?;
                }
                sqlx::query("UPDATE cognitive_sessions SET last_activity_at=$2,last_meaningful_use_at=$2,state_revision=state_revision+1 WHERE session_id=$1").bind(session.0).bind(now).execute(self.store.pool()).await.map_err(db)?;
            }
        }
        Ok(())
    }

    pub async fn upsert_resource(
        &self,
        subject: SubjectId,
        input: ResourceUpsert,
    ) -> Result<ResourceView> {
        self.require_subject(subject).await?;
        ResourceRef::new(input.resource_ref.as_str())?;
        if input.resolver_key.trim().is_empty() || input.authority_class.trim().is_empty() {
            return Err(Error::Invalid(
                "resource resolver_key/authority_class is required".into(),
            ));
        }
        if !matches!(
            input.readiness.as_str(),
            "ready" | "degraded" | "unavailable"
        ) {
            return Err(Error::Invalid("invalid resource readiness".into()));
        }
        sqlx::query("INSERT INTO resources(subject_id,resource_ref,display_label,authority_class,coverage,query_dimensions,modalities,freshness_policy,access_cost_class,resolver_key,readiness,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) ON CONFLICT(subject_id,resource_ref) DO UPDATE SET display_label=excluded.display_label,authority_class=excluded.authority_class,coverage=excluded.coverage,query_dimensions=excluded.query_dimensions,modalities=excluded.modalities,freshness_policy=excluded.freshness_policy,access_cost_class=excluded.access_cost_class,resolver_key=excluded.resolver_key,readiness=excluded.readiness,updated_at=excluded.updated_at")
            .bind(subject.0).bind(input.resource_ref.as_str()).bind(input.display_label).bind(input.authority_class).bind(input.coverage).bind(input.query_dimensions).bind(input.modalities).bind(input.freshness_policy).bind(input.access_cost_class).bind(input.resolver_key).bind(input.readiness).bind(Utc::now()).execute(self.store.pool()).await.map_err(db)?;
        let row=sqlx::query("SELECT resource_ref,display_label,authority_class,coverage,query_dimensions,modalities,freshness_policy,access_cost_class,resolver_key,readiness,updated_at FROM resources WHERE subject_id=$1 AND resource_ref=$2").bind(subject.0).bind(input.resource_ref.as_str()).fetch_one(self.store.pool()).await.map_err(db)?;
        self.rebuild_projection(subject).await?;
        Ok(ResourceView {
            descriptor: decode_resource(subject, row)?,
        })
    }

    pub async fn delete_resource(&self, subject: SubjectId, resource: ResourceRef) -> Result<()> {
        sqlx::query("DELETE FROM resources WHERE subject_id=$1 AND resource_ref=$2")
            .bind(subject.0)
            .bind(resource.as_str())
            .execute(self.store.pool())
            .await
            .map_err(db)?;
        sqlx::query("DELETE FROM resident_refs WHERE ref_kind='resource' AND ref_value=$1")
            .bind(resource.as_str())
            .execute(self.store.pool())
            .await
            .map_err(db)?;
        self.rebuild_projection(subject).await?;
        Ok(())
    }

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
        self.rebuild_projection(subject).await?;
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
        self.rebuild_projection(subject).await?;
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
        self.rebuild_projection(subject).await?;
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
        self.rebuild_projection(subject).await?;
        Ok(())
    }

    pub async fn list_resources(&self, subject: SubjectId) -> Result<Vec<ResourceView>> {
        let rows=sqlx::query("SELECT resource_ref,display_label,authority_class,coverage,query_dimensions,modalities,freshness_policy,access_cost_class,resolver_key,readiness,updated_at FROM resources WHERE subject_id=$1 ORDER BY resource_ref").bind(subject.0).fetch_all(self.store.pool()).await.map_err(db)?;
        rows.into_iter()
            .map(|row| {
                Ok(ResourceView {
                    descriptor: decode_resource(subject, row)?,
                })
            })
            .collect()
    }

    pub(crate) async fn resource_actions_for_query(
        &self,
        query: &CognitiveQuery,
    ) -> Result<(Vec<ResourceActionSuggestion>, Vec<Degradation>)> {
        if query.resources.current_authority == CurrentAuthorityNeed::None {
            return Ok((Vec::new(), Vec::new()));
        }
        let requested = query
            .cues
            .iter()
            .filter_map(|cue| match cue {
                Cue::Resource(resource) => Some(resource.resource.as_str()),
                _ => None,
            })
            .collect::<HashSet<_>>();
        let resources = self.list_resources(query.subject).await?;
        let resolvers = self
            .resource_resolvers
            .read()
            .map_err(|_| Error::Infrastructure("resource resolver lock poisoned".into()))?
            .clone();
        let mut actions = Vec::new();
        let mut degradation = Vec::new();
        for resource in resources {
            let descriptor = resource.descriptor;
            if descriptor.readiness != "ready"
                || (!requested.is_empty() && !requested.contains(descriptor.resource_ref.as_str()))
            {
                continue;
            }
            let Some(resolver) = resolvers.get(&descriptor.resolver_key).cloned() else {
                if query.resources.current_authority == CurrentAuthorityNeed::Required {
                    return Err(Error::Unavailable(format!(
                        "resource resolver '{}' is unavailable",
                        descriptor.resolver_key
                    )));
                }
                degradation.push(Degradation {
                    code: "resource_unavailable".into(),
                    detail: Some(format!(
                        "resolver '{}' is not registered",
                        descriptor.resolver_key
                    )),
                });
                continue;
            };
            let result = resolver
                .query(
                    &descriptor.resource_ref,
                    ResourceQuery {
                        dimensions: descriptor.query_dimensions.clone(),
                        synopsis_only: false,
                        limit: query.result_need.limit,
                    },
                )
                .await;
            match result {
                Ok(result) => {
                    if !result.current_authority {
                        if query.resources.current_authority == CurrentAuthorityNeed::Required {
                            return Err(Error::Unavailable(
                                "resource resolver did not return current authority".into(),
                            ));
                        }
                        degradation.push(Degradation {
                            code: "resource_unavailable".into(),
                            detail: Some(
                                "resolver returned a historical or non-authoritative result".into(),
                            ),
                        });
                    }
                    if let Some(detail) = result.degraded {
                        degradation.push(Degradation {
                            code: "resource_unavailable".into(),
                            detail: Some(detail),
                        });
                    }
                    actions.push(ResourceActionSuggestion {
                        resource: descriptor.resource_ref,
                        action: "query_current_authority".into(),
                        reason: format!(
                            "resolver returned {} current records",
                            result.records.len()
                        ),
                        current_authority: result.current_authority,
                        records: result.records,
                        evidence: if query.result_need.need_evidence {
                            result.evidence
                        } else {
                            Vec::new()
                        },
                    });
                }
                Err(error)
                    if query.resources.current_authority == CurrentAuthorityNeed::Required =>
                {
                    return Err(Error::Unavailable(format!(
                        "current resource authority query failed: {error}"
                    )));
                }
                Err(error) => degradation.push(Degradation {
                    code: "resource_unavailable".into(),
                    detail: Some(error.to_string()),
                }),
            }
        }
        if query.resources.current_authority == CurrentAuthorityNeed::Required && actions.is_empty()
        {
            return Err(Error::Unavailable(
                "required current resource authority is unavailable".into(),
            ));
        }
        if query.resources.current_authority == CurrentAuthorityNeed::Prefer && actions.is_empty() {
            degradation.push(Degradation {
                code: "resource_unavailable".into(),
                detail: Some("no ready matching current resource authority".into()),
            });
        }
        Ok((actions, degradation))
    }
}
