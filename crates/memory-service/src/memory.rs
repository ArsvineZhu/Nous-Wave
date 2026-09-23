use super::memory_support::*;
use super::support::*;
use super::*;
use nous_authority_store::ProjectionInvalidation;

impl MemoryService {
    pub async fn revision(
        &self,
        subject: SubjectId,
        revision: MemoryRevisionId,
    ) -> Result<MemoryView> {
        let memory: Uuid = sqlx::query_scalar(
            "SELECT memory_id FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$2",
        )
        .bind(subject.0)
        .bind(revision.0)
        .fetch_one(self.store.pool())
        .await
        .map_err(db)?;
        self.memory(subject, MemoryId(memory), Some(revision)).await
    }
    pub async fn form_memory(&self, input: ExplicitMemoryInput) -> Result<MemoryView> {
        input.validate()?;
        self.require_subject(input.subject).await?;
        self.validate_evidence(input.subject, &input.evidence)
            .await?;
        for entity in &input.entity_refs {
            EntityRef::new(entity.as_str())?;
        }
        let mut tx = self.store.begin().await?;
        let (memory_id, _revision_id) = self.insert_memory_authority(&mut tx, &input).await?;
        nous_authority_store::AuthorityStore::invalidate_in(
            &mut tx,
            input.subject,
            ProjectionInvalidation {
                topology: true,
                ..ProjectionInvalidation::text()
            },
        )
        .await?;
        tx.commit().await.map_err(db)?;
        self.memory(input.subject, memory_id, None).await
    }

