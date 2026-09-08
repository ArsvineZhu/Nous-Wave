mod types;
pub use types::*;
use crate::*;
use sqlx::Row;
use nous_authority_store::database_error as db;

impl CognitiveRuntimeService {
    pub async fn materialize_resource(&self, subject: SubjectId, resource: &ResourceRef, handle: &str) -> Result<ResourceMaterial> {
        let descriptor = self.list_resources(subject).await?.into_iter().find(|view| view.descriptor.resource_ref==*resource).ok_or_else(||Error::NotFound("Resource not found".into()))?.descriptor;
        let resolver=self.resource_resolvers.read().map_err(|_|Error::Infrastructure("resource resolver lock poisoned".into()))?.get(&descriptor.resolver_key).cloned().ok_or_else(||Error::Unavailable("Resource resolver is unavailable".into()))?;
        resolver.materialize(handle).await
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

    pub async fn resource_actions_for_query(
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
    }}

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
        resolver_key: row.try_get("resolver_key").map_err(db)?,
        readiness: row.try_get("readiness").map_err(db)?,
        updated_at: row.try_get("updated_at").map_err(db)?,
    })
}
