use super::support::*;
use super::*;
use nous_authority_store::ProjectionInvalidation;

impl MemoryService {
    pub(crate) async fn fence_memory_head(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        subject: SubjectId,
        memory: MemoryId,
        expected: i64,
    ) -> Result<()> {
        let revision: i64 = sqlx::query_scalar("SELECT head_revision FROM memory_objects WHERE subject_id=$1 AND memory_id=$2 FOR UPDATE")
            .bind(subject.0).bind(memory.0).fetch_one(&mut **tx).await.map_err(db)?;
        if revision != expected {
            return Err(Error::Conflict("stale Memory head revision".into()));
        }
        Ok(())
    }
    pub async fn suppress(
        &self,
        subject: SubjectId,
        memory: MemoryId,
        expected_head_revision: i64,
    ) -> Result<MemoryView> {
        let mut tx = self.store.begin().await?;
        self.fence_memory_head(&mut tx, subject, memory, expected_head_revision)
            .await?;
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

    pub async fn restore(
        &self,
        subject: SubjectId,
        memory: MemoryId,
        expected_head_revision: i64,
    ) -> Result<MemoryView> {
        let mut tx = self.store.begin().await?;
        self.fence_memory_head(&mut tx, subject, memory, expected_head_revision)
            .await?;
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
    #[expect(
        clippy::too_many_lines,
        reason = "purge coordinates targeted reachability checks, Authority deletion and CAS cleanup"
    )]
    pub async fn purge_memory(
        &self,
        subject: SubjectId,
        memory: MemoryId,
        expected_head_revision: i64,
    ) -> Result<()> {
        let guard = self.objects.reference_guard(true).await?;
        let mut tx = self.store.begin().await?;
        self.fence_memory_head(&mut tx, subject, memory, expected_head_revision)
            .await?;
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
            sqlx::query_scalar::<_, Uuid>("SELECT d.derived_representation_id FROM derived_representations d WHERE d.derived_representation_id=ANY($1) AND NOT EXISTS (SELECT 1 FROM memory_revision_evidence e WHERE e.memory_revision_id <> ALL($2) AND (e.derived_representation_id=d.derived_representation_id OR e.derived_region_id IN (SELECT derived_region_id FROM derived_regions WHERE derived_representation_id=d.derived_representation_id))) AND NOT EXISTS (SELECT 1 FROM anchor_support s WHERE (s.support_ref_kind='derived_representation' AND s.support_ref=d.derived_representation_id::text) OR (s.support_ref_kind='derived_region' AND s.support_ref IN (SELECT derived_region_id::text FROM derived_regions WHERE derived_representation_id=d.derived_representation_id))) AND NOT EXISTS (SELECT 1 FROM association_evidence a WHERE a.subject_id=d.subject_id AND ((a.from_ref_kind='derived_representation' AND a.from_ref=d.derived_representation_id::text) OR (a.to_ref_kind='derived_representation' AND a.to_ref=d.derived_representation_id::text) OR (a.from_ref_kind='derived_region' AND a.from_ref IN (SELECT derived_region_id::text FROM derived_regions WHERE derived_representation_id=d.derived_representation_id)) OR (a.to_ref_kind='derived_region' AND a.to_ref IN (SELECT derived_region_id::text FROM derived_regions WHERE derived_representation_id=d.derived_representation_id)))) AND NOT EXISTS (SELECT 1 FROM resident_refs r JOIN cognitive_sessions s ON s.session_id=r.session_id WHERE s.subject_id=d.subject_id AND ((r.ref_kind='derived_representation' AND r.ref_value=d.derived_representation_id::text) OR (r.ref_kind='derived_region' AND r.ref_value IN (SELECT derived_region_id::text FROM derived_regions WHERE derived_representation_id=d.derived_representation_id)))) AND NOT EXISTS (SELECT 1 FROM coverage_needs c WHERE c.current_representation_id=d.derived_representation_id) AND NOT EXISTS (SELECT 1 FROM derivations de WHERE de.successful_representation_id=d.derived_representation_id) AND NOT EXISTS (SELECT 1 FROM derived_representations d2 WHERE d2.supersedes=d.derived_representation_id) AND NOT EXISTS (SELECT 1 FROM derived_regions dr2 WHERE dr2.parent_derived_region_id IN (SELECT derived_region_id FROM derived_regions WHERE derived_representation_id=d.derived_representation_id))")
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
            sqlx::query("DELETE FROM derived_representations d WHERE d.derived_representation_id=ANY($1) AND NOT EXISTS (SELECT 1 FROM memory_revision_evidence e WHERE e.memory_revision_id <> ALL($2) AND (e.derived_representation_id=d.derived_representation_id OR e.derived_region_id IN (SELECT derived_region_id FROM derived_regions WHERE derived_representation_id=d.derived_representation_id)))")
                .bind(&removable_derived_ids)
                .bind(&revision_values)
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
                "SELECT content_hash FROM artifacts WHERE subject_id=$1 AND artifact_id=$2 AND NOT EXISTS (SELECT 1 FROM observation_occurrences WHERE artifact_id=$2) AND NOT EXISTS (SELECT 1 FROM source_regions WHERE artifact_id=$2) AND NOT EXISTS (SELECT 1 FROM derived_representations WHERE payload_artifact_id=$2) AND NOT EXISTS (SELECT 1 FROM character_seeds WHERE artifact_id=$2)",
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
}
