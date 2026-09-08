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
        if input.evidence.iter().any(|evidence| {
            evidence
                .weight
                .is_some_and(|weight| !weight.is_finite() || weight < 0.0)
        }) {
            return Err(Error::Invalid(
                "revision evidence weight must be finite and non-negative".into(),
            ));
        }
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
        sqlx::query("INSERT INTO memory_revisions(memory_revision_id,memory_id,subject_id,revision_no,parent_revision_id,semantic_role,title,representation_text,attributes,epistemic_class,confidence,occurred_at,observed_at,valid_from,valid_to,created_at,supersession_state) VALUES($1,$2,$3,1,NULL,$4,$5,$6,'{}',$7,$8,$9,$10,$11,$12,$13,'current')")
            .bind(revision_id.0).bind(memory_id.0).bind(input.subject.0).bind(&input.semantic_role).bind(&input.title).bind(&input.representation_text).bind(format!("{:?}",input.epistemic_class).to_lowercase()).bind(input.confidence).bind(input.occurred_at).bind(input.observed_at).bind(input.valid_from).bind(input.valid_to).bind(now).execute(&mut **tx).await.map_err(db)?;
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
            sqlx::query("INSERT INTO memory_revision_tags(memory_revision_id,tag_id,role,ordinal,provenance) VALUES($1,$2,'explicit',$3,'{}')")
                .bind(revision_id.0).bind(tag.0).bind(ordinal as i32).execute(&mut **tx).await.map_err(db)?;
        }
        let entity_attachment = input.evidence.first().map(|item| item.evidence.clone());
        for entity in &input.entity_refs {
            let mention_id = Uuid::now_v7();
            // Entity identity comes from the Host. Keep the explicit binding
            // attached to the same evidence root instead of inventing a
            // mention without a source coordinate (the Authority schema
            // intentionally forbids source-less mentions).
            let (occurrence_id, source_region_id, derived_region_id) = match entity_attachment
                .as_ref()
            {
                Some(EvidenceRef::Occurrence { occurrence_id }) => {
                    (Some(occurrence_id.0), None, None)
                }
                Some(EvidenceRef::SourceRegion { source_region_id }) => {
                    (None, Some(source_region_id.0), None)
                }
                Some(EvidenceRef::DerivedRegion { derived_region_id }) => {
                    (None, None, Some(derived_region_id.0))
                }
                Some(EvidenceRef::DerivedRepresentation {
                    derived_representation_id,
                }) => {
                    let source_region: Option<Uuid> = sqlx::query_scalar(
                        "SELECT source_region_id FROM derived_representations WHERE derived_representation_id=$1 AND subject_id=$2",
                    )
                    .bind(derived_representation_id.0)
                    .bind(input.subject.0)
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(db)?;
                    (None, source_region, None)
                }
                None => (None, None, None),
            };
            sqlx::query("INSERT INTO entity_mentions(mention_id,subject_id,occurrence_id,source_region_id,derived_region_id,surface,semantic_role,created_at) VALUES($1,$2,$3,$4,$5,$6,'explicit',$7)")
                .bind(mention_id)
                .bind(input.subject.0)
                .bind(occurrence_id)
                .bind(source_region_id)
                .bind(derived_region_id)
                .bind(entity.as_str())
                .bind(now)
                .execute(&mut **tx)
                .await
                .map_err(db)?;
            sqlx::query("INSERT INTO entity_binding_revisions(binding_revision_id,mention_id,revision_no,entity_ref,binding_state,created_at) VALUES($1,$2,1,$3,'bound',$4)")
                .bind(Uuid::now_v7()).bind(mention_id).bind(entity.as_str()).bind(now).execute(&mut **tx).await.map_err(db)?;
        }
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
        if proposal.evidence.iter().any(|evidence| {
            evidence
                .weight
                .is_some_and(|weight| !weight.is_finite() || weight < 0.0)
        }) {
            return Err(Error::Invalid(
                "revision evidence weight must be finite and non-negative".into(),
            ));
        }
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
                    occurred_at: proposal.occurred_at,
                    observed_at: Utc::now(),
                    valid_from: proposal.valid_from,
                    valid_to: proposal.valid_to,
                    epistemic_class: proposal.epistemic_class,
                    confidence: proposal.confidence,
                },
            )
            .await?;
        for tag in tag_proposals {
            let existing = sqlx::query_scalar::<_, Uuid>(
                "SELECT t.tag_id FROM tags t JOIN tag_revisions tr ON tr.tag_revision_id=t.current_revision_id WHERE t.subject_id=$1 AND t.status='active' AND lower(trim(tr.label))=lower(trim($2)) LIMIT 1",
            )
            .bind(subject.0)
            .bind(&tag.label)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db)?;
            let tag_id = if let Some(tag_id) = existing {
                TagId(tag_id)
            } else {
                let tag_id = TagId::new();
                let tag_revision_id = Uuid::now_v7();
                let now = Utc::now();
                sqlx::query("INSERT INTO tags(tag_id,subject_id,current_revision_id,created_at,status) VALUES($1,$2,$3,$4,'active')")
                    .bind(tag_id.0).bind(subject.0).bind(tag_revision_id).bind(now).execute(&mut *tx).await.map_err(db)?;
                sqlx::query("INSERT INTO tag_revisions(tag_revision_id,tag_id,revision_no,label,description,kind_hint,origin,created_at) VALUES($1,$2,1,$3,$4,$5,'model_proposed',$6)")
                    .bind(tag_revision_id).bind(tag_id.0).bind(&tag.label).bind(&tag.description).bind(&tag.kind_hint).bind(now).execute(&mut *tx).await.map_err(db)?;
                tag_id
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
        let row = sqlx::query("SELECT o.memory_id,o.subject_id,o.memory_class,o.current_revision_id,o.created_at,o.status,r.memory_revision_id,r.revision_no,r.parent_revision_id,r.semantic_role,r.title,r.representation_text,r.attributes,r.epistemic_class,r.confidence,r.occurred_at,r.observed_at,r.valid_from,r.valid_to,r.created_at AS revision_created_at,r.supersession_state FROM memory_objects o JOIN memory_revisions r ON r.memory_revision_id=COALESCE($3,o.current_revision_id) WHERE o.subject_id=$1 AND o.memory_id=$2 AND r.memory_id=o.memory_id")
            .bind(subject.0).bind(memory_id.0).bind(revision.map(|id|id.0)).fetch_optional(self.store.pool()).await.map_err(db)?.ok_or_else(||Error::NotFound("memory not found".into()))?;
        let revision_id = MemoryRevisionId(row.try_get("memory_revision_id").map_err(db)?);
        let evidence_rows = sqlx::query("SELECT evidence_no,occurrence_id,source_region_id,derived_representation_id,derived_region_id,support_role,weight FROM memory_revision_evidence WHERE memory_revision_id=$1 ORDER BY evidence_no")
            .bind(revision_id.0).fetch_all(self.store.pool()).await.map_err(db)?;
        let evidence = evidence_rows
            .into_iter()
            .map(decode_evidence)
            .collect::<Result<Vec<_>>>()?;
        let tags = sqlx::query_scalar::<_,Uuid>("SELECT tag_id FROM memory_revision_tags WHERE memory_revision_id=$1 ORDER BY ordinal NULLS LAST,tag_id")
            .bind(revision_id.0).fetch_all(self.store.pool()).await.map_err(db)?.into_iter().map(TagId).collect();
        let entities = sqlx::query_scalar::<_,String>("SELECT DISTINCT b.entity_ref FROM memory_revision_evidence e JOIN entity_mentions m ON (m.occurrence_id=e.occurrence_id OR m.source_region_id=e.source_region_id OR m.derived_region_id=e.derived_region_id) JOIN entity_binding_revisions b ON b.mention_id=m.mention_id WHERE e.memory_revision_id=$1 AND b.revision_no=(SELECT max(b2.revision_no) FROM entity_binding_revisions b2 WHERE b2.mention_id=b.mention_id) AND b.binding_state='bound' AND b.entity_ref IS NOT NULL")
            .bind(revision_id.0).fetch_all(self.store.pool()).await.map_err(db)?.into_iter().filter_map(|value|EntityRef::new(value).ok()).collect();
        Ok(MemoryView {
            object: MemoryObject {
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
                confidence: row.try_get("confidence").map_err(db)?,
                occurred_at: row.try_get("occurred_at").map_err(db)?,
                observed_at: row.try_get("observed_at").map_err(db)?,
                valid_from: row.try_get("valid_from").map_err(db)?,
                valid_to: row.try_get("valid_to").map_err(db)?,
                created_at: row.try_get("revision_created_at").map_err(db)?,
                supersession_state: parse_supersession(
                    &row.try_get::<String, _>("supersession_state").map_err(db)?,
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

    pub async fn revise_memory(&self, input: ReviseMemoryInput) -> Result<MemoryView> {
        if input.representation_text.trim().is_empty()
            || input.representation_text.len() > 1_000_000
        {
            return Err(Error::Invalid("revision representation is required".into()));
        }
        if let Some(role) = &input.semantic_role
            && (role.trim().is_empty() || role.len() > 128)
        {
            return Err(Error::Invalid("revision semantic_role is invalid".into()));
        }
        if let (Some(start), Some(end)) = (input.valid_from, input.valid_to)
            && start > end
        {
            return Err(Error::Invalid("valid_to precedes valid_from".into()));
        }
        if input
            .confidence
            .is_some_and(|confidence| !confidence.is_finite() || !(0.0..=1.0).contains(&confidence))
        {
            return Err(Error::Invalid("confidence must be within [0,1]".into()));
        }
        self.validate_evidence(input.subject, &input.evidence)
            .await?;
        let current = self.memory(input.subject, input.memory_id, None).await?;
        let now = Utc::now();
        let new_id = MemoryRevisionId::new();
        let next_no = current.revision.revision_no + 1;
        let mut tx = self.store.begin().await?;
        sqlx::query(
            "UPDATE memory_revisions SET supersession_state=$2 WHERE memory_revision_id=$1",
        )
        .bind(current.revision.memory_revision_id.0)
        .bind(match input.relation {
            MemoryRelation::Contradicts => "contradicted",
            _ => "superseded",
        })
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        sqlx::query("INSERT INTO memory_revisions(memory_revision_id,memory_id,subject_id,revision_no,parent_revision_id,semantic_role,title,representation_text,attributes,epistemic_class,confidence,occurred_at,observed_at,valid_from,valid_to,created_at,supersession_state) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'{}',$9,$10,$11,$12,$13,$14,$15,'current')")
            .bind(new_id.0).bind(input.memory_id.0).bind(input.subject.0).bind(next_no).bind(current.revision.memory_revision_id.0).bind(input.semantic_role.unwrap_or(current.revision.semantic_role)).bind(input.title.or(current.revision.title)).bind(&input.representation_text).bind(format!("{:?}",input.epistemic_class).to_lowercase()).bind(input.confidence).bind(input.occurred_at).bind(now).bind(input.valid_from).bind(input.valid_to).bind(now).execute(&mut *tx).await.map_err(db)?;
        sqlx::query(
            "UPDATE memory_objects SET current_revision_id=$2 WHERE subject_id=$1 AND memory_id=$3",
        )
        .bind(input.subject.0)
        .bind(new_id.0)
        .bind(input.memory_id.0)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        sqlx::query("INSERT INTO memory_revision_relations(from_revision_id,to_revision_id,relation,created_at) VALUES($1,$2,$3,$4)").bind(new_id.0).bind(current.revision.memory_revision_id.0).bind(input.relation.as_str()).bind(now).execute(&mut *tx).await.map_err(db)?;
        for evidence in &input.evidence {
            insert_evidence(&mut tx, new_id, evidence).await?;
        }
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
        let view = self.memory(input.subject, input.memory_id, None).await?;
        Ok(view)
    }

    pub async fn suppress(&self, subject: SubjectId, memory: MemoryId) -> Result<MemoryView> {
        let mut tx = self.store.begin().await?;
        let changed = sqlx::query(
            "UPDATE memory_objects SET status='suppressed' WHERE subject_id=$1 AND memory_id=$2",
        )
        .bind(subject.0)
        .bind(memory.0)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        if changed.rows_affected() == 0 {
            return Err(Error::NotFound("memory not found".into()));
        }
        sqlx::query(
            "DELETE FROM resident_refs r USING cognitive_sessions s WHERE r.session_id=s.session_id AND s.subject_id=$1 AND ((r.ref_kind='memory' AND r.ref_value=$2) OR (r.ref_kind='memory_revision' AND r.ref_value IN (SELECT memory_revision_id::text FROM memory_revisions WHERE memory_id=$3)))",
        )
        .bind(subject.0)
        .bind(memory.0.to_string())
        .bind(memory.0)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
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
        let view = self.memory(subject, memory, None).await?;
        Ok(view)
    }

    pub async fn restore(&self, subject: SubjectId, memory: MemoryId) -> Result<MemoryView> {
        let mut tx = self.store.begin().await?;
        let changed = sqlx::query(
            "UPDATE memory_objects SET status='active' WHERE subject_id=$1 AND memory_id=$2",
        )
        .bind(subject.0)
        .bind(memory.0)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        if changed.rows_affected() == 0 {
            return Err(Error::NotFound("memory not found".into()));
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
        let view = self.memory(subject, memory, None).await?;
        Ok(view)
    }

    // Purge coordinates provenance reachability, Authority deletion and CAS cleanup.
    #[allow(clippy::too_many_lines)]
    pub async fn purge_memory(&self, subject: SubjectId, memory: MemoryId) -> Result<()> {
        let guard = self.objects.reference_guard(true).await?;
        let mut tx = self.store.begin().await?;
        let revision_ids = sqlx::query_scalar::<_, Uuid>(
            "SELECT memory_revision_id FROM memory_revisions WHERE subject_id=$1 AND memory_id=$2",
        )
        .bind(subject.0)
        .bind(memory.0)
        .fetch_all(&mut *tx)
        .await
        .map_err(db)?;
        if revision_ids.is_empty() {
            drop(tx);
            drop(guard);
            return Err(Error::NotFound("memory not found".into()));
        }
        let revision_values = revision_ids.clone();
        let derived_ids = sqlx::query_scalar::<_, Uuid>("SELECT DISTINCT derived_representation_id FROM memory_revision_evidence WHERE memory_revision_id=ANY($1) AND derived_representation_id IS NOT NULL UNION SELECT DISTINCT dr.derived_representation_id FROM memory_revision_evidence e JOIN derived_regions dr ON dr.derived_region_id=e.derived_region_id WHERE e.memory_revision_id=ANY($1) AND e.derived_region_id IS NOT NULL")
            .bind(&revision_values)
            .fetch_all(&mut *tx)
            .await
            .map_err(db)?;
        let removable_derived_ids = if derived_ids.is_empty() {
            Vec::new()
        } else {
            sqlx::query_scalar::<_, Uuid>("SELECT d.derived_representation_id FROM derived_representations d WHERE d.derived_representation_id=ANY($1) AND NOT EXISTS (SELECT 1 FROM memory_revision_evidence e WHERE e.memory_revision_id <> ALL($2) AND (e.derived_representation_id=d.derived_representation_id OR e.derived_region_id IN (SELECT derived_region_id FROM derived_regions WHERE derived_representation_id=d.derived_representation_id)))")
                .bind(&derived_ids)
                .bind(&revision_values)
                .fetch_all(&mut *tx)
                .await
                .map_err(db)?
        };
        let mut artifact_ids = sqlx::query_scalar::<_, Uuid>("SELECT DISTINCT sr.artifact_id FROM memory_revision_evidence e JOIN source_regions sr ON sr.source_region_id=e.source_region_id WHERE e.memory_revision_id=ANY($1) AND e.source_region_id IS NOT NULL UNION SELECT DISTINCT o.artifact_id FROM memory_revision_evidence e JOIN observation_occurrences o ON o.occurrence_id=e.occurrence_id WHERE e.memory_revision_id=ANY($1) AND o.artifact_id IS NOT NULL")
            .bind(&revision_values)
            .fetch_all(&mut *tx)
            .await
            .map_err(db)?;
        if !removable_derived_ids.is_empty() {
            let derived_artifacts = sqlx::query_scalar::<_, Uuid>(
                "SELECT payload_artifact_id FROM derived_representations WHERE derived_representation_id=ANY($1) AND payload_artifact_id IS NOT NULL",
            )
            .bind(&removable_derived_ids)
            .fetch_all(&mut *tx)
            .await
            .map_err(db)?;
            artifact_ids.extend(derived_artifacts);
            sqlx::query("UPDATE coverage_needs SET current_representation_id=NULL,state='missing',updated_at=$2 WHERE current_representation_id=ANY($1)")
                .bind(&removable_derived_ids)
                .bind(Utc::now())
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            sqlx::query("UPDATE derivations SET successful_representation_id=NULL,state='pending',updated_at=$2 WHERE successful_representation_id=ANY($1)")
                .bind(&removable_derived_ids)
                .bind(Utc::now())
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            sqlx::query("DELETE FROM derived_representations d WHERE d.derived_representation_id=ANY($1) AND NOT EXISTS (SELECT 1 FROM memory_revision_evidence e WHERE e.derived_representation_id=d.derived_representation_id)")
                .bind(&removable_derived_ids)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        let reference_values = revision_ids
            .iter()
            .map(ToString::to_string)
            .chain(std::iter::once(memory.0.to_string()))
            .collect::<Vec<_>>();
        sqlx::query("DELETE FROM association_evidence WHERE subject_id=$1 AND ((from_ref_kind='memory' AND from_ref=ANY($2)) OR (from_ref_kind='memory_revision' AND from_ref=ANY($2)) OR (to_ref_kind='memory' AND to_ref=ANY($2)) OR (to_ref_kind='memory_revision' AND to_ref=ANY($2)) OR memory_revision_id=ANY($3))")
            .bind(subject.0)
            .bind(&reference_values)
            .bind(&revision_values)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("DELETE FROM anchor_support WHERE support_ref_kind IN ('memory','memory_revision') AND support_ref=ANY($1)")
            .bind(&reference_values)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("UPDATE anchors a SET status='revoked' WHERE a.subject_id=$1 AND a.status='active' AND NOT EXISTS (SELECT 1 FROM anchor_revisions ar JOIN anchor_support s ON s.anchor_revision_id=ar.anchor_revision_id WHERE ar.anchor_revision_id=a.current_revision_id)")
            .bind(subject.0)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("DELETE FROM memory_objects WHERE subject_id=$1 AND memory_id=$2")
            .bind(subject.0)
            .bind(memory.0)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        let mut ref_values = vec![memory.0.to_string()];
        ref_values.extend(
            revision_ids
                .into_iter()
                .map(|revision| revision.to_string()),
        );
        sqlx::query("DELETE FROM resident_refs r USING cognitive_sessions s WHERE r.session_id=s.session_id AND s.subject_id=$1 AND r.ref_kind IN ('memory','memory_revision') AND r.ref_value=ANY($2)")
            .bind(subject.0)
            .bind(&ref_values)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
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
        let mut removable_hashes = Vec::new();
        artifact_ids.sort_unstable();
        artifact_ids.dedup();
        for artifact_id in artifact_ids {
            let Some(hash) = sqlx::query_scalar::<_, String>(
                "SELECT content_hash FROM artifacts WHERE subject_id=$1 AND artifact_id=$2 AND NOT EXISTS (SELECT 1 FROM observation_occurrences WHERE artifact_id=$2) AND NOT EXISTS (SELECT 1 FROM source_regions WHERE artifact_id=$2) AND NOT EXISTS (SELECT 1 FROM derived_representations WHERE payload_artifact_id=$2)",
            )
            .bind(subject.0)
            .bind(artifact_id)
            .fetch_optional(self.store.pool())
            .await
            .map_err(db)? else {
                continue;
            };
            let deleted =
                sqlx::query("DELETE FROM artifacts WHERE subject_id=$1 AND artifact_id=$2")
                    .bind(subject.0)
                    .bind(artifact_id)
                    .execute(self.store.pool())
                    .await
                    .map_err(db)?;
            if deleted.rows_affected() == 1 {
                removable_hashes.push(hash);
            }
        }
        for hash in removable_hashes {
            let retained: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM artifacts WHERE content_hash=$1)")
                    .bind(&hash)
                    .fetch_one(self.store.pool())
                    .await
                    .map_err(db)?;
            if !retained {
                self.objects.delete_unreferenced(&hash).await?;
            }
        }
        drop(guard);
        Ok(())
    }

    pub async fn consolidate(
        &self,
        subject: SubjectId,
        request: ConsolidationRequest,
    ) -> Result<MemoryView> {
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
        let mut evidence = Vec::new();
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
            for item in source.evidence {
                let mut item = item;
                item.evidence_no = evidence.len() as i32;
                item.support_role = SupportRole::Contextual;
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
                return Err(Error::Invalid(
                    "TopologyOnly consolidation has no Memory representation".into(),
                ));
            }
        };
        let memory = self
            .form_memory(ExplicitMemoryInput {
                subject,
                memory_class,
                semantic_role: request.semantic_role.unwrap_or_else(|| role.into()),
                representation_text: representation,
                title: None,
                evidence,
                entity_refs: Vec::new(),
                tags: Vec::new(),
                occurred_at: None,
                observed_at: Utc::now(),
                valid_from: None,
                valid_to: None,
                epistemic_class: EpistemicClass::Derived,
                confidence: None,
            })
            .await?;
        for source_revision in &request.source_memories {
            sqlx::query("INSERT INTO memory_revision_relations(from_revision_id,to_revision_id,relation,created_at) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING")
                .bind(memory.revision.memory_revision_id.0)
                .bind(source_revision.0)
                .bind(match memory_class {
                    MemoryClass::Integrative => "integrates",
                    MemoryClass::Procedural => "proceduralizes",
                    MemoryClass::Specific => "derived_from",
                })
                .bind(Utc::now())
                .execute(self.store.pool())
                .await
                .map_err(db)?;
        }
        Ok(memory)
    }
}