    async fn insert_memory_authority(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        input: &ExplicitMemoryInput,
    ) -> Result<(MemoryId, MemoryRevisionId)> {
        let memory_id = MemoryId::new();
        let revision_id = MemoryRevisionId::new();
        let now = Utc::now();
        sqlx::query("INSERT INTO memory_objects(memory_id,subject_id,memory_class,current_revision_id,created_at,status) VALUES($1,$2,$3,$4,$5,'active')")
            .bind(memory_id.0).bind(input.subject.0).bind(input.memory_class.as_str()).bind(revision_id.0).bind(now).execute(&mut **tx).await.map_err(db)?;
        sqlx::query("INSERT INTO memory_revisions(memory_revision_id,memory_id,subject_id,revision_no,parent_revision_id,semantic_role,title,representation_text,attributes,epistemic_class,valid_from,valid_to,created_at,revision_lifecycle) VALUES($1,$2,$3,1,NULL,$4,$5,$6,'{}',$7,$8,$9,$10,'current')")
            .bind(revision_id.0).bind(memory_id.0).bind(input.subject.0).bind(&input.semantic_role).bind(&input.title).bind(&input.representation_text).bind(format!("{:?}",input.epistemic_class).to_lowercase()).bind(input.valid_from).bind(input.valid_to).bind(now).execute(&mut **tx).await.map_err(db)?;
        sqlx::query("UPDATE memory_objects SET current_revision_id=$2 WHERE memory_id=$1")
            .bind(memory_id.0)
            .bind(revision_id.0)
            .execute(&mut **tx)
            .await
            .map_err(db)?;
        for evidence in &input.evidence {
            insert_evidence(tx, revision_id, evidence).await?;
        }
        for (ordinal, tag) in input.tags.iter().enumerate() {
            let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM tags WHERE subject_id=$1 AND tag_id=$2 AND status='active')").bind(input.subject.0).bind(tag.0).fetch_one(&mut **tx).await.map_err(db)?;
            if !exists {
                return Err(Error::Invalid("tag does not belong to Subject".into()));
            }
            let (ordinal, order_provenance) = input
                .tag_order_provenance
                .as_ref()
                .map(|provenance| (Some(ordinal as i32), Some(provenance.clone())))
                .unwrap_or((None, None));
            sqlx::query("INSERT INTO memory_revision_tags(memory_revision_id,tag_id,role,ordinal,order_provenance,provenance) VALUES($1,$2,'explicit',$3,$4,'{}')")
                .bind(revision_id.0).bind(tag.0).bind(ordinal).bind(order_provenance).execute(&mut **tx).await.map_err(db)?;
        }
        insert_memory_entities(tx,revision_id,&input.entity_refs,"explicit_formation").await?;
        Ok((memory_id, revision_id))
    }

    pub async fn commit_formation_proposal(
        &self,
        subject: SubjectId,
        proposal: MemoryFormationProposal,
        allowed_entity_refs: &[EntityRef],
    ) -> Result<MemoryView> {
        proposal.validate()?;
        let allowed = allowed_entity_refs.iter().collect::<HashSet<_>>();
        if proposal
            .entity_refs
            .iter()
            .any(|entity| !allowed.contains(entity))
        {
            return Err(Error::Invalid(
                "formation proposal contains an EntityRef not supplied by Host context".into(),
            ));
        }
        self.validate_evidence(subject, &proposal.evidence).await?;
        for entity in &proposal.entity_refs {
            EntityRef::new(entity.as_str())?;
        }
        let tag_proposals = proposal.tag_proposals.clone();
        let mut tx = self.store.begin().await?;
        let (memory_id, revision_id) = self
            .insert_memory_authority(
                &mut tx,
                &ExplicitMemoryInput {
                    subject,
                    memory_class: proposal.memory_class,
                    semantic_role: proposal.semantic_role,
                    representation_text: proposal.representation_text,
                    title: proposal.title,
                    evidence: proposal.evidence,
                    entity_refs: proposal.entity_refs,
                    tags: Vec::new(),
                    tag_order_provenance: None,
                    valid_from: proposal.valid_from,
                    valid_to: proposal.valid_to,
                    epistemic_class: proposal.epistemic_class,
                },
            )
            .await?;
        for tag in tag_proposals {
            let tag_id = match tag {
                TagProposal::Existing{tag_id}=>{
                    let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM tags WHERE subject_id=$1 AND tag_id=$2 AND status='active')").bind(subject.0).bind(tag_id.0).fetch_one(&mut *tx).await.map_err(db)?;
                    if !valid{return Err(Error::Invalid("Tag proposal references another Subject or inactive Tag".into()));}
                    tag_id
                }
                TagProposal::New{label,description,kind_hint}=>insert_tag_in_tx(&mut tx,subject,&label,description.as_deref(),kind_hint.as_deref(),"model_proposed").await?,
            };
            sqlx::query("INSERT INTO memory_revision_tags(memory_revision_id,tag_id,role,ordinal,provenance) VALUES($1,$2,'inferred',NULL,$3) ON CONFLICT DO NOTHING")
                .bind(revision_id.0)
                .bind(tag_id.0)
                .bind(serde_json::json!({"origin":"formation_proposal"}))
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        nous_authority_store::AuthorityStore::invalidate_in(
            &mut tx,
            subject,
            ProjectionInvalidation {
                topology: true,
                ..ProjectionInvalidation::text()
            },
        )
        .await?;
        tx.commit().await.map_err(db)?;
        self.memory(subject, memory_id, None).await
    }

    pub async fn memory(
        &self,
        subject: SubjectId,
        memory_id: MemoryId,
        revision: Option<MemoryRevisionId>,
    ) -> Result<MemoryView> {
        let row = sqlx::query("SELECT o.accessibility_mode,o.head_revision,o.memory_id,o.subject_id,o.memory_class,o.current_revision_id,o.created_at,o.status,r.memory_revision_id,r.revision_no,r.parent_revision_id,r.semantic_role,r.title,r.representation_text,r.attributes,r.epistemic_class,r.valid_from,r.valid_to,r.created_at AS revision_created_at,r.revision_lifecycle,r.revision_intent FROM memory_objects o JOIN memory_revisions r ON r.memory_revision_id=COALESCE($3,o.current_revision_id) WHERE o.subject_id=$1 AND o.memory_id=$2 AND r.memory_id=o.memory_id")
            .bind(subject.0).bind(memory_id.0).bind(revision.map(|id|id.0)).fetch_optional(self.store.pool()).await.map_err(db)?.ok_or_else(||Error::NotFound("memory not found".into()))?;
        let revision_id = MemoryRevisionId(row.try_get("memory_revision_id").map_err(db)?);
        let evidence_rows = sqlx::query("SELECT evidence_no,occurrence_id,source_region_id,derived_representation_id,derived_region_id,support_role FROM memory_revision_evidence WHERE memory_revision_id=$1 ORDER BY evidence_no")
            .bind(revision_id.0).fetch_all(self.store.pool()).await.map_err(db)?;
        let evidence = evidence_rows
            .into_iter()
            .map(decode_evidence)
            .collect::<Result<Vec<_>>>()?;
        let tags = sqlx::query_scalar::<_,Uuid>("SELECT tag_id FROM memory_revision_tags WHERE memory_revision_id=$1 ORDER BY ordinal NULLS LAST,tag_id")
            .bind(revision_id.0).fetch_all(self.store.pool()).await.map_err(db)?.into_iter().map(TagId).collect();
        let entities = sqlx::query_scalar::<_,String>("SELECT DISTINCT entity_ref FROM memory_revision_entities WHERE memory_revision_id=$1 ORDER BY entity_ref")
            .bind(revision_id.0).fetch_all(self.store.pool()).await.map_err(db)?.into_iter().filter_map(|value|EntityRef::new(value).ok()).collect();
        let (relations,relations_truncated)=self.revision_relations(subject,revision_id).await?;
        Ok(MemoryView {
            relations,relations_truncated,accessibility_level:self.accessibility_level(subject,memory_id,Utc::now()).await?,
            temporal_evidence:self.temporal_evidence(revision_id).await?,
            object: MemoryObject {
                head_revision: row.try_get("head_revision").map_err(db)?,
                accessibility_mode:parse_accessibility_mode(&row.try_get::<String,_>("accessibility_mode").map_err(db)?)?,
                memory_id: MemoryId(row.try_get("memory_id").map_err(db)?),
                subject_id: SubjectId(row.try_get("subject_id").map_err(db)?),
                memory_class: parse_memory_class(
                    &row.try_get::<String, _>("memory_class").map_err(db)?,
                )?,
                current_revision_id: MemoryRevisionId(
                    row.try_get("current_revision_id").map_err(db)?,
                ),
                created_at: row.try_get("created_at").map_err(db)?,
                status: parse_memory_status(&row.try_get::<String, _>("status").map_err(db)?)?,
            },
            revision: MemoryRevision {
                revision_intent:row.try_get::<Option<String>,_>("revision_intent").map_err(db)?.map(|s|serde_json::from_value(serde_json::Value::String(s)).map_err(|e|Error::Infrastructure(e.to_string()))).transpose()?,
                memory_revision_id: revision_id,
                memory_id,
                subject_id: subject,
                revision_no: row.try_get("revision_no").map_err(db)?,
                parent_revision_id: row
                    .try_get::<Option<Uuid>, _>("parent_revision_id")
                    .map_err(db)?
                    .map(MemoryRevisionId),
                semantic_role: row.try_get("semantic_role").map_err(db)?,
                title: row.try_get("title").map_err(db)?,
                representation_text: row.try_get("representation_text").map_err(db)?,
                attributes: row.try_get("attributes").map_err(db)?,
                epistemic_class: parse_epistemic(
                    &row.try_get::<String, _>("epistemic_class").map_err(db)?,
                )?,
                valid_from: row.try_get("valid_from").map_err(db)?,
                valid_to: row.try_get("valid_to").map_err(db)?,
                created_at: row.try_get("revision_created_at").map_err(db)?,
                revision_lifecycle: parse_revision_lifecycle(
                    &row.try_get::<String, _>("revision_lifecycle").map_err(db)?,
                )?,
            },
            evidence,
            entities,
            tags,
        })
    }

    pub async fn memory_history(
        &self,
        subject: SubjectId,
        memory_id: MemoryId,
    ) -> Result<Vec<MemoryRevision>> {
        self.memory(subject, memory_id, None).await?;
        let ids = sqlx::query_scalar::<_,Uuid>("SELECT memory_revision_id FROM memory_revisions WHERE subject_id=$1 AND memory_id=$2 ORDER BY revision_no")
            .bind(subject.0).bind(memory_id.0).fetch_all(self.store.pool()).await.map_err(db)?;
        let mut revisions = Vec::new();
        for id in ids {
            revisions.push(
                self.memory(subject, memory_id, Some(MemoryRevisionId(id)))
                    .await?
                    .revision,
            );
        }
        Ok(revisions)
    }

    pub async fn revise_memory(&self,input:ReviseMemoryInput)->Result<MemoryView>{
        self.validate_evidence(input.subject,&input.evidence).await?;
        let mut tx=self.store.begin().await?;
        self.fence_memory_head(&mut tx,input.subject,input.memory_id,input.expected_head_revision).await?;
        let current=self.memory(input.subject,input.memory_id,None).await?;
        let proposal=TopologyRevisionProposal{memory_id:input.memory_id,expected_head_revision:input.expected_head_revision,intent:input.intent,entity_refs:input.entity_refs,representation_text:input.representation_text,semantic_role:input.semantic_role,title:input.title,evidence:input.evidence,valid_from:input.valid_from,valid_to:input.valid_to,epistemic_class:input.epistemic_class};
        let revision=insert_revision_in_tx(&mut tx,input.subject,&current,&proposal).await?;
        AuthorityStore::invalidate_in(&mut tx,input.subject,ProjectionInvalidation{topology:true,..ProjectionInvalidation::text()}).await?;
        tx.commit().await.map_err(db)?;
        self.memory(input.subject,input.memory_id,Some(revision)).await
    }

    pub async fn consolidate(
        &self,
        subject: SubjectId,
        request: ConsolidationRequest,
    ) -> Result<ConsolidationResult> {
        if request.subject != subject {
            return Err(Error::Invalid(
                "consolidation subject does not match route subject".into(),
            ));
        }
        if request.source_memories.is_empty() {
            return Err(Error::Invalid(
                "consolidation needs source revisions".into(),
            ));
        }
        for revision_id in &request.source_memories {
            let valid: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$2)",
            )
            .bind(subject.0)
            .bind(revision_id.0)
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
            if !valid {
                return Err(Error::Invalid(
                    "consolidation source revision is outside Subject".into(),
                ));
            }
        }
        if matches!(request.target, ConsolidationTarget::TopologyOnly) {
            let topology = request
                .topology
                .as_ref()
                .ok_or_else(|| Error::Invalid("TopologyOnly needs a topology proposal".into()))?;
            return self.consolidate_topology(subject, topology).await;
        }
        let mut evidence = Vec::new();
        let mut entities=HashSet::new();
        for revision_id in &request.source_memories {
            let row = sqlx::query("SELECT memory_id FROM memory_revisions WHERE subject_id=$1 AND memory_revision_id=$2")
                .bind(subject.0)
                .bind(revision_id.0)
                .fetch_optional(self.store.pool())
                .await
                .map_err(db)?
                .ok_or_else(|| Error::Invalid("consolidation source revision is outside Subject".into()))?;
            let memory_id = MemoryId(row.try_get("memory_id").map_err(db)?);
            let source = self.memory(subject, memory_id, Some(*revision_id)).await?;
            entities.extend(source.entities);
            for item in source.evidence {
                let mut item = item;
                item.evidence_no = evidence.len() as i32;
                evidence.push(item);
            }
        }
        let representation = request.representation_text.ok_or_else(|| {
            Error::Invalid(
                "consolidation requires an explicit representation in the zero-model core".into(),
            )
        })?;
        let (memory_class, role) = match request.target {
            ConsolidationTarget::Integrative => (MemoryClass::Integrative, "schema"),
            ConsolidationTarget::Procedural => (MemoryClass::Procedural, "procedure"),
            ConsolidationTarget::TopologyOnly => {
                unreachable!("TopologyOnly handled above")
            }
        };
        let memory_input = ExplicitMemoryInput {
            subject,
            memory_class,
            semantic_role: request.semantic_role.unwrap_or_else(|| role.into()),
            representation_text: representation,
            title: None,
            evidence,
            entity_refs: entities.into_iter().collect(),
            tags: Vec::new(),
            tag_order_provenance: None,
            valid_from: None,
            valid_to: None,
            epistemic_class: EpistemicClass::Derived,
        };
        let mut tx = self.store.begin().await?;
        let (memory_id, revision_id) = self.insert_memory_authority(&mut tx, &memory_input).await?;
        for source_revision in &request.source_memories {
            sqlx::query("INSERT INTO memory_revision_relations(from_revision_id,to_revision_id,relation,created_at) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING")
                .bind(revision_id.0)
                .bind(source_revision.0)
                .bind(match memory_class {
                    MemoryClass::Integrative => "integrates",
                    MemoryClass::Procedural => "proceduralizes",
                    MemoryClass::Specific => "derived_from",
                })
                .bind(Utc::now())
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        nous_authority_store::AuthorityStore::invalidate_in(
            &mut tx,
            subject,
            ProjectionInvalidation {
                topology: true,
                ..ProjectionInvalidation::text()
            },
        )
        .await?;
        tx.commit().await.map_err(db)?;
        let memory = self.memory(subject, memory_id, None).await?;
        Ok(ConsolidationResult {
            memory: Some(memory),
            topology_changes: 0,
        })
    }

    async fn consolidate_topology(
        &self,
        subject: SubjectId,
        topology: &TopologyConsolidationProposal,
    ) -> Result<ConsolidationResult> {
        self.validate_topology_proposal(subject, topology).await?;
        let mut tx = self.store.begin().await?;
        let mut changes = 0;
        for tag in &topology.tags {
            let tag_id = if let Some(tag_id) = tag.tag_id {
                tag_id
            } else {
                insert_tag_in_tx(
                    &mut tx,
                    subject,
                    &tag.label,
                    tag.description.as_deref(),
                    tag.kind_hint.as_deref(),
                    "consolidation",
                )
                .await?
            };
            for revision in &tag.attach_to {
                sqlx::query("INSERT INTO memory_revision_tags(memory_revision_id,tag_id,role,ordinal,order_provenance,provenance) VALUES($1,$2,'inferred',NULL,NULL,$3) ON CONFLICT DO NOTHING")
                    .bind(revision.0)
                    .bind(tag_id.0)
                    .bind(serde_json::json!({"origin":"consolidation"}))
                    .execute(&mut *tx)
                    .await
                    .map_err(db)?;
                changes += 1;
            }
            changes += 1;
        }
        for anchor in &topology.anchors {
            insert_anchor_in_tx(&mut tx, subject, anchor, "consolidation").await?;
            changes += 1;
        }
        for association in &topology.associations {
            insert_association_in_tx(&mut tx, subject, association).await?;
            changes += 1;
        }
        for revision in &topology.revisions {
            self.fence_memory_head(&mut tx,subject,revision.memory_id,revision.expected_head_revision).await?;
            let current = self.memory(subject, revision.memory_id, None).await?;
            insert_revision_in_tx(&mut tx, subject, &current, revision).await?;
            changes += 1;
        }
        nous_authority_store::AuthorityStore::invalidate_in(
            &mut tx,
            subject,
            ProjectionInvalidation {
                topology: true,
                ..ProjectionInvalidation::text()
            },
        )
        .await?;
        tx.commit().await.map_err(db)?;
        Ok(ConsolidationResult {
            memory: None,
            topology_changes: changes,
        })
    }

    async fn validate_topology_proposal(
        &self,
        subject: SubjectId,
        proposal: &TopologyConsolidationProposal,
    ) -> Result<()> {
        for tag in &proposal.tags {
            if tag.label.trim().is_empty() {
                return Err(Error::Invalid("consolidation Tag label is required".into()));
            }
            if let Some(tag_id) = tag.tag_id
                && !self
                    .reference_in_subject(subject, &CognitiveRef::Tag(tag_id))
                    .await?
            {
                return Err(Error::Invalid(
                    "consolidation Tag is outside Subject".into(),
                ));
            }
            for revision in &tag.attach_to {
                if !self
                    .reference_in_subject(subject, &CognitiveRef::MemoryRevision(*revision))
                    .await?
                {
                    return Err(Error::Invalid(
                        "consolidation Tag attachment is outside Subject".into(),
                    ));
                }
            }
        }
        for anchor in &proposal.anchors {
            if anchor.description.trim().is_empty() {
                return Err(Error::Invalid(
                    "consolidation Anchor description is required".into(),
                ));
            }
            let mut roots = HashSet::new();
            for support in &anchor.supports {
                if !self
                    .reference_in_subject(subject, &support.reference)
                    .await?
                {
                    return Err(Error::Invalid(
                        "consolidation Anchor support is outside Subject".into(),
                    ));
                }
                if let Some(root) = support_root(&support.reference) {
                    roots.insert(root);
                }
            }
            if !anchor.confirmed && roots.len() < 2 {
                return Err(Error::Invalid(
                    "consolidation-derived Anchor needs two independent support roots".into(),
                ));
            }
        }
        for association in &proposal.associations {
            if !association.support_value.is_finite() || !(0.0..=1.0).contains(&association.support_value) {
                return Err(Error::Invalid(
                    "consolidation Association support_value is invalid".into(),
                ));
            }
            if !self
                .reference_in_subject(subject, &association.from)
                .await?
                || !self.reference_in_subject(subject, &association.to).await?
            {
                return Err(Error::Invalid(
                    "consolidation Association endpoint is outside Subject".into(),
                ));
            }
            if let Some(occurrence) = association.occurrence_id {
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
                        "consolidation Association occurrence is outside Subject".into(),
                    ));
                }
            }
            if let Some(revision) = association.memory_revision_id
                && !self
                    .reference_in_subject(subject, &CognitiveRef::MemoryRevision(revision))
                    .await?
            {
                return Err(Error::Invalid(
                    "consolidation Association revision is outside Subject".into(),
                ));
            }
        }
        for revision in &proposal.revisions {
            if revision.representation_text.trim().is_empty()
                || !self
                    .reference_in_subject(subject, &CognitiveRef::Memory(revision.memory_id))
                    .await?
            {
                return Err(Error::Invalid(
                    "consolidation revision target is outside Subject".into(),
                ));
            }
            self.validate_evidence(subject, &revision.evidence).await?;
        }
        Ok(())
    }
}
