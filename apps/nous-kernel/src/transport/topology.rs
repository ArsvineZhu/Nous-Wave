use super::*;
use nous_authority_store::database_error as db;
use nous_core::*;
use nous_memory_service::{
    AnchorSupportInput, CreateAnchorRequest, CreateAssociationRequest, CreateTagRequest,
    RebindEntityRequest,
};
use sqlx::Row;
use std::collections::HashSet;
use uuid::Uuid;

impl KernelService {
    pub(super) async fn create_tag(&self, input: p::CreateTagRequest) -> Result<p::Tag> {
        let t = required(input.tag, "tag")?;
        let subject = SubjectId(id(&input.subject_id)?);
        let tag = self
            .0
            .require_memory()?
            .create_tag(
                subject,
                CreateTagRequest {
                    label: t.label.clone(),
                    description: t.description,
                    kind_hint: t.kind_hint,
                    origin: if t.origin.is_empty() {
                        "explicit".into()
                    } else {
                        t.origin
                    },
                },
            )
            .await?;
        self.0
            .store
            .bind_identity(subject, CognitiveRef::Tag(tag.tag_id), t.label, vec![])
            .await?;
        self.get_tag(p::ObjectRequest {
            subject_id: input.subject_id,
            id: tag.tag_id.0.to_string(),
        })
        .await
    }
    pub(super) async fn get_tag(&self, input: p::ObjectRequest) -> Result<p::Tag> {
        let r=sqlx::query("SELECT t.tag_id,r.label,r.description,r.kind_hint,r.origin FROM tags t JOIN tag_revisions r ON t.current_revision_id=r.tag_revision_id WHERE t.subject_id=$1 AND t.tag_id=$2")
            .bind(id(&input.subject_id)?).bind(id(&input.id)?).fetch_one(self.0.store.pool()).await.map_err(db)?;
        Ok(p::Tag {
            tag_id: r.try_get::<Uuid, _>("tag_id").map_err(db)?.to_string(),
            label: r.try_get("label").map_err(db)?,
            description: r.try_get("description").map_err(db)?,
            kind_hint: r.try_get("kind_hint").map_err(db)?,
            origin: r.try_get("origin").map_err(db)?,
        })
    }
    pub(super) async fn list_tags(&self, input: p::ListRequest) -> Result<p::ListTagsResponse> {
        let (ids, next_page_token) = self.topology_ids(&input, "tags", "tag_id").await?;
        let mut items = vec![];
        for id in ids {
            items.push(
                self.get_tag(p::ObjectRequest {
                    subject_id: input.subject_id.clone(),
                    id: id.to_string(),
                })
                .await?,
            );
        }
        Ok(p::ListTagsResponse {
            items,
            next_page_token,
        })
    }
    pub(super) async fn create_anchor(&self, input: p::CreateAnchorRequest) -> Result<p::Anchor> {
        let a = required(input.anchor, "anchor")?;
        let subject = SubjectId(id(&input.subject_id)?);
        let result = self
            .0
            .require_memory()?
            .create_anchor(
                subject,
                CreateAnchorRequest {
                    label: a.label.clone(),
                    description: a.description,
                    origin: if a.origin.is_empty() {
                        "explicit".into()
                    } else {
                        a.origin
                    },
                    confirmed: a.confirmed,
                    supports: a
                        .supports
                        .into_iter()
                        .map(|s| {
                            Ok(AnchorSupportInput {
                                reference: from_ref(required(s.reference, "support reference")?)?,
                                role: s.role,
                            })
                        })
                        .collect::<Result<_>>()?,
                },
            )
            .await?;
        self.0
            .store
            .bind_identity(
                subject,
                CognitiveRef::Anchor(result.anchor_id),
                a.label.unwrap_or_default(),
                vec![],
            )
            .await?;
        self.get_anchor(p::ObjectRequest {
            subject_id: input.subject_id,
            id: result.anchor_id.0.to_string(),
        })
        .await
    }
    pub(super) async fn get_anchor(&self, input: p::ObjectRequest) -> Result<p::Anchor> {
        let r=sqlx::query("SELECT a.anchor_id,a.current_revision_id,r.label,r.description,r.origin,r.confirmed FROM anchors a JOIN anchor_revisions r ON a.current_revision_id=r.anchor_revision_id WHERE a.subject_id=$1 AND a.anchor_id=$2")
            .bind(id(&input.subject_id)?).bind(id(&input.id)?).fetch_one(self.0.store.pool()).await.map_err(db)?;
        let rows=sqlx::query("SELECT support_ref_kind,support_ref,support_role FROM anchor_support WHERE anchor_revision_id=$1 ORDER BY support_ref_kind,support_ref LIMIT 257")
            .bind(r.try_get::<Uuid,_>("current_revision_id").map_err(db)?).fetch_all(self.0.store.pool()).await.map_err(db)?;
        if rows.len() > 256 {
            return Err(Error::Invalid("Anchor support bound exceeded".into()));
        }
        let supports = rows
            .into_iter()
            .map(|s| {
                Ok(p::AnchorSupport {
                    reference: Some(p::CognitiveRef {
                        kind: s.try_get("support_ref_kind").map_err(db)?,
                        value: s.try_get("support_ref").map_err(db)?,
                    }),
                    role: s.try_get("support_role").map_err(db)?,
                })
            })
            .collect::<Result<_>>()?;
        Ok(p::Anchor {
            anchor_id: input.id,
            label: r.try_get("label").map_err(db)?,
            description: r.try_get("description").map_err(db)?,
            origin: r.try_get("origin").map_err(db)?,
            confirmed: r.try_get("confirmed").map_err(db)?,
            supports,
        })
    }
    pub(super) async fn list_anchors(
        &self,
        input: p::ListRequest,
    ) -> Result<p::ListAnchorsResponse> {
        let (ids, next_page_token) = self.topology_ids(&input, "anchors", "anchor_id").await?;
        let mut items = vec![];
        for id in ids {
            items.push(
                self.get_anchor(p::ObjectRequest {
                    subject_id: input.subject_id.clone(),
                    id: id.to_string(),
                })
                .await?,
            );
        }
        Ok(p::ListAnchorsResponse {
            items,
            next_page_token,
        })
    }
    async fn topology_ids(
        &self,
        input: &p::ListRequest,
        table: &str,
        column: &str,
    ) -> Result<(Vec<Uuid>, String)> {
        // Table/column are private literals selected by the two callers above.
        let subject = id(&input.subject_id)?;
        self.0.store.require_subject(SubjectId(subject)).await?;
        let scope = format!("{table}:{subject}:{}", input.status);
        let (limit, last) = page(input.page.clone(), &scope)?;
        let mut ids=sqlx::query_scalar::<_,Uuid>(match (table,column) {
            ("tags","tag_id")=>"SELECT tag_id FROM tags WHERE subject_id=$1 AND ($2::uuid IS NULL OR tag_id>$2) AND ($3='' OR status=$3) ORDER BY tag_id LIMIT $4",
            ("anchors","anchor_id")=>"SELECT anchor_id FROM anchors WHERE subject_id=$1 AND ($2::uuid IS NULL OR anchor_id>$2) AND ($3='' OR status=$3) ORDER BY anchor_id LIMIT $4",
            _=>return Err(Error::Invalid("invalid topology collection".into())),
        })
            .bind(subject).bind(last).bind(&input.status).bind(limit+1).fetch_all(self.0.store.pool()).await.map_err(db)?;
        let more = ids.len() > limit as usize;
        ids.truncate(limit as usize);
        let token = if more {
            next_token(&scope, *ids.last().expect("page"))
        } else {
            String::new()
        };
        Ok((ids, token))
    }
    pub(super) async fn create_association(
        &self,
        input: p::CreateAssociationRequest,
    ) -> Result<p::Association> {
        let mut a = required(input.association, "association")?;
        let result = self
            .0
            .require_memory()?
            .create_association(
                SubjectId(id(&input.subject_id)?),
                CreateAssociationRequest {
                    from: from_ref(required(a.from.clone(), "from")?)?,
                    to: from_ref(required(a.to.clone(), "to")?)?,
                    association_kind: a.kind.clone(),
                    polarity: enum_value(&a.polarity)?,
                    support_class: enum_value(&a.support_class)?,
                    support_value: a.support_value,
                    occurrence_id: a
                        .occurrence_id
                        .as_deref()
                        .map(id)
                        .transpose()?
                        .map(OccurrenceId),
                    memory_revision_id: a
                        .memory_revision_id
                        .as_deref()
                        .map(id)
                        .transpose()?
                        .map(MemoryRevisionId),
                    bridge_hint: a.bridge_hint,
                },
            )
            .await?;
        a.association_id = result.association_evidence_id.to_string();
        Ok(a)
    }
    pub(super) async fn rebind_entity(&self, input: p::RebindEntityRequest) -> Result<()> {
        self.0
            .require_memory()?
            .rebind_entity(
                SubjectId(id(&input.subject_id)?),
                RebindEntityRequest {
                    mention_id: id(&input.mention_id)?,
                    entity_ref: input.entity_ref.map(EntityRef::new).transpose()?,
                    binding_state: input.binding_state,
                    host_resolution_ref: input.host_resolution_ref,
                    reason: input.reason,
                },
            )
            .await
    }
    pub(super) async fn get_neighborhood(
        &self,
        input: p::NeighborhoodRequest,
    ) -> Result<p::NeighborhoodResponse> {
        let subject = SubjectId(id(&input.subject_id)?);
        let root = from_ref(required(input.root, "root")?)?;
        self.0.store.validate_reference(subject, &root).await?;
        if input.max_nodes == 0
            || input.max_nodes > 256
            || input.max_depth == 0
            || input.max_depth > 8
        {
            return Err(Error::Invalid(
                "Neighborhood requires max_nodes<=256, max_depth<=8".into(),
            ));
        }
        let mut nodes = vec![root.clone()];
        let mut seen = HashSet::from([root]);
        let mut edges = vec![];
        let mut edge_ids = HashSet::new();
        let mut frontier = nodes.clone();
        let mut truncated = false;
        'depths: for _ in 0..input.max_depth {
            let mut next = vec![];
            for reference in frontier {
                let (kind, value) = reference_parts(&reference);
                let rows=sqlx::query("SELECT * FROM association_evidence WHERE subject_id=$1 AND ((from_ref_kind=$2 AND from_ref=$3) OR (to_ref_kind=$2 AND to_ref=$3)) ORDER BY association_evidence_id LIMIT 257")
                    .bind(subject.0).bind(kind).bind(value).fetch_all(self.0.store.pool()).await.map_err(db)?;
                if rows.len() > 256 {
                    truncated = true;
                }
                for row in rows.into_iter().take(256) {
                    if edges.len() >= 1024 {
                        truncated = true;
                        break 'depths;
                    }
                    let from = parse_reference(
                        &row.try_get::<String, _>("from_ref_kind").map_err(db)?,
                        &row.try_get::<String, _>("from_ref").map_err(db)?,
                    )?;
                    let to = parse_reference(
                        &row.try_get::<String, _>("to_ref_kind").map_err(db)?,
                        &row.try_get::<String, _>("to_ref").map_err(db)?,
                    )?;
                    let other = if from == reference {
                        to.clone()
                    } else {
                        from.clone()
                    };
                    if !seen.contains(&other) {
                        if nodes.len() >= input.max_nodes as usize {
                            truncated = true;
                            continue;
                        }
                        seen.insert(other.clone());
                        nodes.push(other.clone());
                        next.push(other);
                    }
                    let key: Uuid = row.try_get("association_evidence_id").map_err(db)?;
                    if edge_ids.insert(key) {
                        edges.push(p::Association {
                            association_id: key.to_string(),
                            from: Some(to_ref(from)),
                            to: Some(to_ref(to)),
                            kind: row.try_get("association_kind").map_err(db)?,
                            polarity: row.try_get("polarity").map_err(db)?,
                            support_class: row.try_get("support_class").map_err(db)?,
                            support_value: row.try_get("support_value").map_err(db)?,
                            occurrence_id: row
                                .try_get::<Option<Uuid>, _>("occurrence_id")
                                .map_err(db)?
                                .map(|i| i.to_string()),
                            memory_revision_id: row
                                .try_get::<Option<Uuid>, _>("memory_revision_id")
                                .map_err(db)?
                                .map(|i| i.to_string()),
                            bridge_hint: row.try_get("bridge_hint").map_err(db)?,
                        });
                    }
                }
            }
            frontier = next;
            if frontier.is_empty() {
                break;
            }
        }
        Ok(p::NeighborhoodResponse {
            nodes: nodes.into_iter().map(to_ref).collect(),
            associations: edges,
            truncated,
        })
    }
}
