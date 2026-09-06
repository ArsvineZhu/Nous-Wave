use crate::{
    LocalRuntime,
    material::{MaterialContent, PreparedArtifact, decode_enum, write_artifact},
    memory::{MemoryDraft, form_memory, lock_subject},
    models::ExtractionMaterial,
    subjects::db,
};
use chrono::{DateTime, Utc};
use nous_core::{Error, Result, SubjectId};
use nous_material::{Classification, EpistemicClass};
use nous_memory_domain::MemoryKind;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub succeeded: usize,
    pub failed: usize,
    pub pending: i64,
}

struct Claim {
    id: Uuid,
    subject: SubjectId,
    attempt: Uuid,
    kind: String,
    source: Option<Uuid>,
    payload: serde_json::Value,
}

impl LocalRuntime {
    pub async fn process_pending(&self, limit: usize) -> Result<ProcessingResult> {
        if limit == 0 || limit > 1000 {
            return Err(Error::Invalid("processing limit must be 1..1000".into()));
        }
        let mut result = ProcessingResult {
            succeeded: 0,
            failed: 0,
            pending: 0,
        };
        self.resume_purges(1).await?;
        let mut kinds = vec!["ingest", "regenerate"];
        if self.models.embedding.is_some() {
            kinds.push("embedding");
        }
        if self.projection.is_some() {
            kinds.push("projection");
        }
        for _ in 0..limit {
            let attempt = Uuid::now_v7();
            let row = sqlx::query("WITH candidate AS (SELECT obligation_id FROM processing_obligations WHERE kind=ANY($1) AND (state='pending' OR (state='running' AND lease_until<clock_timestamp())) ORDER BY created_at,obligation_id FOR UPDATE SKIP LOCKED LIMIT 1) UPDATE processing_obligations o SET state='running',attempt_id=$2,lease_until=clock_timestamp()+interval '5 minutes',attempts=attempts+1,updated_at=clock_timestamp() FROM candidate c WHERE o.obligation_id=c.obligation_id RETURNING o.*")
                .bind(&kinds).bind(attempt).fetch_optional(self.store.pool()).await.map_err(db)?;
            let Some(row) = row else { break };
            let claim = Claim {
                id: row.try_get("obligation_id").map_err(db)?,
                subject: SubjectId(row.try_get("subject_id").map_err(db)?),
                attempt,
                kind: row.try_get("kind").map_err(db)?,
                source: row.try_get("source_id").map_err(db)?,
                payload: row.try_get("payload").map_err(db)?,
            };
            let work = match claim.kind.as_str() {
                "ingest" => self.process_ingest(&claim).await,
                "embedding" => self.process_embedding(&claim).await,
                "projection" => self.process_projection(&claim).await,
                "regenerate" => self.process_regeneration(&claim).await,
                _ => Err(Error::Invalid("unsupported memory obligation".into())),
            };
            match work {
                Ok(()) => result.succeeded += 1,
                Err(error) => {
                    let (state, code) = match error {
                        Error::Unavailable(_) | Error::Infrastructure(_) => {
                            ("pending", "dependency_unavailable")
                        }
                        Error::Conflict(_) => ("pending", "revision_changed"),
                        _ => ("failed", "invalid_material"),
                    };
                    sqlx::query("UPDATE processing_obligations SET state=$3,attempt_id=NULL,lease_until=NULL,problem_code=$4,updated_at=clock_timestamp() WHERE obligation_id=$1 AND attempt_id=$2")
                        .bind(claim.id).bind(claim.attempt).bind(state).bind(code).execute(self.store.pool()).await.map_err(db)?;
                    result.failed += 1;
                    // A dependency outage must not spin on the same accepted obligation.
                    if state == "pending" {
                        break;
                    }
                }
            }
        }
        result.pending = sqlx::query_scalar(
            "SELECT count(*) FROM processing_obligations WHERE state IN ('pending','running')",
        )
        .fetch_one(self.store.pool())
        .await
        .map_err(db)?;
        Ok(result)
    }

