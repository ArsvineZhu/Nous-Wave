use crate::{LocalRuntime, material::decode_enum, memory::MemoryView, subjects::db};
use chrono::{DateTime, Utc};
use nous_core::{Error, Result, SeedRevision, Subject, SubjectId};
use nous_material::{
    Artifact, Classification, Derivation, ProcessorProvenance, SourceRecord, TemporalRange,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub schema_version: u32,
    pub subject_id: SubjectId,
    pub exported_at: DateTime<Utc>,
    pub object_hashes: Vec<String>,
    pub memory_revision_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMemory {
    pub object_id: Uuid,
    pub history: Vec<MemoryView>,
    pub formation_class: String,
    pub formation_metadata: serde_json::Value,
    pub accessibility: BundleAccessibility,
    pub suppression: Option<BundleSuppression>,
    pub episode_ids: Vec<Uuid>,
    pub member_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleAccessibility {
    pub meaningful_uses: i64,
    pub last_meaningful_use: Option<DateTime<Utc>>,
    pub retention_hint: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSuppression {
    pub suppressed: bool,
    pub reason: String,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleEntity {
    pub entity_id: Uuid,
    pub entity_kind: String,
    pub label: String,
    pub identity_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleDerivation {
    pub derivation: Derivation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleUseEvent {
    pub event_id: Uuid,
    pub kind: String,
    pub object_refs: Vec<Uuid>,
    pub occurred_at: DateTime<Utc>,
    pub causation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleAssociation {
    pub evidence_id: Uuid,
    pub from_kind: String,
    pub from_id: Uuid,
    pub to_kind: String,
    pub to_id: Uuid,
    pub evidence_class: String,
    pub support: f64,
    pub source_id: Option<Uuid>,
    pub use_event_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectBundle {
    pub manifest: BundleManifest,
    pub subject: Subject,
    pub seeds: Vec<SeedRevision>,
    pub sources: Vec<SourceRecord>,
    pub artifacts: Vec<Artifact>,
    pub memories: Vec<BundleMemory>,
    pub entities: Vec<BundleEntity>,
    pub derivations: Vec<BundleDerivation>,
    pub use_events: Vec<BundleUseEvent>,
    pub associations: Vec<BundleAssociation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRequest {
    pub api_version: u32,
    pub preserve_identity: bool,
    pub rebuild_projection: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub subject_id: SubjectId,
    pub preserved_identity: bool,
    pub imported_sources: usize,
    pub imported_artifacts: usize,
    pub imported_memory_objects: usize,
    pub rebuilt_projection_rows: usize,
}

fn path(root: &Path, name: &str) -> PathBuf {
    root.join(name)
}
async fn write_json<T: Serialize>(root: &Path, name: &str, value: &T) -> Result<()> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|e| Error::Infrastructure(e.to_string()))?;
    let temp = root.join(format!(".{name}.tmp-{}", Uuid::now_v7()));
    tokio::fs::write(&temp, bytes)
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?;
    tokio::fs::rename(temp, path(root, name))
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))
}
async fn read_json<T: for<'a> Deserialize<'a>>(root: &Path, name: &str) -> Result<T> {
    let bytes = tokio::fs::read(path(root, name))
        .await
        .map_err(|e| Error::Invalid(format!("bundle missing {name}: {e}")))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| Error::Invalid(format!("bundle {name} is invalid: {e}")))
}
fn enum_name<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_value(value)
        .map_err(|e| Error::Infrastructure(e.to_string()))?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::Infrastructure("expected enum string".into()))
}

impl LocalRuntime {
    pub async fn export_bundle(
        &self,
        subject: SubjectId,
        root: impl AsRef<Path>,
    ) -> Result<BundleManifest> {
        let _guard = self.objects.reference_guard(false).await?;
        let mut snapshot_guard = self.store.pool().begin().await.map_err(db)?;
        crate::memory::lock_subject(&mut snapshot_guard, subject).await?;
        let root = root.as_ref();
        let incoherent: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM memory_objects WHERE subject_id=$1 AND availability='regenerating') OR EXISTS(SELECT 1 FROM memory_operation_results WHERE subject_id=$1 AND kind='purge' AND state IN ('physical_pending','running'))",
        )
        .bind(subject.0)
        .fetch_one(self.store.pool())
        .await
        .map_err(db)?;
        if incoherent {
            return Err(Error::Conflict(
                "subject has in-flight destructive or regeneration state".into(),
            ));
        }
        tokio::fs::create_dir_all(root.join("objects"))
            .await
            .map_err(|e| Error::Infrastructure(e.to_string()))?;
        let subject_data = self.subject(subject).await?;
        let seeds = self.seed_history(subject).await?;
        let sources = self.export_sources(subject).await?;
        let artifacts = self.export_artifacts(subject).await?;
        let object_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT object_id FROM memory_objects WHERE subject_id=$1 ORDER BY object_id",
        )
        .bind(subject.0)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?;
        let mut memories = Vec::with_capacity(object_ids.len());
        let mut revision_count = 0;
        for object_id in object_ids {
            let history = self.memory_history(subject, object_id).await?;
            revision_count += history.len();
            let object = sqlx::query("SELECT formation_class,formation_metadata FROM memory_objects WHERE subject_id=$1 AND object_id=$2")
                .bind(subject.0).bind(object_id).fetch_one(self.store.pool()).await.map_err(db)?;
            let accessibility = sqlx::query("SELECT meaningful_uses,last_meaningful_use,retention_hint FROM accessibility_state WHERE object_id=$1")
                .bind(object_id).fetch_one(self.store.pool()).await.map_err(db)?;
            let suppression = sqlx::query(
                "SELECT suppressed,reason,changed_at FROM suppression_state WHERE object_id=$1",
            )
            .bind(object_id)
            .fetch_optional(self.store.pool())
            .await
            .map_err(db)?
            .map(|row| {
                Ok(BundleSuppression {
                    suppressed: row.try_get("suppressed").map_err(db)?,
                    reason: row.try_get("reason").map_err(db)?,
                    changed_at: row.try_get("changed_at").map_err(db)?,
                })
            })
            .transpose()?;
            let episode_ids = sqlx::query_scalar("SELECT episode_id FROM episode_members WHERE subject_id=$1 AND member_id=$2 ORDER BY episode_id")
                .bind(subject.0).bind(object_id).fetch_all(self.store.pool()).await.map_err(db)?;
            let member_ids = sqlx::query_scalar("SELECT member_id FROM episode_members WHERE subject_id=$1 AND episode_id=$2 ORDER BY member_id")
                .bind(subject.0).bind(object_id).fetch_all(self.store.pool()).await.map_err(db)?;
            memories.push(BundleMemory {
                object_id,
                history,
                formation_class: object.try_get("formation_class").map_err(db)?,
                formation_metadata: object.try_get("formation_metadata").map_err(db)?,
                accessibility: BundleAccessibility {
                    meaningful_uses: accessibility.try_get("meaningful_uses").map_err(db)?,
                    last_meaningful_use: accessibility
                        .try_get("last_meaningful_use")
                        .map_err(db)?,
                    retention_hint: accessibility.try_get("retention_hint").map_err(db)?,
                },
                suppression,
                episode_ids,
                member_ids,
            });
        }
        let entities = sqlx::query("SELECT entity_id,entity_kind,label,identity_key FROM memory_entities WHERE subject_id=$1 ORDER BY entity_id").bind(subject.0).fetch_all(self.store.pool()).await.map_err(db)?.into_iter().map(|row| Ok(BundleEntity { entity_id: row.try_get("entity_id").map_err(db)?, entity_kind: row.try_get("entity_kind").map_err(db)?, label: row.try_get("label").map_err(db)?, identity_key: row.try_get("identity_key").map_err(db)? })).collect::<Result<Vec<_>>>()?;
        let derivations = self.export_derivations(subject).await?;
        let use_events = self.export_use_events(subject).await?;
        let associations = self.export_associations(subject).await?;
        let mut hashes = Vec::new();
        for artifact in &artifacts {
            let bytes = self.artifact_bytes(subject, artifact.artifact_id).await?;
            tokio::fs::write(root.join("objects").join(&artifact.content_hash), bytes)
                .await
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
            hashes.push(artifact.content_hash.clone());
        }
        hashes.sort();
        hashes.dedup();
        let manifest = BundleManifest {
            schema_version: VERSION,
            subject_id: subject,
            exported_at: Utc::now(),
            object_hashes: hashes,
            memory_revision_count: revision_count,
        };
        let bundle = SubjectBundle {
            manifest: manifest.clone(),
            subject: subject_data,
            seeds,
            sources,
            artifacts,
            memories,
            entities,
            derivations,
            use_events,
            associations,
        };
        write_json(root, "manifest.json", &bundle.manifest).await?;
        write_json(root, "subject.json", &bundle.subject).await?;
        write_json(root, "seeds.json", &bundle.seeds).await?;
        write_json(root, "sources.json", &bundle.sources).await?;
        write_json(root, "artifacts.json", &bundle.artifacts).await?;
        write_json(root, "memories.json", &bundle.memories).await?;
        write_json(root, "entities.json", &bundle.entities).await?;
        write_json(root, "derivations.json", &bundle.derivations).await?;
        write_json(root, "use-events.json", &bundle.use_events).await?;
        write_json(root, "associations.json", &bundle.associations).await?;
        snapshot_guard.commit().await.map_err(db)?;
        Ok(manifest)
    }

    pub async fn import_bundle(
        &self,
        root: impl AsRef<Path>,
        request: ImportRequest,
    ) -> Result<ImportResult> {
        if request.api_version != nous_core::API_VERSION {
            return Err(Error::Invalid("unsupported API version".into()));
        }
        let _guard = self.objects.reference_guard(false).await?;
        let root = root.as_ref();
        let manifest: BundleManifest = read_json(root, "manifest.json").await?;
        if manifest.schema_version != VERSION {
            return Err(Error::Invalid("unsupported subject bundle schema".into()));
        }
        let subject: Subject = read_json(root, "subject.json").await?;
        let seeds: Vec<SeedRevision> = read_json(root, "seeds.json").await?;
        let sources: Vec<SourceRecord> = read_json(root, "sources.json").await?;
        let artifacts: Vec<Artifact> = read_json(root, "artifacts.json").await?;
        let memories: Vec<BundleMemory> = read_json(root, "memories.json").await?;
        let entities: Vec<BundleEntity> = read_json(root, "entities.json").await?;
        let derivations: Vec<BundleDerivation> = read_json(root, "derivations.json").await?;
        let use_events: Vec<BundleUseEvent> = read_json(root, "use-events.json").await?;
        let associations: Vec<BundleAssociation> = read_json(root, "associations.json").await?;
        for hash in &manifest.object_hashes {
            let bytes = tokio::fs::read(root.join("objects").join(hash))
                .await
                .map_err(|e| Error::Invalid(format!("bundle object {hash} missing: {e}")))?;
            if blake3::hash(&bytes).to_hex().as_str() != hash {
                return Err(Error::Invalid(format!(
                    "bundle object hash mismatch: {hash}"
                )));
            }
            self.objects.put(bytes).await?;
        }
        let imported_subject = if request.preserve_identity {
            subject.subject_id
        } else {
            SubjectId::default()
        };
        if self.subject(imported_subject).await.is_ok() {
            return Err(Error::Conflict("subject identity already exists".into()));
        }
        let remap = !request.preserve_identity;
        let source_map = remap_ids(sources.iter().map(|item| item.source_id), remap);
        let artifact_map = remap_ids(artifacts.iter().map(|item| item.artifact_id), remap);
        let seed_map = remap_ids(seeds.iter().map(|item| item.revision_id), remap);
        let object_map = remap_ids(memories.iter().map(|item| item.object_id), remap);
        let revision_map = remap_ids(
            memories
                .iter()
                .flat_map(|item| item.history.iter().map(|view| view.revision_id)),
            remap,
        );
        let entity_map = remap_ids(entities.iter().map(|item| item.entity_id), remap);
        let derivation_map = remap_ids(
            derivations.iter().map(|item| item.derivation.derivation_id),
            remap,
        );
        let use_event_map = remap_ids(use_events.iter().map(|item| item.event_id), remap);
        let association_map = remap_ids(associations.iter().map(|item| item.evidence_id), remap);
        let sources: Vec<_> = sources
            .iter()
            .map(|source| {
                let mut mapped = source.clone();
                mapped.source_id = map_id(&source_map, source.source_id);
                mapped.parent_sources = source
                    .parent_sources
                    .iter()
                    .map(|id| map_id(&source_map, *id))
                    .collect();
                mapped
            })
            .collect();
        let artifacts: Vec<_> = artifacts
            .iter()
            .map(|artifact| {
                let mut mapped = artifact.clone();
                mapped.artifact_id = map_id(&artifact_map, artifact.artifact_id);
                mapped.source_refs = artifact
                    .source_refs
                    .iter()
                    .map(|id| map_id(&source_map, *id))
                    .collect();
                mapped
            })
            .collect();
        let seeds: Vec<_> = seeds
            .iter()
            .map(|seed| {
                let mut mapped = seed.clone();
                mapped.subject_id = imported_subject;
                mapped.revision_id = map_id(&seed_map, seed.revision_id);
                mapped.artifact_id = map_id(&artifact_map, seed.artifact_id);
                mapped.parent_revision = seed.parent_revision.map(|id| map_id(&seed_map, id));
                mapped
            })
            .collect();
        let memories: Vec<_> = memories
            .iter()
            .map(|bundle| {
                let mut mapped = bundle.clone();
                mapped.object_id = map_id(&object_map, bundle.object_id);
                if let Some(parents) = mapped
                    .formation_metadata
                    .get_mut("parent_objects")
                    .and_then(serde_json::Value::as_array_mut)
                {
                    for parent in parents {
                        if let Some(id) = parent.as_str().and_then(|v| Uuid::parse_str(v).ok()) {
                            *parent = serde_json::json!(map_id(&object_map, id));
                        }
                    }
                }
                for view in &mut mapped.history {
                    view.object_id = mapped.object_id;
                    view.revision_id = map_id(&revision_map, view.revision_id);
                    view.superseded_by = view.superseded_by.map(|id| map_id(&object_map, id));
                    view.parent_revisions = view
                        .parent_revisions
                        .iter()
                        .map(|id| map_id(&revision_map, *id))
                        .collect();
                    view.content.source_refs = view
                        .content
                        .source_refs
                        .iter()
                        .map(|id| map_id(&source_map, *id))
                        .collect();
                    view.content.artifact_refs = view
                        .content
                        .artifact_refs
                        .iter()
                        .map(|id| map_id(&artifact_map, *id))
                        .collect();
                    view.content.entities = view
                        .content
                        .entities
                        .iter()
                        .map(|id| map_id(&entity_map, *id))
                        .collect();
                    view.content.derivation_refs = view
                        .content
                        .derivation_refs
                        .iter()
                        .map(|id| map_id(&derivation_map, *id))
                        .collect();
                }
                mapped.episode_ids = bundle
                    .episode_ids
                    .iter()
                    .map(|id| map_id(&object_map, *id))
                    .collect();
                mapped.member_ids = bundle
                    .member_ids
                    .iter()
                    .map(|id| map_id(&object_map, *id))
                    .collect();
                mapped
            })
            .collect();
        let use_events: Vec<_> = use_events
            .into_iter()
            .map(|mut event| {
                event.event_id = map_id(&use_event_map, event.event_id);
                event.object_refs = event
                    .object_refs
                    .into_iter()
                    .map(|id| map_id(&object_map, id))
                    .collect();
                event.causation_id = event.causation_id.map(|id| map_id(&use_event_map, id));
                event
            })
            .collect();
        let associations: Vec<_> = associations
            .into_iter()
            .map(|mut evidence| {
                evidence.evidence_id = map_id(&association_map, evidence.evidence_id);
                evidence.from_id = remap_node_id(
                    &evidence.from_kind,
                    evidence.from_id,
                    &object_map,
                    &source_map,
                    &artifact_map,
                    &entity_map,
                );
                evidence.to_id = remap_node_id(
                    &evidence.to_kind,
                    evidence.to_id,
                    &object_map,
                    &source_map,
                    &artifact_map,
                    &entity_map,
                );
                evidence.source_id = evidence.source_id.map(|id| map_id(&source_map, id));
                evidence.use_event_id = evidence.use_event_id.map(|id| map_id(&use_event_map, id));
                evidence
            })
            .collect();
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        sqlx::query("INSERT INTO subjects(subject_id,label,metadata,revision,memory_enabled,created_at) VALUES($1,$2,$3,$4,$5,$6)")
            .bind(imported_subject.0).bind(&subject.label).bind(&subject.metadata).bind(subject.revision).bind(subject.memory_enabled).bind(subject.created_at).execute(&mut *tx).await.map_err(db)?;
        for source in &sources {
            self.insert_source(&mut tx, imported_subject, source)
                .await?;
        }
        for entity in &entities {
            sqlx::query("INSERT INTO memory_entities(entity_id,subject_id,entity_kind,label,identity_key) VALUES($1,$2,$3,$4,$5)").bind(entity.entity_id).bind(imported_subject.0).bind(&entity.entity_kind).bind(&entity.label).bind(&entity.identity_key).execute(&mut *tx).await.map_err(db)?;
        }
        for source in &sources {
            for parent in &source.parent_sources {
                if sources
                    .iter()
                    .any(|candidate| candidate.source_id == *parent)
                {
                    sqlx::query("INSERT INTO source_parents(subject_id,source_id,parent_source_id) VALUES($1,$2,$3)").bind(imported_subject.0).bind(source.source_id).bind(parent).execute(&mut *tx).await.map_err(db)?;
                }
            }
        }
        for artifact in &artifacts {
            self.insert_artifact(&mut tx, imported_subject, artifact)
                .await?;
        }
        for seed in &seeds {
            sqlx::query("INSERT INTO character_seed_revisions(revision_id,subject_id,artifact_id,authored_by,parent_revision,created_at) VALUES($1,$2,$3,$4,$5,$6)").bind(seed.revision_id).bind(imported_subject.0).bind(seed.artifact_id).bind(&seed.authored_by).bind(seed.parent_revision).bind(seed.created_at).execute(&mut *tx).await.map_err(db)?;
        }
        if let Some(last) = seeds.iter().max_by_key(|seed| seed.created_at) {
            sqlx::query("INSERT INTO character_seed_heads(subject_id,revision_id) VALUES($1,$2)")
                .bind(imported_subject.0)
                .bind(last.revision_id)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        for item in &derivations {
            let d = &item.derivation;
            let id = map_id(&derivation_map, d.derivation_id);
            sqlx::query("INSERT INTO derivations(derivation_id,subject_id,processor_identity,processor_revision,preprocessing_identity,config_digest,created_at) VALUES($1,$2,$3,$4,$5,$6,$7)")
                .bind(id).bind(imported_subject.0).bind(&d.processor.identity).bind(&d.processor.revision).bind(&d.processor.preprocessing).bind(&d.processor.config_digest).bind(d.created_at).execute(&mut *tx).await.map_err(db)?;
            for artifact in &d.input_artifacts {
                sqlx::query("INSERT INTO derivation_inputs(subject_id,derivation_id,artifact_id) VALUES($1,$2,$3)").bind(imported_subject.0).bind(id).bind(map_id(&artifact_map,*artifact)).execute(&mut *tx).await.map_err(db)?;
            }
            for artifact in &d.output_artifacts {
                sqlx::query("INSERT INTO derivation_outputs(subject_id,derivation_id,artifact_id) VALUES($1,$2,$3)").bind(imported_subject.0).bind(id).bind(map_id(&artifact_map,*artifact)).execute(&mut *tx).await.map_err(db)?;
            }
        }
        for bundle_memory in &memories {
            self.insert_memory(&mut tx, imported_subject, bundle_memory)
                .await?;
        }
        for event in &use_events {
            sqlx::query("INSERT INTO cognitive_use_events(event_id,subject_id,cycle_id,kind,object_refs,occurred_at,causation_id) VALUES($1,$2,NULL,$3,$4,$5,$6)")
                .bind(event.event_id).bind(imported_subject.0).bind(&event.kind).bind(&event.object_refs).bind(event.occurred_at).bind(event.causation_id).execute(&mut *tx).await.map_err(db)?;
        }
        for evidence in &associations {
            sqlx::query("INSERT INTO association_evidence(evidence_id,subject_id,from_kind,from_id,to_kind,to_id,evidence_class,support,source_id,use_event_id,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)")
                .bind(evidence.evidence_id).bind(imported_subject.0).bind(&evidence.from_kind).bind(evidence.from_id).bind(&evidence.to_kind).bind(evidence.to_id).bind(&evidence.evidence_class).bind(evidence.support).bind(evidence.source_id).bind(evidence.use_event_id).bind(evidence.created_at).execute(&mut *tx).await.map_err(db)?;
        }
        tx.commit().await.map_err(db)?;
        drop(_guard);
        let rebuilt = if request.rebuild_projection && self.projection.is_some() {
            self.rebuild_projection(imported_subject).await?
        } else {
            0
        };
        Ok(ImportResult {
            subject_id: imported_subject,
            preserved_identity: request.preserve_identity,
            imported_sources: sources.len(),
            imported_artifacts: artifacts.len(),
            imported_memory_objects: memories.len(),
            rebuilt_projection_rows: rebuilt,
        })
    }

    async fn insert_source(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        subject: SubjectId,
        source: &SourceRecord,
    ) -> Result<()> {
        let range = source.occurred.as_ref();
        sqlx::query("INSERT INTO source_records(source_id,subject_id,source_kind,origin_class,semantic_class,epistemic_class,scope,occurred,approximate_time,observed_at,recorded_at,actor,invocation_id,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,CASE WHEN $8 THEN tstzrange($9,$10,'[]') ELSE NULL END,$11,$12,$13,$14,$15,$16)")
            .bind(source.source_id).bind(subject.0).bind(&source.source_kind).bind(enum_name(&source.classification.origin)?).bind(&source.classification.semantic).bind(enum_name(&source.classification.epistemic)?).bind(&source.scope).bind(range.is_some()).bind(range.and_then(|r| r.start)).bind(range.and_then(|r| r.end)).bind(range.is_some_and(|r| r.approximate)).bind(source.observed_at).bind(source.recorded_at).bind(&source.actor).bind(&source.invocation_id).bind(&source.metadata).execute(&mut **tx).await.map_err(db)?;
        Ok(())
    }

    async fn insert_artifact(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        subject: SubjectId,
        artifact: &Artifact,
    ) -> Result<()> {
        sqlx::query("INSERT INTO artifacts(artifact_id,subject_id,content_hash,media_type,byte_size,origin_class,semantic_class,epistemic_class) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(artifact.artifact_id).bind(subject.0).bind(&artifact.content_hash).bind(&artifact.media_type).bind(artifact.byte_size).bind(enum_name(&artifact.classification.origin)?).bind(&artifact.classification.semantic).bind(enum_name(&artifact.classification.epistemic)?).execute(&mut **tx).await.map_err(db)?;
        for source in &artifact.source_refs {
            sqlx::query(
                "INSERT INTO artifact_sources(subject_id,artifact_id,source_id) VALUES($1,$2,$3)",
            )
            .bind(subject.0)
            .bind(artifact.artifact_id)
            .bind(source)
            .execute(&mut **tx)
            .await
            .map_err(db)?;
        }
        Ok(())
    }

    async fn insert_memory(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        subject: SubjectId,
        bundle: &BundleMemory,
    ) -> Result<()> {
        let current = bundle
            .history
            .last()
            .ok_or_else(|| Error::Invalid("memory bundle has empty history".into()))?;
        sqlx::query("INSERT INTO memory_objects(object_id,subject_id,object_kind,scope,formation_class,formation_metadata,availability,superseded_by,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)")
            .bind(bundle.object_id).bind(subject.0).bind(enum_name(&current.content.kind)?).bind(&current.content.scope).bind(&bundle.formation_class).bind(&bundle.formation_metadata).bind(&current.availability).bind(current.superseded_by).bind(current.recorded_at).execute(&mut **tx).await.map_err(db)?;
        sqlx::query("INSERT INTO accessibility_state(object_id,meaningful_uses,last_meaningful_use,retention_hint) VALUES($1,$2,$3,$4)")
            .bind(bundle.object_id).bind(bundle.accessibility.meaningful_uses).bind(bundle.accessibility.last_meaningful_use).bind(bundle.accessibility.retention_hint).execute(&mut **tx).await.map_err(db)?;
        if let Some(suppression) = &bundle.suppression {
            sqlx::query("INSERT INTO suppression_state(object_id,suppressed,reason,changed_at) VALUES($1,$2,$3,$4)")
                .bind(bundle.object_id).bind(suppression.suppressed).bind(&suppression.reason).bind(suppression.changed_at).execute(&mut **tx).await.map_err(db)?;
        }
        for view in &bundle.history {
            let range = view.content.occurred.as_ref();
            let digest = blake3::hash(view.content.text.as_bytes())
                .to_hex()
                .to_string();
            sqlx::query("INSERT INTO memory_revisions(revision_id,subject_id,object_id,title,representation_text,origin_class,semantic_class,epistemic_class,occurred,approximate_time,observed_at,recorded_at,content_digest,reason) VALUES($1,$2,$3,$4,$5,$6,$7,$8,CASE WHEN $9 THEN tstzrange($10,$11,'[]') ELSE NULL END,$12,$13,$14,$15,$16)")
                .bind(view.revision_id).bind(subject.0).bind(bundle.object_id).bind(&view.content.title).bind(&view.content.text).bind(enum_name(&view.content.classification.origin)?).bind(&view.content.classification.semantic).bind(enum_name(&view.content.classification.epistemic)?).bind(range.is_some()).bind(range.and_then(|r|r.start)).bind(range.and_then(|r|r.end)).bind(range.is_some_and(|r|r.approximate)).bind(view.content.observed_at).bind(view.recorded_at).bind(digest).bind(&view.reason).execute(&mut **tx).await.map_err(db)?;
            for source in &view.content.source_refs {
                sqlx::query("INSERT INTO memory_revision_sources(subject_id,revision_id,source_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(subject.0).bind(view.revision_id).bind(source).execute(&mut **tx).await.map_err(db)?;
            }
            for artifact in &view.content.artifact_refs {
                sqlx::query("INSERT INTO memory_revision_artifacts(subject_id,revision_id,artifact_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(subject.0).bind(view.revision_id).bind(artifact).execute(&mut **tx).await.map_err(db)?;
            }
            for entity in &view.content.entities {
                sqlx::query("INSERT INTO memory_entity_mentions(subject_id,entity_id,revision_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(subject.0).bind(entity).bind(view.revision_id).execute(&mut **tx).await.map_err(db)?;
            }
            for derivation in &view.content.derivation_refs {
                sqlx::query("INSERT INTO memory_revision_derivations(subject_id,revision_id,derivation_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(subject.0).bind(view.revision_id).bind(derivation).execute(&mut **tx).await.map_err(db)?;
            }
            for parent in &view.parent_revisions {
                sqlx::query("INSERT INTO memory_revision_parents(subject_id,revision_id,parent_revision_id,relation) VALUES($1,$2,$3,'CORRECTION') ON CONFLICT DO NOTHING").bind(subject.0).bind(view.revision_id).bind(parent).execute(&mut **tx).await.map_err(db)?;
            }
        }
        sqlx::query("INSERT INTO memory_current_heads(object_id,revision_id) VALUES($1,$2)")
            .bind(bundle.object_id)
            .bind(current.revision_id)
            .execute(&mut **tx)
            .await
            .map_err(db)?;
        for episode in &bundle.episode_ids {
            sqlx::query("INSERT INTO episode_members(subject_id,episode_id,member_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
                .bind(subject.0).bind(episode).bind(bundle.object_id).execute(&mut **tx).await.map_err(db)?;
        }
        for member in &bundle.member_ids {
            sqlx::query("INSERT INTO episode_members(subject_id,episode_id,member_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
                .bind(subject.0).bind(bundle.object_id).bind(member).execute(&mut **tx).await.map_err(db)?;
        }
        Ok(())
    }

    async fn export_sources(&self, subject: SubjectId) -> Result<Vec<SourceRecord>> {
        let rows=sqlx::query("SELECT *,lower(occurred) AS occurred_start,upper(occurred) AS occurred_end,occurred IS NOT NULL AS has_occurred FROM source_records WHERE subject_id=$1 ORDER BY source_id").bind(subject.0).fetch_all(self.store.pool()).await.map_err(db)?;
        let mut result = Vec::new();
        for row in rows {
            let source_id: Uuid = row.try_get("source_id").map_err(db)?;
            let parents=sqlx::query_scalar("SELECT parent_source_id FROM source_parents WHERE subject_id=$1 AND source_id=$2 ORDER BY parent_source_id").bind(subject.0).bind(source_id).fetch_all(self.store.pool()).await.map_err(db)?;
            let occurred = if row.try_get("has_occurred").map_err(db)? {
                Some(TemporalRange {
                    start: row.try_get("occurred_start").map_err(db)?,
                    end: row.try_get("occurred_end").map_err(db)?,
                    approximate: row.try_get("approximate_time").map_err(db)?,
                })
            } else {
                None
            };
            result.push(SourceRecord {
                source_id,
                subject_id: subject,
                source_kind: row.try_get("source_kind").map_err(db)?,
                classification: Classification {
                    origin: decode_enum(row.try_get("origin_class").map_err(db)?)?,
                    semantic: row.try_get("semantic_class").map_err(db)?,
                    epistemic: decode_enum(row.try_get("epistemic_class").map_err(db)?)?,
                },
                scope: row.try_get("scope").map_err(db)?,
                occurred,
                observed_at: row.try_get("observed_at").map_err(db)?,
                recorded_at: row.try_get("recorded_at").map_err(db)?,
                actor: row.try_get("actor").map_err(db)?,
                invocation_id: row.try_get("invocation_id").map_err(db)?,
                parent_sources: parents,
                metadata: row.try_get("metadata").map_err(db)?,
            });
        }
        Ok(result)
    }

    async fn export_artifacts(&self, subject: SubjectId) -> Result<Vec<Artifact>> {
        let rows = sqlx::query("SELECT * FROM artifacts WHERE subject_id=$1 ORDER BY artifact_id")
            .bind(subject.0)
            .fetch_all(self.store.pool())
            .await
            .map_err(db)?;
        let mut result = Vec::new();
        for row in rows {
            let id: Uuid = row.try_get("artifact_id").map_err(db)?;
            let sources=sqlx::query_scalar("SELECT source_id FROM artifact_sources WHERE subject_id=$1 AND artifact_id=$2 ORDER BY source_id").bind(subject.0).bind(id).fetch_all(self.store.pool()).await.map_err(db)?;
            result.push(Artifact {
                artifact_id: id,
                subject_id: subject,
                content_hash: row.try_get("content_hash").map_err(db)?,
                media_type: row.try_get("media_type").map_err(db)?,
                byte_size: row.try_get("byte_size").map_err(db)?,
                classification: Classification {
                    origin: decode_enum(row.try_get("origin_class").map_err(db)?)?,
                    semantic: row.try_get("semantic_class").map_err(db)?,
                    epistemic: decode_enum(row.try_get("epistemic_class").map_err(db)?)?,
                },
                source_refs: sources,
                created_at: row.try_get("created_at").map_err(db)?,
            });
        }
        Ok(result)
    }

    async fn export_derivations(&self, subject: SubjectId) -> Result<Vec<BundleDerivation>> {
        let rows =
            sqlx::query("SELECT * FROM derivations WHERE subject_id=$1 ORDER BY derivation_id")
                .bind(subject.0)
                .fetch_all(self.store.pool())
                .await
                .map_err(db)?;
        let mut result = Vec::new();
        for row in rows {
            let id: Uuid = row.try_get("derivation_id").map_err(db)?;
            let inputs: Vec<Uuid> = sqlx::query_scalar("SELECT artifact_id FROM derivation_inputs WHERE subject_id=$1 AND derivation_id=$2 ORDER BY artifact_id").bind(subject.0).bind(id).fetch_all(self.store.pool()).await.map_err(db)?;
            let outputs: Vec<Uuid> = sqlx::query_scalar("SELECT artifact_id FROM derivation_outputs WHERE subject_id=$1 AND derivation_id=$2 ORDER BY artifact_id").bind(subject.0).bind(id).fetch_all(self.store.pool()).await.map_err(db)?;
            result.push(BundleDerivation {
                derivation: Derivation {
                    derivation_id: id,
                    subject_id: subject,
                    input_artifacts: inputs,
                    output_artifacts: outputs,
                    processor: ProcessorProvenance {
                        identity: row.try_get("processor_identity").map_err(db)?,
                        revision: row.try_get("processor_revision").map_err(db)?,
                        preprocessing: row.try_get("preprocessing_identity").map_err(db)?,
                        config_digest: row.try_get("config_digest").map_err(db)?,
                    },
                    created_at: row.try_get("created_at").map_err(db)?,
                },
            });
        }
        Ok(result)
    }

    async fn export_use_events(&self, subject: SubjectId) -> Result<Vec<BundleUseEvent>> {
        let rows = sqlx::query("SELECT event_id,kind,object_refs,occurred_at,causation_id FROM cognitive_use_events WHERE subject_id=$1 AND kind IN ('FOLLOWED','REFERENCED_OR_ACTED_ON') ORDER BY occurred_at,event_id")
            .bind(subject.0).fetch_all(self.store.pool()).await.map_err(db)?;
        rows.into_iter()
            .map(|row| {
                Ok(BundleUseEvent {
                    event_id: row.try_get("event_id").map_err(db)?,
                    kind: row.try_get("kind").map_err(db)?,
                    object_refs: row.try_get("object_refs").map_err(db)?,
                    occurred_at: row.try_get("occurred_at").map_err(db)?,
                    causation_id: row.try_get("causation_id").map_err(db)?,
                })
            })
            .collect()
    }

    async fn export_associations(&self, subject: SubjectId) -> Result<Vec<BundleAssociation>> {
        let rows = sqlx::query("SELECT evidence_id,from_kind,from_id,to_kind,to_id,evidence_class,support,source_id,use_event_id,created_at FROM association_evidence WHERE subject_id=$1 ORDER BY evidence_id")
            .bind(subject.0).fetch_all(self.store.pool()).await.map_err(db)?;
        rows.into_iter()
            .map(|row| {
                Ok(BundleAssociation {
                    evidence_id: row.try_get("evidence_id").map_err(db)?,
                    from_kind: row.try_get("from_kind").map_err(db)?,
                    from_id: row.try_get("from_id").map_err(db)?,
                    to_kind: row.try_get("to_kind").map_err(db)?,
                    to_id: row.try_get("to_id").map_err(db)?,
                    evidence_class: row.try_get("evidence_class").map_err(db)?,
                    support: row.try_get("support").map_err(db)?,
                    source_id: row.try_get("source_id").map_err(db)?,
                    use_event_id: row.try_get("use_event_id").map_err(db)?,
                    created_at: row.try_get("created_at").map_err(db)?,
                })
            })
            .collect()
    }
}

fn remap_ids<I>(ids: I, remap: bool) -> HashMap<Uuid, Uuid>
where
    I: IntoIterator<Item = Uuid>,
{
    ids.into_iter()
        .map(|id| (id, if remap { Uuid::now_v7() } else { id }))
        .collect()
}
fn map_id(map: &HashMap<Uuid, Uuid>, id: Uuid) -> Uuid {
    map.get(&id).copied().unwrap_or(id)
}
fn remap_node_id(
    kind: &str,
    id: Uuid,
    objects: &HashMap<Uuid, Uuid>,
    sources: &HashMap<Uuid, Uuid>,
    artifacts: &HashMap<Uuid, Uuid>,
    entities: &HashMap<Uuid, Uuid>,
) -> Uuid {
    match kind {
        "memory" => map_id(objects, id),
        "source" => map_id(sources, id),
        "artifact" => map_id(artifacts, id),
        "entity" => map_id(entities, id),
        _ => id,
    }
}
