use crate::{
    LocalRuntime,
    subjects::{check_version, db},
};
use chrono::{DateTime, Utc};
use nous_core::{Error, Result, SubjectId};
use nous_material::{
    Artifact, Classification, Derivation, EpistemicClass, OriginClass, ProcessorProvenance,
    TemporalRange,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaterialContent {
    Text { text: String },
    Json { value: serde_json::Value },
    Bytes { bytes: Vec<u8> },
    Artifact { artifact_id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestMaterial {
    pub api_version: u32,
    pub source_kind: String,
    pub classification: Classification,
    pub media_type: String,
    pub scope: String,
    pub occurred: Option<TemporalRange>,
    pub observed_at: Option<DateTime<Utc>>,
    pub actor: Option<String>,
    pub invocation_id: Option<String>,
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub parent_sources: Vec<Uuid>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub content: MaterialContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedMaterial {
    pub api_version: u32,
    pub source_id: Uuid,
    pub artifact_id: Uuid,
    pub obligation_id: Option<Uuid>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolObservation {
    pub api_version: u32,
    pub tool_identity: String,
    pub operation: String,
    pub invocation_id: String,
    pub arguments_ref_or_redacted_metadata: serde_json::Value,
    pub result: MaterialContent,
    pub media_type: String,
    pub outcome: ToolOutcome,
    pub invoked_at: Option<DateTime<Utc>>,
    pub observed_at: Option<DateTime<Utc>>,
    pub scope: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ToolOutcome {
    Success,
    Partial,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitDerivation {
    pub api_version: u32,
    pub input_artifacts: Vec<Uuid>,
    pub processor: ProcessorProvenance,
    pub classification: Classification,
    pub media_type: String,
    pub content: MaterialContent,
}

impl LocalRuntime {
    pub async fn submit_derivation(
        &self,
        subject: SubjectId,
        input: SubmitDerivation,
    ) -> Result<Derivation> {
        let _objects = self.objects.reference_guard(false).await?;
        check_version(input.api_version)?;
        input.classification.validate()?;
        if input.input_artifacts.is_empty()
            || input.input_artifacts.len() > 1000
            || input.processor.identity.is_empty()
            || input.processor.revision.is_empty()
            || input.processor.preprocessing.is_empty()
            || input.processor.config_digest.is_empty()
        {
            return Err(Error::Invalid(
                "derivation needs bounded input refs and complete processor provenance".into(),
            ));
        }
        if !matches!(
            input.classification.epistemic,
            EpistemicClass::Derived
                | EpistemicClass::Inferred
                | EpistemicClass::Narrative
                | EpistemicClass::Simulated
        ) {
            return Err(Error::Invalid("a processor output cannot be classified as directly observed or reported source evidence".into()));
        }
        if matches!(input.content, MaterialContent::Artifact { .. }) {
            return Err(Error::Invalid(
                "submit new derived content; an existing artifact is immutable".into(),
            ));
        }
        for artifact in &input.input_artifacts {
            self.artifact(subject, *artifact).await?;
        }
        let prepared = self
            .prepare_artifact(subject, &input.media_type, &input.content)
            .await?;
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        sqlx::query("SELECT subject_id FROM subjects WHERE subject_id=$1 FOR UPDATE")
            .bind(subject.0)
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?;
        let output = write_artifact(&mut tx, subject, prepared, &input.classification).await?;
        let id = Uuid::now_v7();
        let created: DateTime<Utc> = sqlx::query_scalar("INSERT INTO derivations(derivation_id,subject_id,processor_identity,processor_revision,preprocessing_identity,config_digest) VALUES($1,$2,$3,$4,$5,$6) RETURNING created_at")
            .bind(id).bind(subject.0).bind(&input.processor.identity).bind(&input.processor.revision).bind(&input.processor.preprocessing).bind(&input.processor.config_digest)
            .fetch_one(&mut *tx).await.map_err(db)?;
        for artifact in &input.input_artifacts {
            sqlx::query("INSERT INTO derivation_inputs(subject_id,derivation_id,artifact_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
                .bind(subject.0).bind(id).bind(artifact).execute(&mut *tx).await.map_err(db)?;
        }
        sqlx::query(
            "INSERT INTO derivation_outputs(subject_id,derivation_id,artifact_id) VALUES($1,$2,$3)",
        )
        .bind(subject.0)
        .bind(id)
        .bind(output)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        sqlx::query("INSERT INTO artifact_sources(subject_id,artifact_id,source_id) SELECT $1,$2,source_id FROM artifact_sources WHERE subject_id=$1 AND artifact_id=ANY($3) ON CONFLICT DO NOTHING")
            .bind(subject.0).bind(output).bind(&input.input_artifacts).execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)?;
        Ok(Derivation {
            derivation_id: id,
            subject_id: subject,
            input_artifacts: input.input_artifacts,
            output_artifacts: vec![output],
            processor: input.processor,
            created_at: created,
        })
    }

    pub async fn ingest(
        &self,
        subject: SubjectId,
        input: IngestMaterial,
    ) -> Result<AcceptedMaterial> {
        let _objects = self.objects.reference_guard(false).await?;
        check_version(input.api_version)?;
        self.subject(subject).await?;
        input.classification.validate()?;
        if let Some(range) = &input.occurred {
            range.validate()?;
        }
        if input.source_kind.is_empty()
            || input.source_kind.len() > 128
            || input.scope.is_empty()
            || input.scope.len() > 1024
            || input.media_type.is_empty()
            || input.media_type.len() > 256
            || input.metadata.to_string().len() > 65536
        {
            return Err(Error::Invalid(
                "invalid source kind, scope, media type or metadata size".into(),
            ));
        }
        if input
            .idempotency_key
            .as_ref()
            .is_some_and(|key| key.is_empty() || key.len() > 1024)
        {
            return Err(Error::Invalid("invalid idempotency key".into()));
        }
        // Retry identity belongs to the subject, not the content hash.
        if let Some(key) = &input.idempotency_key
            && let Some(source) = sqlx::query_scalar::<_, Uuid>(
                "SELECT source_id FROM source_records WHERE subject_id=$1 AND idempotency_key=$2",
            )
            .bind(subject.0)
            .bind(key)
            .fetch_optional(self.store.pool())
            .await
            .map_err(db)?
        {
            return self.material_status(subject, source).await;
        }
        let prepared = self
            .prepare_artifact(subject, &input.media_type, &input.content)
            .await?;
        let mut tx = self.store.pool().begin().await.map_err(db)?;
        // A row lock fences purge against late ingest commits for this subject.
        let memory_enabled: bool = sqlx::query_scalar(
            "SELECT memory_enabled FROM subjects WHERE subject_id=$1 FOR UPDATE",
        )
        .bind(subject.0)
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
        if let Some(key) = &input.idempotency_key
            && let Some(source) = sqlx::query_scalar::<_, Uuid>(
                "SELECT source_id FROM source_records WHERE subject_id=$1 AND idempotency_key=$2",
            )
            .bind(subject.0)
            .bind(key)
            .fetch_optional(&mut *tx)
            .await
            .map_err(db)?
        {
            tx.rollback().await.map_err(db)?;
            return self.material_status(subject, source).await;
        }
        let source = Uuid::now_v7();
        let start = input.occurred.as_ref().and_then(|range| range.start);
        let end = input.occurred.as_ref().and_then(|range| range.end);
        sqlx::query("INSERT INTO source_records(source_id,subject_id,source_kind,origin_class,semantic_class,epistemic_class,scope,occurred,approximate_time,observed_at,actor,invocation_id,idempotency_key,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,CASE WHEN $8 THEN tstzrange($9,$10,'[]') ELSE NULL END,$11,$12,$13,$14,$15,$16)")
            .bind(source).bind(subject.0).bind(&input.source_kind).bind(enum_name(&input.classification.origin)?).bind(&input.classification.semantic)
            .bind(enum_name(&input.classification.epistemic)?).bind(&input.scope).bind(input.occurred.is_some()).bind(start).bind(end)
            .bind(input.occurred.as_ref().is_some_and(|range| range.approximate)).bind(input.observed_at).bind(input.actor).bind(input.invocation_id)
            .bind(input.idempotency_key).bind(input.metadata).execute(&mut *tx).await.map_err(db)?;
        for parent in input.parent_sources {
            sqlx::query("INSERT INTO source_parents(subject_id,source_id,parent_source_id) VALUES($1,$2,$3)")
                .bind(subject.0).bind(source).bind(parent).execute(&mut *tx).await.map_err(db)?;
        }
        let artifact = write_artifact(&mut tx, subject, prepared, &input.classification).await?;
        sqlx::query(
            "INSERT INTO artifact_sources(subject_id,artifact_id,source_id) VALUES($1,$2,$3)",
        )
        .bind(subject.0)
        .bind(artifact)
        .bind(source)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        if memory_enabled {
            sqlx::query("INSERT INTO processing_obligations(obligation_id,subject_id,kind,payload_version,source_id,payload) VALUES($1,$2,'ingest',1,$3,'{}')")
                .bind(Uuid::now_v7()).bind(subject.0).bind(source).execute(&mut *tx).await.map_err(db)?;
            sqlx::query("SELECT pg_notify('nous_memory_processing', '')")
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        tx.commit().await.map_err(db)?;
        self.material_status(subject, source).await
    }

    pub async fn material_status(
        &self,
        subject: SubjectId,
        source: Uuid,
    ) -> Result<AcceptedMaterial> {
        let row = sqlx::query("SELECT s.source_id,a.artifact_id,o.obligation_id,o.state FROM source_records s JOIN artifact_sources a USING(subject_id,source_id) LEFT JOIN processing_obligations o ON o.subject_id=s.subject_id AND o.source_id=s.source_id AND o.kind='ingest' WHERE s.subject_id=$1 AND s.source_id=$2 ORDER BY a.artifact_id LIMIT 1")
            .bind(subject.0).bind(source).fetch_optional(self.store.pool()).await.map_err(db)?
            .ok_or_else(|| Error::NotFound("source not found".into()))?;
        let state: Option<String> = row.try_get("state").map_err(db)?;
        Ok(AcceptedMaterial {
            api_version: 1,
            source_id: source,
            artifact_id: row.try_get("artifact_id").map_err(db)?,
            obligation_id: row.try_get("obligation_id").map_err(db)?,
            status: match state.as_deref() {
                Some("succeeded") => "ready",
                Some("running") => "processing",
                Some("failed") => "failed",
                Some(_) => "degraded",
                None => "recorded",
            }
            .into(),
        })
    }

    pub async fn record_tool_observation(
        &self,
        subject: SubjectId,
        input: ToolObservation,
    ) -> Result<AcceptedMaterial> {
        if input.tool_identity.is_empty()
            || input.operation.is_empty()
            || input.invocation_id.is_empty()
        {
            return Err(Error::Invalid(
                "tool, operation and invocation identities are required".into(),
            ));
        }
        self.ingest(subject, IngestMaterial {
            api_version: input.api_version, source_kind: "tool:observation".into(),
            classification: Classification { origin: OriginClass::Tool, semantic: "TOOL_OBSERVATION".into(), epistemic: EpistemicClass::Reported },
            media_type: input.media_type, scope: input.scope, occurred: None, observed_at: input.observed_at,
            actor: Some(input.tool_identity.clone()), invocation_id: Some(input.invocation_id.clone()),
            idempotency_key: Some(format!("tool:{}:{}:{}", input.tool_identity, input.operation, input.invocation_id)), parent_sources: vec![],
            metadata: serde_json::json!({"tool": input.tool_identity, "operation": input.operation, "outcome": input.outcome,
                "invoked_at": input.invoked_at, "arguments_ref_or_redacted_metadata": input.arguments_ref_or_redacted_metadata}), content: input.result,
        }).await
    }

    pub async fn artifact(&self, subject: SubjectId, artifact: Uuid) -> Result<Artifact> {
        let row = sqlx::query("SELECT * FROM artifacts WHERE subject_id=$1 AND artifact_id=$2")
            .bind(subject.0)
            .bind(artifact)
            .fetch_optional(self.store.pool())
            .await
            .map_err(db)?
            .ok_or_else(|| Error::NotFound("artifact not found".into()))?;
        let sources = sqlx::query_scalar("SELECT source_id FROM artifact_sources WHERE subject_id=$1 AND artifact_id=$2 ORDER BY source_id")
            .bind(subject.0).bind(artifact).fetch_all(self.store.pool()).await.map_err(db)?;
        Ok(Artifact {
            artifact_id: artifact,
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
        })
    }

    pub async fn artifact_bytes(&self, subject: SubjectId, artifact: Uuid) -> Result<Vec<u8>> {
        let artifact = self.artifact(subject, artifact).await?;
        let inline: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT inline_payload FROM artifacts WHERE subject_id=$1 AND artifact_id=$2",
        )
        .bind(subject.0)
        .bind(artifact.artifact_id)
        .fetch_one(self.store.pool())
        .await
        .map_err(db)?;
        let bytes = match inline {
            Some(bytes) => bytes,
            None => self.objects.get(&artifact.content_hash).await?,
        };
        if bytes.len() as i64 != artifact.byte_size
            || blake3::hash(&bytes).to_hex().as_str() != artifact.content_hash
        {
            return Err(Error::Infrastructure(
                "artifact integrity check failed".into(),
            ));
        }
        Ok(bytes)
    }

    pub(crate) async fn prepare_artifact(
        &self,
        subject: SubjectId,
        media_type: &str,
        content: &MaterialContent,
    ) -> Result<PreparedArtifact> {
        let bytes = match content {
            MaterialContent::Text { text } => text.as_bytes().to_vec(),
            MaterialContent::Json { value } => {
                serde_json::to_vec(value).map_err(|e| Error::Invalid(e.to_string()))?
            }
            MaterialContent::Bytes { bytes } => bytes.clone(),
            MaterialContent::Artifact { artifact_id } => {
                self.artifact(subject, *artifact_id).await?;
                return Ok(PreparedArtifact::Existing(*artifact_id));
            }
        };
        let size = bytes.len() as i64;
        let hash = self.objects.put(bytes).await?;
        Ok(PreparedArtifact::New {
            id: Uuid::now_v7(),
            hash,
            size,
            media_type: media_type.into(),
        })
    }
}

pub(crate) enum PreparedArtifact {
    New {
        id: Uuid,
        hash: String,
        size: i64,
        media_type: String,
    },
    Existing(Uuid),
}

pub(crate) async fn write_artifact(
    tx: &mut Transaction<'_, Postgres>,
    subject: SubjectId,
    prepared: PreparedArtifact,
    classification: &Classification,
) -> Result<Uuid> {
    match prepared {
        PreparedArtifact::Existing(id) => Ok(id),
        PreparedArtifact::New {
            id,
            hash,
            size,
            media_type,
        } => {
            sqlx::query("INSERT INTO artifacts(artifact_id,subject_id,content_hash,media_type,byte_size,origin_class,semantic_class,epistemic_class) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
                .bind(id).bind(subject.0).bind(hash).bind(media_type).bind(size).bind(enum_name(&classification.origin)?).bind(&classification.semantic).bind(enum_name(&classification.epistemic)?)
                .execute(&mut **tx).await.map_err(db)?;
            Ok(id)
        }
    }
}

pub(crate) fn enum_name(value: &impl Serialize) -> Result<String> {
    serde_json::to_value(value)
        .map_err(|e| Error::Infrastructure(e.to_string()))?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| Error::Infrastructure("expected string enumeration".into()))
}
pub(crate) fn decode_enum<T: serde::de::DeserializeOwned>(value: String) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(value))
        .map_err(|e| Error::Infrastructure(e.to_string()))
}
