mod types;
use crate::*;
use nous_authority_store::{ProjectionInvalidation, database_error as db};
use sqlx::Row;
pub use types::*;

impl CognitiveRuntimeService {
    pub async fn upsert_resource(
        &self,
        subject: SubjectId,
        input: ResourceUpsert,
    ) -> Result<ResourceView> {
        self.require_subject(subject).await?;
        ResourceRef::new(input.resource_ref.as_str())?;
        if input.authority_class.trim().is_empty() {
            return Err(Error::Invalid(
                "resource authority_class is required".into(),
            ));
        }
        if !matches!(
            input.readiness.as_str(),
            "ready" | "degraded" | "unavailable"
        ) {
            return Err(Error::Invalid("invalid resource readiness".into()));
        }
        let mut tx = self.store.begin().await?;
        sqlx::query("INSERT INTO resources(subject_id,resource_ref,display_label,authority_class,coverage,query_dimensions,modalities,freshness_policy,access_cost_class,readiness,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) ON CONFLICT(subject_id,resource_ref) DO UPDATE SET display_label=excluded.display_label,authority_class=excluded.authority_class,coverage=excluded.coverage,query_dimensions=excluded.query_dimensions,modalities=excluded.modalities,freshness_policy=excluded.freshness_policy,access_cost_class=excluded.access_cost_class,readiness=excluded.readiness,updated_at=excluded.updated_at")
            .bind(subject.0).bind(input.resource_ref.as_str()).bind(input.display_label).bind(input.authority_class).bind(input.coverage).bind(input.query_dimensions).bind(input.modalities).bind(input.freshness_policy).bind(input.access_cost_class).bind(input.readiness).bind(Utc::now()).execute(&mut *tx).await.map_err(db)?;
        let row=sqlx::query("SELECT resource_ref,display_label,authority_class,coverage,query_dimensions,modalities,freshness_policy,access_cost_class,readiness,updated_at FROM resources WHERE subject_id=$1 AND resource_ref=$2").bind(subject.0).bind(input.resource_ref.as_str()).fetch_one(&mut *tx).await.map_err(db)?;
        nous_authority_store::AuthorityStore::invalidate_in(
            &mut tx,
            subject,
            ProjectionInvalidation {
                exact: true,
                topology: true,
                ..ProjectionInvalidation::default()
            },
        )
        .await?;
        tx.commit().await.map_err(db)?;
        Ok(ResourceView {
            descriptor: decode_resource(subject, row)?,
        })
    }

    pub async fn delete_resource(&self, subject: SubjectId, resource: ResourceRef) -> Result<()> {
        let mut tx = self.store.begin().await?;
        sqlx::query("DELETE FROM resources WHERE subject_id=$1 AND resource_ref=$2")
            .bind(subject.0)
            .bind(resource.as_str())
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("DELETE FROM resident_refs r USING cognitive_sessions s WHERE r.session_id=s.session_id AND s.subject_id=$1 AND r.ref_kind='resource' AND r.ref_value=$2")
            .bind(subject.0)
            .bind(resource.as_str())
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        nous_authority_store::AuthorityStore::invalidate_in(
            &mut tx,
            subject,
            ProjectionInvalidation {
                exact: true,
                topology: true,
                ..ProjectionInvalidation::default()
            },
        )
        .await?;
        tx.commit().await.map_err(db)?;
        Ok(())
    }

    pub async fn list_resources(&self, subject: SubjectId) -> Result<Vec<ResourceView>> {
        let rows=sqlx::query("SELECT resource_ref,display_label,authority_class,coverage,query_dimensions,modalities,freshness_policy,access_cost_class,readiness,updated_at FROM resources WHERE subject_id=$1 ORDER BY resource_ref").bind(subject.0).fetch_all(self.store.pool()).await.map_err(db)?;
        rows.into_iter()
            .map(|row| {
                Ok(ResourceView {
                    descriptor: decode_resource(subject, row)?,
                })
            })
            .collect()
    }

    pub async fn resource_actions_for_query(
        &self,
        query: &CognitiveQuery,
        plan: &QueryPlan,
    ) -> Result<(Vec<ResourceActionSuggestion>, Vec<Degradation>)> {
        if query.resources.current_authority == CurrentAuthorityNeed::None
            && !query.resources.synopsis_only
            && !plan.prefer_resource_synopsis
            && !query.cues.iter().any(|cue| matches!(cue, Cue::Resource(_)))
        {
            return Ok((Vec::new(), Vec::new()));
        }
        let requested = query
            .cues
            .iter()
            .filter_map(|cue| match cue {
                Cue::Resource(r) => Some(r.resource.as_str()),
                _ => None,
            })
            .collect::<HashSet<_>>();
        let resources = self.list_resources(query.subject).await?;
        let actions = resources
            .into_iter()
            .filter(|r| {
                requested.is_empty() || requested.contains(r.descriptor.resource_ref.as_str())
            })
            .take(plan.resource_limit)
            .map(|r| ResourceActionSuggestion {
                resource: r.descriptor.resource_ref,
                action: if query.resources.synopsis_only || plan.prefer_resource_synopsis {
                    "inspect_synopsis"
                } else {
                    "query_current_authority"
                }
                .into(),
                reason: format!(
                    "Consumer must access external authority; descriptor readiness={}",
                    r.descriptor.readiness
                ),
                current_authority: false,
                records: vec![],
                evidence: vec![],
            })
            .collect();
        let degradation = vec![Degradation {
            code: if query.resources.current_authority == CurrentAuthorityNeed::Required {
                "required_external_authority_unresolved"
            } else {
                "external_resource_action_required"
            }
            .into(),
            detail: Some(
                "Kernel returned resource awareness; external access has not occurred".into(),
            ),
        }];
        Ok((actions, degradation))
    }
}

pub(crate) fn decode_resource(
    subject: SubjectId,
    row: sqlx::postgres::PgRow,
) -> Result<ResourceDescriptor> {
    Ok(ResourceDescriptor {
        subject_id: subject,
        resource_ref: ResourceRef::new(row.try_get::<String, _>("resource_ref").map_err(db)?)?,
        display_label: row.try_get("display_label").map_err(db)?,
        authority_class: row.try_get("authority_class").map_err(db)?,
        coverage: row.try_get("coverage").map_err(db)?,
        query_dimensions: row.try_get("query_dimensions").map_err(db)?,
        modalities: row.try_get("modalities").map_err(db)?,
        freshness_policy: row.try_get("freshness_policy").map_err(db)?,
        access_cost_class: row.try_get("access_cost_class").map_err(db)?,
        readiness: row.try_get("readiness").map_err(db)?,
        updated_at: row.try_get("updated_at").map_err(db)?,
    })
}
