use nous_authority_store::database_error as db;
use super::*;

impl MaterialService {
    pub async fn persist_derived_representation(
        &self,
        representation: DerivedRepresentation,
    ) -> Result<DerivedRepresentation> {
        representation.validate()?;
        let region_subject: Option<Uuid> =
            sqlx::query_scalar("SELECT subject_id FROM source_regions WHERE source_region_id=$1")
                .bind(representation.source_region_id.0)
                .fetch_optional(self.store.pool())
                .await
                .map_err(db)?;
        if region_subject != Some(representation.subject_id.0) {
            return Err(Error::Invalid("source region is outside Subject".into()));
        }
        let mut tx = self.store.begin().await?;
        let producer_id: Uuid = sqlx::query_scalar("INSERT INTO producer_signatures(producer_signature_id,signature_hash,provider_class,operation,implementation,model_identity,model_revision,preprocessing_identity,preprocessing_revision,config_digest,created_at,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) ON CONFLICT(signature_hash) DO UPDATE SET signature_hash=excluded.signature_hash RETURNING producer_signature_id")
            .bind(Uuid::now_v7())
            .bind(&representation.producer.signature_hash)
            .bind(&representation.producer.provider_class)
            .bind(representation.producer.operation.as_str())
            .bind(&representation.producer.implementation)
            .bind(&representation.producer.model_identity)
            .bind(&representation.producer.model_revision)
            .bind(&representation.producer.preprocessing_identity)
            .bind(&representation.producer.preprocessing_revision)
            .bind(&representation.producer.config_digest)
            .bind(representation.created_at)
            .bind(serde_json::json!({}))
            .fetch_one(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("INSERT INTO derived_representations(derived_representation_id,subject_id,source_region_id,representation_kind,producer_signature_id,revision,payload_text,payload_artifact_id,quality,created_at,supersedes) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)")
            .bind(representation.derived_representation_id.0)
            .bind(representation.subject_id.0)
            .bind(representation.source_region_id.0)
            .bind(representation.representation_kind.as_str())
            .bind(producer_id)
            .bind(representation.revision)
            .bind(&representation.payload_text)
            .bind(representation.payload_artifact_id.map(|id| id.0))
            .bind(&representation.quality)
            .bind(representation.created_at)
            .bind(representation.supersedes.map(|id| id.0))
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("UPDATE derivations SET state='succeeded',successful_representation_id=$4,updated_at=$5 WHERE subject_id=$1 AND source_region_id=$2 AND representation_kind=$3 AND producer_signature_id=$6")
            .bind(representation.subject_id.0)
            .bind(representation.source_region_id.0)
            .bind(representation.representation_kind.as_str())
            .bind(representation.derived_representation_id.0)
            .bind(Utc::now())
            .bind(producer_id)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("UPDATE coverage_needs SET state='ready',current_representation_id=$4,updated_at=$5 WHERE subject_id=$1 AND source_region_id=$2 AND representation_kind=$3 AND capability_operation=$6")
            .bind(representation.subject_id.0)
            .bind(representation.source_region_id.0)
            .bind(representation.representation_kind.as_str())
            .bind(representation.derived_representation_id.0)
            .bind(Utc::now())
            .bind(representation.producer.operation.as_str())
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        tx.commit().await.map_err(db)?;
        self.store.invalidate(representation.subject_id, ProjectionInvalidation::text()).await?;
        Ok(representation)
    }

    /// Create one explicit derivation obligation.  This records work already
    /// requested by a Host/configuration; it does not schedule cognition.
    pub async fn schedule_derivation(
        &self,
        subject: SubjectId,
        source_region: SourceRegionId,
        representation_kind: RepresentationKind,
        operation: CapabilityOperation,
        producer: ProducerSignature,
        requirement: &str,
    ) -> Result<DerivationId> {
        self.store.require_subject(subject).await?;
        if !matches!(requirement, "required" | "preferred" | "opportunistic") {
            return Err(Error::Invalid("invalid derivation requirement".into()));
        }
        if producer.operation != operation {
            return Err(Error::Invalid(
                "producer operation does not match derivation capability".into(),
            ));
        }
        let source = sqlx::query(
            "SELECT subject_id,artifact_id FROM source_regions WHERE source_region_id=$1",
        )
        .bind(source_region.0)
        .fetch_optional(self.store.pool())
        .await
        .map_err(db)?
        .ok_or_else(|| Error::NotFound("source region not found".into()))?;
        let source_subject: Uuid = source.try_get("subject_id").map_err(db)?;
        if source_subject != subject.0 {
            return Err(Error::Invalid("source region is outside Subject".into()));
        }
        let artifact_id: Uuid = source.try_get("artifact_id").map_err(db)?;
        let artifact_hash: String = sqlx::query_scalar(
            "SELECT content_hash FROM artifacts WHERE artifact_id=$1 AND subject_id=$2",
        )
        .bind(artifact_id)
        .bind(subject.0)
        .fetch_one(self.store.pool())
        .await
        .map_err(db)?;
        let producer_id: Uuid = sqlx::query_scalar("INSERT INTO producer_signatures(producer_signature_id,signature_hash,provider_class,operation,implementation,model_identity,model_revision,preprocessing_identity,preprocessing_revision,config_digest,created_at,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'{}') ON CONFLICT(signature_hash) DO UPDATE SET signature_hash=excluded.signature_hash RETURNING producer_signature_id")
            .bind(Uuid::now_v7())
            .bind(&producer.signature_hash)
            .bind(&producer.provider_class)
            .bind(producer.operation.as_str())
            .bind(&producer.implementation)
            .bind(&producer.model_identity)
            .bind(&producer.model_revision)
            .bind(&producer.preprocessing_identity)
            .bind(&producer.preprocessing_revision)
            .bind(&producer.config_digest)
            .bind(Utc::now())
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
        let derivation_key = blake3::hash(
            format!(
                "nous-wave-derivation-v1\0{}\0{}\0{}\0{}\0{}",
                source_region.0,
                artifact_hash,
                representation_kind.as_str(),
                producer.signature_hash,
                operation.as_str()
            )
            .as_bytes(),
        )
        .to_hex()
        .to_string();
        let derivation_id: Uuid = sqlx::query_scalar("INSERT INTO derivations(derivation_id,derivation_key,subject_id,source_region_id,representation_kind,producer_signature_id,state,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,'pending',$7,$7) ON CONFLICT(derivation_key) DO UPDATE SET updated_at=excluded.updated_at RETURNING derivation_id")
            .bind(Uuid::now_v7())
            .bind(derivation_key)
            .bind(subject.0)
            .bind(source_region.0)
            .bind(representation_kind.as_str())
            .bind(producer_id)
            .bind(Utc::now())
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
        sqlx::query("INSERT INTO coverage_needs(coverage_need_id,subject_id,source_region_id,representation_kind,capability_operation,requirement,state,updated_at) VALUES($1,$2,$3,$4,$5,$6,'scheduled',$7) ON CONFLICT(subject_id,source_region_id,representation_kind,capability_operation) DO UPDATE SET requirement=excluded.requirement,state=CASE WHEN coverage_needs.state='ready' THEN 'ready' ELSE 'scheduled' END,updated_at=excluded.updated_at")
            .bind(Uuid::now_v7())
            .bind(subject.0)
            .bind(source_region.0)
            .bind(representation_kind.as_str())
            .bind(operation.as_str())
            .bind(requirement)
            .bind(Utc::now())
            .execute(self.store.pool())
            .await
            .map_err(db)?;
        Ok(DerivationId(derivation_id))
    }

    pub async fn persist_derived_region(&self, region: DerivedRegion) -> Result<DerivedRegion> {
        region.validate()?;
        let valid: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM derived_representations WHERE subject_id=$1 AND derived_representation_id=$2)")
            .bind(region.subject_id.0)
            .bind(region.derived_representation_id.0)
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
        if !valid
            || region.coordinate_kind.trim().is_empty()
            || region.coordinate_hash.trim().is_empty()
        {
            return Err(Error::Invalid(
                "derived region is invalid or outside Subject".into(),
            ));
        }
        if let Some(parent) = region.parent_derived_region_id {
            let parent_valid: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM derived_regions WHERE subject_id=$1 AND derived_region_id=$2 AND derived_representation_id=$3)",
            )
            .bind(region.subject_id.0)
            .bind(parent.0)
            .bind(region.derived_representation_id.0)
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
            if !parent_valid {
                return Err(Error::Invalid(
                    "derived region parent is outside the representation".into(),
                ));
            }
        }
        sqlx::query("INSERT INTO derived_regions(derived_region_id,subject_id,derived_representation_id,coordinate_kind,coordinate,coordinate_hash,parent_derived_region_id,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT(derived_representation_id,coordinate_kind,coordinate_hash) DO NOTHING")
            .bind(region.derived_region_id.0)
            .bind(region.subject_id.0)
            .bind(region.derived_representation_id.0)
            .bind(&region.coordinate_kind)
            .bind(&region.coordinate)
            .bind(&region.coordinate_hash)
            .bind(region.parent_derived_region_id.map(|id| id.0))
            .bind(region.created_at)
            .execute(self.store.pool())
            .await
            .map_err(db)?;
        Ok(region)
    }

    pub async fn claim_derivations(
        &self,
        limit: usize,
        lease_owner: &str,
        lease_for: std::time::Duration,
    ) -> Result<Vec<DerivationClaim>> {
        if limit == 0 || limit > 1024 || lease_owner.trim().is_empty() {
            return Err(Error::Invalid("derivation claim bounds are invalid".into()));
        }
        let lease_until = Utc::now()
            + chrono::Duration::from_std(lease_for)
                .map_err(|error| Error::Invalid(error.to_string()))?;
        let mut tx = self.store.begin().await?;
        let rows = sqlx::query("SELECT d.derivation_id,d.subject_id,d.source_region_id,d.representation_kind,d.producer_signature_id FROM derivations d WHERE d.state='pending' OR (d.state='running' AND NOT EXISTS (SELECT 1 FROM derivation_attempts a WHERE a.derivation_id=d.derivation_id AND a.state='running' AND a.lease_until > now())) ORDER BY d.created_at FOR UPDATE OF d SKIP LOCKED LIMIT $1")
            .bind(limit as i64)
            .fetch_all(&mut *tx)
            .await
            .map_err(db)?;
        let mut claims = Vec::new();
        for row in rows {
            let derivation_id = DerivationId(row.try_get("derivation_id").map_err(db)?);
            let attempt_no: i32 = sqlx::query_scalar("SELECT COALESCE(max(attempt_no),0)+1 FROM derivation_attempts WHERE derivation_id=$1")
                .bind(derivation_id.0)
                .fetch_one(&mut *tx)
                .await
                .map_err(db)?;
            let attempt_id = Uuid::now_v7();
            let now = Utc::now();
            sqlx::query("INSERT INTO derivation_attempts(attempt_id,derivation_id,attempt_no,state,lease_owner,lease_until,started_at) VALUES($1,$2,$3,'running',$4,$5,$6)")
                .bind(attempt_id).bind(derivation_id.0).bind(attempt_no).bind(lease_owner).bind(lease_until).bind(now).execute(&mut *tx).await.map_err(db)?;
            sqlx::query(
                "UPDATE derivations SET state='running',updated_at=$2 WHERE derivation_id=$1",
            )
            .bind(derivation_id.0)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
            sqlx::query("UPDATE coverage_needs SET state='scheduled',updated_at=$3 WHERE subject_id=$1 AND source_region_id=$2 AND representation_kind=$4")
                .bind(row.try_get::<Uuid, _>("subject_id").map_err(db)?)
                .bind(row.try_get::<Uuid, _>("source_region_id").map_err(db)?)
                .bind(now)
                .bind(row.try_get::<String, _>("representation_kind").map_err(db)?)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            claims.push(DerivationClaim {
                derivation_id,
                subject_id: SubjectId(row.try_get("subject_id").map_err(db)?),
                source_region_id: SourceRegionId(row.try_get("source_region_id").map_err(db)?),
                representation_kind: row.try_get("representation_kind").map_err(db)?,
                producer_signature_id: row.try_get("producer_signature_id").map_err(db)?,
                attempt_id,
                attempt_no,
                lease_until,
            });
        }
        tx.commit().await.map_err(db)?;
        Ok(claims)
    }

    /// Execute already-created document derivations with the supplied narrow
    /// provider. The provider call is outside the database transaction; only
    /// the lease/commit records are transactional.
    pub async fn run_pending_derivations(
        &self,
        limit: usize,
        lease_owner: &str,
        provider: Arc<dyn DocumentExtractionProvider>,
    ) -> Result<DerivationRunResult> {
        let claims = self
            .claim_derivations(limit, lease_owner, std::time::Duration::from_secs(60))
            .await?;
        let mut result = DerivationRunResult {
            claimed: claims.len(),
            succeeded: 0,
            failed: 0,
        };
        for claim in claims {
            let outcome = async {
                let source = sqlx::query(
                    "SELECT a.media_type,a.content_hash FROM source_regions r JOIN artifacts a ON a.artifact_id=r.artifact_id WHERE r.subject_id=$1 AND r.source_region_id=$2",
                )
                .bind(claim.subject_id.0)
                .bind(claim.source_region_id.0)
                .fetch_optional(self.store.pool())
                .await
                .map_err(db)?
                .ok_or_else(|| Error::NotFound("derivation source artifact not found".into()))?;
                let media_type: String = source.try_get("media_type").map_err(db)?;
                let hash: String = source.try_get("content_hash").map_err(db)?;
                let input = self.objects.get(&hash).await?;
                let output = provider
                    .extract(DocumentExtractionRequest {
                        subject: claim.subject_id,
                        source_region: claim.source_region_id,
                        media_type,
                        input,
                    })
                    .await?;
                if output.text.trim().is_empty() {
                    return Err(Error::Invalid("document provider returned empty text".into()));
                }
                let expected_hash: String = sqlx::query_scalar(
                    "SELECT signature_hash FROM producer_signatures WHERE producer_signature_id=$1",
                )
                .bind(claim.producer_signature_id)
                .fetch_one(self.store.pool())
                .await
                .map_err(db)?;
                if expected_hash != output.producer.signature_hash {
                    return Err(Error::Conflict(
                        "derivation provider signature does not match claimed producer".into(),
                    ));
                }
                let kind = output.representation_kind;
                if kind.as_str() != claim.representation_kind {
                    return Err(Error::Conflict(
                        "derivation provider representation kind does not match claim".into(),
                    ));
                }
                let revision: i32 = sqlx::query_scalar("SELECT COALESCE(max(revision),0)+1 FROM derived_representations WHERE subject_id=$1 AND source_region_id=$2 AND representation_kind=$3")
                    .bind(claim.subject_id.0)
                    .bind(claim.source_region_id.0)
                    .bind(kind.as_str())
                    .fetch_one(self.store.pool())
                    .await
                    .map_err(db)?;
                let representation = DerivedRepresentation {
                    derived_representation_id: DerivedRepresentationId::new(),
                    subject_id: claim.subject_id,
                    source_region_id: claim.source_region_id,
                    representation_kind: kind,
                    producer: output.producer,
                    revision,
                    payload_text: Some(output.text),
                    payload_artifact_id: None,
                    quality: output.coverage,
                    created_at: Utc::now(),
                    supersedes: None,
                };
                let representation = self.persist_derived_representation(representation).await?;
                for region in output.derived_regions {
                    let mut region = region;
                    region.derived_representation_id = representation.derived_representation_id;
                    region.subject_id = claim.subject_id;
                    self.persist_derived_region(region).await?;
                }
                Ok::<(), Error>(())
            }
            .await;
            match outcome {
                Ok(()) => {
                    sqlx::query("UPDATE derivation_attempts SET state='succeeded',finished_at=$2 WHERE attempt_id=$1")
                        .bind(claim.attempt_id)
                        .bind(Utc::now())
                        .execute(self.store.pool())
                        .await
                        .map_err(db)?;
                    result.succeeded += 1;
                }
                Err(error) => {
                    sqlx::query("UPDATE derivation_attempts SET state='transient_failed',finished_at=$2,problem_code='provider_error',problem_detail=$3 WHERE attempt_id=$1")
                        .bind(claim.attempt_id)
                        .bind(Utc::now())
                        .bind(serde_json::json!({"detail": error.to_string()}))
                        .execute(self.store.pool())
                        .await
                        .map_err(db)?;
                    sqlx::query("UPDATE derivations SET state='failed',updated_at=$2 WHERE derivation_id=$1")
                        .bind(claim.derivation_id.0)
                        .bind(Utc::now())
                        .execute(self.store.pool())
                        .await
                        .map_err(db)?;
                    sqlx::query("UPDATE coverage_needs SET state='failed',updated_at=$3 WHERE subject_id=$1 AND source_region_id=$2 AND representation_kind=$4")
                        .bind(claim.subject_id.0)
                        .bind(claim.source_region_id.0)
                        .bind(Utc::now())
                        .bind(&claim.representation_kind)
                        .execute(self.store.pool())
                        .await
                        .map_err(db)?;
                    result.failed += 1;
                }
            }
        }
        Ok(result)
    }

    pub async fn retry_derivation(&self, derivation: DerivationId) -> Result<()> {
        let changed = sqlx::query("UPDATE derivations SET state='pending',updated_at=$2 WHERE derivation_id=$1 AND state IN ('failed','unavailable')")
            .bind(derivation.0)
            .bind(Utc::now())
            .execute(self.store.pool())
            .await
            .map_err(db)?;
        if changed.rows_affected() == 0 {
            return Err(Error::NotFound("retryable derivation not found".into()));
        }
        Ok(())
    }
}