    async fn process_ingest(&self, claim: &Claim) -> Result<()> {
        let _objects = self.objects.reference_guard(false).await?;
        self.require_memory(claim.subject).await?;
        let source = claim
            .source
            .ok_or_else(|| Error::Invalid("ingest obligation has no source".into()))?;
        let row = sqlx::query("SELECT *,lower(occurred) AS occurred_start,upper(occurred) AS occurred_end,occurred IS NOT NULL AS has_occurred FROM source_records WHERE subject_id=$1 AND source_id=$2")
            .bind(claim.subject.0).bind(source).fetch_one(self.store.pool()).await.map_err(db)?;
        let class = Classification {
            origin: decode_enum(row.try_get("origin_class").map_err(db)?)?,
            semantic: row.try_get("semantic_class").map_err(db)?,
            epistemic: decode_enum(row.try_get("epistemic_class").map_err(db)?)?,
        };
        let scope: String = row.try_get("scope").map_err(db)?;
        let metadata: serde_json::Value = row.try_get("metadata").map_err(db)?;
        let roots:Vec<Uuid> = sqlx::query_scalar("SELECT a.artifact_id FROM artifact_sources a WHERE a.subject_id=$1 AND a.source_id=$2 AND NOT EXISTS(SELECT 1 FROM derivation_outputs d WHERE d.artifact_id=a.artifact_id) ORDER BY a.artifact_id")
            .bind(claim.subject.0).bind(source).fetch_all(self.store.pool()).await.map_err(db)?;
        let mut sections: Vec<(Uuid, usize, usize, String, PreparedArtifact)> = vec![];
        let mut extraction = vec![];
        let mut preview = String::new();
        for root in &roots {
            let artifact = self.artifact(claim.subject, *root).await?;
            if artifact.media_type.starts_with("text/") || artifact.media_type == "application/json"
            {
                let text = String::from_utf8(self.artifact_bytes(claim.subject, *root).await?)
                    .map_err(|_| Error::Invalid("text artifact is not valid UTF-8".into()))?;
                if preview.is_empty() {
                    preview = text.chars().take(4096).collect();
                }
                for (start, end) in segment_ranges(&text, 4096) {
                    let section = text[start..end].to_string();
                    let prepared = self
                        .prepare_artifact(
                            claim.subject,
                            "text/plain",
                            &MaterialContent::Text {
                                text: section.clone(),
                            },
                        )
                        .await?;
                    if self.models.generation.is_some() {
                        extraction.push(ExtractionMaterial {
                            source_id: source,
                            artifact_id: *root,
                            text: section.clone(),
                            epistemic_class: class.epistemic,
                        });
                    }
                    sections.push((*root, start, end, section, prepared));
                }
            }
        }
        let mut proposals = vec![];
        for material in &extraction {
            let (result, provenance) = self.models.extract(std::slice::from_ref(material)).await?;
            proposals.push((result, provenance));
        }
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        lock_subject(&mut tx, claim.subject).await?;
        finish_claim(&mut tx, claim).await?;
        let mut ordinals = std::collections::HashMap::<Uuid, i32>::new();
        for (root, start, end, text, prepared) in sections {
            let output = write_artifact(&mut tx, claim.subject, prepared, &class).await?;
            let derivation = Uuid::now_v7();
            sqlx::query("INSERT INTO derivations(derivation_id,subject_id,processor_identity,processor_revision,preprocessing_identity,config_digest) VALUES($1,$2,'nous:unicode_sections','1','utf8-preserving','chars=4096')")
                .bind(derivation).bind(claim.subject.0).execute(&mut *tx).await.map_err(db)?;
            sqlx::query("INSERT INTO derivation_inputs(subject_id,derivation_id,artifact_id) VALUES($1,$2,$3)").bind(claim.subject.0).bind(derivation).bind(root).execute(&mut *tx).await.map_err(db)?;
            sqlx::query("INSERT INTO derivation_outputs(subject_id,derivation_id,artifact_id) VALUES($1,$2,$3)").bind(claim.subject.0).bind(derivation).bind(output).execute(&mut *tx).await.map_err(db)?;
            sqlx::query(
                "INSERT INTO artifact_sources(subject_id,artifact_id,source_id) VALUES($1,$2,$3)",
            )
            .bind(claim.subject.0)
            .bind(output)
            .bind(source)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
            let ordinal = ordinals.entry(root).or_default();
            sqlx::query("INSERT INTO document_sections(artifact_id,subject_id,source_artifact,ordinal,byte_start,byte_end,section_text) VALUES($1,$2,$3,$4,$5,$6,$7)")
                .bind(output).bind(claim.subject.0).bind(root).bind(*ordinal).bind(start as i64).bind(end as i64).bind(text).execute(&mut *tx).await.map_err(db)?;
            *ordinal += 1;
        }
        let occurred = if row.try_get::<bool, _>("has_occurred").map_err(db)? {
            Some(nous_material::TemporalRange {
                start: row.try_get("occurred_start").map_err(db)?,
                end: row.try_get("occurred_end").map_err(db)?,
                approximate: row.try_get("approximate_time").map_err(db)?,
            })
        } else {
            None
        };
        let observed: Option<DateTime<Utc>> = row.try_get("observed_at").map_err(db)?;
        let source_kind: String = row.try_get("source_kind").map_err(db)?;
        let draft = MemoryDraft {
            kind: MemoryKind::Reference,
            scope: scope.clone(),
            title: format!("{}: {}", class.semantic, source_kind),
            text: preview,
            classification: class.clone(),
            occurred: occurred.clone(),
            observed_at: observed,
            source_refs: vec![source],
            artifact_refs: roots.clone(),
            derivation_refs: vec![],
            entities: vec![],
        };
        let (reference, _) =
            form_memory(&mut tx, claim.subject, &draft, &format!("source:{source}")).await?;
        if let Some(group) = metadata
            .get("episode_key")
            .and_then(serde_json::Value::as_str)
        {
            let source_refs:Vec<Uuid>=sqlx::query_scalar("SELECT source_id FROM source_records WHERE subject_id=$1 AND scope=$2 AND metadata->>'episode_key'=$3 ORDER BY recorded_at,source_id")
                .bind(claim.subject.0).bind(&scope).bind(group).fetch_all(&mut *tx).await.map_err(db)?;
            let artifact_refs:Vec<Uuid>=sqlx::query_scalar("SELECT DISTINCT artifact_id FROM artifact_sources WHERE subject_id=$1 AND source_id=ANY($2) AND NOT EXISTS(SELECT 1 FROM derivation_outputs d WHERE d.artifact_id=artifact_sources.artifact_id)")
                .bind(claim.subject.0).bind(&source_refs).fetch_all(&mut *tx).await.map_err(db)?;
            let episode = MemoryDraft {
                kind: MemoryKind::Episode,
                title: format!("Episode: {group}"),
                text: format!(
                    "Host-grouped experience with {} source events",
                    source_refs.len()
                ),
                classification: Classification {
                    epistemic: EpistemicClass::Derived,
                    ..class.clone()
                },
                source_refs,
                artifact_refs,
                ..draft.clone()
            };
            let key = format!("episode:{scope}:{group}");
            let (episode_id, head) = form_memory(&mut tx, claim.subject, &episode, &key).await?;
            let current_count: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM memory_revision_sources WHERE revision_id=$1",
            )
            .bind(head)
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?;
            if current_count != episode.source_refs.len() as i64 {
                let new_head = crate::memory::write_revision(
                    &mut tx,
                    claim.subject,
                    episode_id,
                    &episode,
                    "additional host-grouped evidence",
                    None,
                )
                .await?;
                sqlx::query("INSERT INTO memory_revision_parents(subject_id,revision_id,parent_revision_id,relation) VALUES($1,$2,$3,'WORLD_EVOLUTION')").bind(claim.subject.0).bind(new_head).bind(head).execute(&mut *tx).await.map_err(db)?;
            }
            sqlx::query("INSERT INTO episode_members(subject_id,episode_id,member_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(claim.subject.0).bind(episode_id).bind(reference).execute(&mut *tx).await.map_err(db)?;
        }
        let mut proposal_index = 0;
        for (result, provenance) in proposals {
            let derivation = Uuid::now_v7();
            sqlx::query("INSERT INTO derivations(derivation_id,subject_id,processor_identity,processor_revision,preprocessing_identity,config_digest) VALUES($1,$2,$3,$4,$5,$6)")
                .bind(derivation).bind(claim.subject.0).bind(provenance.identity).bind(provenance.revision).bind(provenance.preprocessing).bind(provenance.config_digest).execute(&mut *tx).await.map_err(db)?;
            for root in &roots {
                sqlx::query("INSERT INTO derivation_inputs(subject_id,derivation_id,artifact_id) VALUES($1,$2,$3)").bind(claim.subject.0).bind(derivation).bind(root).execute(&mut *tx).await.map_err(db)?;
            }
            for proposal in result.memories {
                let mut entities = vec![];
                for label in proposal.entities {
                    let id:Uuid=sqlx::query_scalar("INSERT INTO memory_entities(entity_id,subject_id,entity_kind,label,identity_key) VALUES($1,$2,'concept',$3,$4) ON CONFLICT(subject_id,identity_key) DO UPDATE SET label=memory_entities.label RETURNING entity_id")
                        .bind(Uuid::now_v7()).bind(claim.subject.0).bind(&label).bind(format!("name:{label}")).fetch_one(&mut *tx).await.map_err(db)?;
                    entities.push(id);
                }
                let inferred = MemoryDraft {
                    kind: proposal.kind,
                    title: proposal.title,
                    text: proposal.text,
                    source_refs: proposal.source_refs,
                    artifact_refs: proposal.artifact_refs,
                    derivation_refs: vec![derivation],
                    entities,
                    classification: Classification {
                        epistemic: EpistemicClass::Inferred,
                        ..class.clone()
                    },
                    ..draft.clone()
                };
                form_memory(
                    &mut tx,
                    claim.subject,
                    &inferred,
                    &format!("source:{source}:proposal:{proposal_index}"),
                )
                .await?;
                proposal_index += 1;
            }
        }
        tx.commit().await.map_err(db)
    }

    async fn process_embedding(&self, claim: &Claim) -> Result<()> {
        let revision = payload_uuid(&claim.payload, "revision_id")?;
        let object = payload_uuid(&claim.payload, "object_id")?;
        let memory = self.memory(claim.subject, object, Some(revision)).await?;
        let (embedding, provenance) = self
            .models
            .embed(&[format!("{}\n{}", memory.content.title, memory.content.text)])
            .await?;
        let vector = &embedding.vectors[0];
        let table = nous_memory_retrieval::dense::model_table(
            &provenance.identity,
            &provenance.revision,
            &provenance.preprocessing,
            vector.len(),
        );
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        lock_subject(&mut tx, claim.subject).await?;
        finish_claim(&mut tx, claim).await?;
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM memory_revisions WHERE subject_id=$1 AND revision_id=$2)",
        )
        .bind(claim.subject.0)
        .bind(revision)
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
        if !exists {
            return Err(Error::Conflict("memory was purged during embedding".into()));
        }
        let embedding_id:Uuid=sqlx::query_scalar("INSERT INTO retained_embeddings(embedding_id,subject_id,revision_id,model_identity,model_revision,preprocessing_identity,config_digest,projection_table,vector) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(revision_id,model_identity,model_revision,preprocessing_identity,config_digest) DO UPDATE SET revision_id=excluded.revision_id RETURNING embedding_id")
            .bind(Uuid::now_v7()).bind(claim.subject.0).bind(revision).bind(provenance.identity).bind(provenance.revision).bind(provenance.preprocessing).bind(provenance.config_digest).bind(table).bind(vector).fetch_one(&mut *tx).await.map_err(db)?;
        sqlx::query("INSERT INTO processing_obligations(obligation_id,subject_id,kind,payload_version,payload) VALUES($1,$2,'projection',1,$3)").bind(Uuid::now_v7()).bind(claim.subject.0).bind(serde_json::json!({"embedding_id":embedding_id})).execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)
    }

    async fn process_regeneration(&self, claim: &Claim) -> Result<()> {
        let _objects = self.objects.reference_guard(false).await?;
        let object = payload_uuid(&claim.payload, "object_id")?;
        let sources: Vec<Uuid> = serde_json::from_value(
            claim
                .payload
                .get("source_refs")
                .cloned()
                .unwrap_or_default(),
        )
        .map_err(|_| Error::Invalid("invalid regeneration sources".into()))?;
        let kind: MemoryKind =
            serde_json::from_value(claim.payload.get("kind").cloned().unwrap_or_default())
                .map_err(|_| Error::Invalid("invalid regeneration kind".into()))?;
        let scope = claim
            .payload
            .get("scope")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| Error::Invalid("missing regeneration scope".into()))?;
        let roots:Vec<Uuid>=sqlx::query_scalar("SELECT DISTINCT a.artifact_id FROM artifact_sources a WHERE a.subject_id=$1 AND a.source_id=ANY($2) AND NOT EXISTS(SELECT 1 FROM derivation_outputs d WHERE d.artifact_id=a.artifact_id)").bind(claim.subject.0).bind(&sources).fetch_all(self.store.pool()).await.map_err(db)?;
        let mut text = String::new();
        let mut material = vec![];
        for artifact in &roots {
            let metadata = self.artifact(claim.subject, *artifact).await?;
            if metadata.media_type.starts_with("text/") || metadata.media_type == "application/json"
            {
                let raw = String::from_utf8(self.artifact_bytes(claim.subject, *artifact).await?)
                    .map_err(|_| Error::Invalid("invalid retained UTF-8 content".into()))?;
                let excerpt: String = raw.chars().take(4096).collect();
                if text.len() + excerpt.len() < 200000 {
                    text.push_str(&excerpt);
                    text.push('\n');
                }
                for source in metadata
                    .source_refs
                    .iter()
                    .filter(|id| sources.contains(id))
                {
                    material.push(ExtractionMaterial {
                        source_id: *source,
                        artifact_id: *artifact,
                        text: excerpt.clone(),
                        epistemic_class: metadata.classification.epistemic,
                    });
                }
            }
        }
        let mut draft = MemoryDraft {
            kind,
            scope: scope.into(),
            title: "Reconstructed from retained evidence".into(),
            text,
            classification: Classification {
                origin: nous_material::OriginClass::Host,
                semantic: "NOTE".into(),
                epistemic: EpistemicClass::Derived,
            },
            occurred: None,
            observed_at: None,
            source_refs: sources,
            artifact_refs: roots,
            derivation_refs: vec![],
            entities: vec![],
        };
        let provenance = if matches!(kind, MemoryKind::Episode | MemoryKind::Reference) {
            None
        } else {
            let (result, provenance) = self.models.extract(&material).await?;
            let proposal = result
                .memories
                .into_iter()
                .find(|proposal| proposal.kind == kind)
                .ok_or_else(|| {
                    Error::Invalid("no supported replacement for purged interpretation".into())
                })?;
            draft.title = proposal.title;
            draft.text = proposal.text;
            draft.source_refs = proposal.source_refs;
            draft.artifact_refs = proposal.artifact_refs;
            draft.classification.epistemic = EpistemicClass::Inferred;
            Some(provenance)
        };
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        lock_subject(&mut tx, claim.subject).await?;
        finish_claim(&mut tx, claim).await?;
        if let Some(provenance) = provenance {
            let derivation = Uuid::now_v7();
            sqlx::query("INSERT INTO derivations(derivation_id,subject_id,processor_identity,processor_revision,preprocessing_identity,config_digest) VALUES($1,$2,$3,$4,$5,$6)").bind(derivation).bind(claim.subject.0).bind(provenance.identity).bind(provenance.revision).bind(provenance.preprocessing).bind(provenance.config_digest).execute(&mut *tx).await.map_err(db)?;
            for artifact in &draft.artifact_refs {
                sqlx::query("INSERT INTO derivation_inputs(subject_id,derivation_id,artifact_id) VALUES($1,$2,$3)").bind(claim.subject.0).bind(derivation).bind(artifact).execute(&mut *tx).await.map_err(db)?;
            }
            draft.derivation_refs.push(derivation);
        }
        crate::memory::write_revision(
            &mut tx,
            claim.subject,
            object,
            &draft,
            "regeneration after explicit purge",
            None,
        )
        .await?;
        sqlx::query(
            "UPDATE memory_objects SET availability='ready' WHERE subject_id=$1 AND object_id=$2",
        )
        .bind(claim.subject.0)
        .bind(object)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        tx.commit().await.map_err(db)
    }

    async fn process_projection(&self, claim: &Claim) -> Result<()> {
        let _objects = self.objects.reference_guard(false).await?;
        let embedding = payload_uuid(&claim.payload, "embedding_id")?;
        let row=sqlx::query("SELECT e.*,r.object_id FROM retained_embeddings e JOIN memory_revisions r USING(revision_id) WHERE e.subject_id=$1 AND e.embedding_id=$2")
            .bind(claim.subject.0).bind(embedding).fetch_one(self.store.pool()).await.map_err(db)?;
        let table: String = row.try_get("projection_table").map_err(db)?;
        let vector = nous_memory_retrieval::dense::VectorRow {
            subject: claim.subject,
            object: row.try_get("object_id").map_err(db)?,
            revision: row.try_get("revision_id").map_err(db)?,
            vector: row.try_get("vector").map_err(db)?,
        };
        self.projection
            .as_ref()
            .ok_or_else(|| Error::Unavailable("LanceDB unavailable".into()))?
            .upsert(&table, &[vector])
            .await?;
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        finish_claim(&mut tx, claim).await?;
        tx.commit().await.map_err(db)
    }
}

async fn finish_claim(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, claim: &Claim) -> Result<()> {
    let result=sqlx::query("UPDATE processing_obligations SET state='succeeded',attempt_id=NULL,lease_until=NULL,problem_code=NULL,updated_at=clock_timestamp() WHERE obligation_id=$1 AND attempt_id=$2 AND state='running' AND lease_until>clock_timestamp()")
        .bind(claim.id).bind(claim.attempt).execute(&mut **tx).await.map_err(db)?;
    if result.rows_affected() != 1 {
        return Err(Error::Conflict(
            "processing lease expired or changed".into(),
        ));
    }
    Ok(())
}
fn payload_uuid(payload: &serde_json::Value, key: &str) -> Result<Uuid> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| Error::Invalid(format!("missing {key}")))
}
pub fn segment_ranges(text: &str, max_chars: usize) -> Vec<(usize, usize)> {
    if max_chars == 0 {
        return vec![];
    }
    let mut ranges = vec![];
    let mut start = 0;
    let mut count = 0;
    for (offset, _) in text.char_indices() {
        if count == max_chars {
            ranges.push((start, offset));
            start = offset;
            count = 0;
        }
        count += 1;
    }
    if start < text.len() {
        ranges.push((start, text.len()));
    }
    ranges
}
