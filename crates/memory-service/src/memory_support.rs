use super::support::*;
use super::*;
use nous_authority_store::database_error as db;

pub(crate) fn support_root(reference: &CognitiveRef) -> Option<String> {
    let (kind, value) = reference_parts(reference);
    Some(format!("{kind}:{value}"))
}

pub(crate) async fn insert_tag_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subject: SubjectId,
    label: &str,
    description: Option<&str>,
    kind_hint: Option<&str>,
    origin: &str,
) -> Result<TagId> {
    let tag_id = TagId::new();
    let revision_id = Uuid::now_v7();
    let now = Utc::now();
    sqlx::query("INSERT INTO tags(tag_id,subject_id,current_revision_id,created_at,status) VALUES($1,$2,$3,$4,'active')")
        .bind(tag_id.0)
        .bind(subject.0)
        .bind(revision_id)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db)?;
    sqlx::query("INSERT INTO tag_revisions(tag_revision_id,tag_id,revision_no,label,description,kind_hint,origin,created_at) VALUES($1,$2,1,$3,$4,$5,$6,$7)")
        .bind(revision_id)
        .bind(tag_id.0)
        .bind(label)
        .bind(description)
        .bind(kind_hint)
        .bind(origin)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db)?;
    Ok(tag_id)
}

pub(crate) async fn insert_anchor_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subject: SubjectId,
    anchor: &TopologyAnchorProposal,
    origin: &str,
) -> Result<AnchorId> {
    let anchor_id = AnchorId::new();
    let revision_id = Uuid::now_v7();
    let now = Utc::now();
    sqlx::query("INSERT INTO anchors(anchor_id,subject_id,current_revision_id,created_at,status) VALUES($1,$2,$3,$4,'active')")
        .bind(anchor_id.0)
        .bind(subject.0)
        .bind(revision_id)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db)?;
    sqlx::query("INSERT INTO anchor_revisions(anchor_revision_id,anchor_id,revision_no,label,description,origin,created_at,confirmed) VALUES($1,$2,1,$3,$4,$5,$6,$7)")
        .bind(revision_id)
        .bind(anchor_id.0)
        .bind(anchor.label.clone())
        .bind(&anchor.description)
        .bind(origin)
        .bind(now)
        .bind(anchor.confirmed)
        .execute(&mut **tx)
        .await
        .map_err(db)?;
    for support in &anchor.supports {
        let (kind, value) = reference_parts(&support.reference);
        sqlx::query("INSERT INTO anchor_support(anchor_revision_id,support_ref_kind,support_ref,support_role,provenance) VALUES($1,$2,$3,$4,'{}')")
            .bind(revision_id)
            .bind(kind)
            .bind(value)
            .bind(&support.role)
            .execute(&mut **tx)
            .await
            .map_err(db)?;
    }
    Ok(anchor_id)
}

pub(crate) async fn insert_association_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subject: SubjectId,
    association: &TopologyAssociationProposal,
) -> Result<()> {
    let (from_kind, from_ref) = reference_parts(&association.from);
    let (to_kind, to_ref) = reference_parts(&association.to);
    let id = Uuid::now_v7();
    let now = Utc::now();
    sqlx::query("INSERT INTO association_evidence(association_evidence_id,subject_id,from_ref_kind,from_ref,to_ref_kind,to_ref,association_kind,polarity,support_class,support_value,occurrence_id,memory_revision_id,bridge_hint,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)")
        .bind(id)
        .bind(subject.0)
        .bind(&from_kind)
        .bind(&from_ref)
        .bind(&to_kind)
        .bind(&to_ref)
        .bind(&association.association_kind)
        .bind(association.polarity.as_str())
        .bind(association.support_class.as_str())
        .bind(association.support_value)
        .bind(association.occurrence_id.map(|id| id.0))
        .bind(association.memory_revision_id.map(|id| id.0))
        .bind(association.bridge_hint)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db)?;
    Ok(())
}

pub(crate) async fn insert_revision_in_tx(
    tx:&mut sqlx::Transaction<'_,sqlx::Postgres>,subject:SubjectId,current:&MemoryView,proposal:&TopologyRevisionProposal,
)->Result<MemoryRevisionId>{
    let semantic_role=proposal.semantic_role.as_deref().unwrap_or(&current.revision.semantic_role);
    ExplicitMemoryInput{subject,memory_class:current.object.memory_class,semantic_role:semantic_role.into(),representation_text:proposal.representation_text.clone(),title:proposal.title.clone(),evidence:proposal.evidence.clone(),entity_refs:proposal.entity_refs.clone(),tags:vec![],tag_order_provenance:None,valid_from:proposal.valid_from,valid_to:proposal.valid_to,epistemic_class:proposal.epistemic_class}.validate()?;
    let new_id=MemoryRevisionId::new();let now=Utc::now();
    sqlx::query("UPDATE memory_revisions SET revision_lifecycle='superseded' WHERE memory_revision_id=$1 AND revision_lifecycle='current'")
        .bind(current.revision.memory_revision_id.0).execute(&mut **tx).await.map_err(db)?;
    let lifecycle=if proposal.intent==RevisionIntent::Revoke{"revoked"}else{"current"};
    sqlx::query("INSERT INTO memory_revisions(memory_revision_id,memory_id,subject_id,revision_no,parent_revision_id,semantic_role,title,representation_text,attributes,epistemic_class,valid_from,valid_to,created_at,revision_lifecycle,revision_intent) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'{}',$9,$10,$11,$12,$13,$14)")
        .bind(new_id.0).bind(current.object.memory_id.0).bind(subject.0).bind(current.revision.revision_no+1).bind(current.revision.memory_revision_id.0).bind(semantic_role).bind(proposal.title.as_ref().or(current.revision.title.as_ref())).bind(&proposal.representation_text).bind(format!("{:?}",proposal.epistemic_class).to_lowercase()).bind(proposal.valid_from).bind(proposal.valid_to).bind(now).bind(lifecycle).bind(format!("{:?}",proposal.intent).to_lowercase())
        .execute(&mut **tx).await.map_err(db)?;
    let changed=sqlx::query("UPDATE memory_objects SET current_revision_id=$2 WHERE subject_id=$1 AND memory_id=$3 AND head_revision=$4")
        .bind(subject.0).bind(new_id.0).bind(current.object.memory_id.0).bind(proposal.expected_head_revision).execute(&mut **tx).await.map_err(db)?;
    if changed.rows_affected()!=1{return Err(Error::Conflict("stale Memory head revision".into()));}
    sqlx::query("INSERT INTO memory_revision_relations(from_revision_id,to_revision_id,relation,created_at) VALUES($1,$2,'supersedes',$3)")
        .bind(new_id.0).bind(current.revision.memory_revision_id.0).bind(now).execute(&mut **tx).await.map_err(db)?;
    for evidence in &proposal.evidence{insert_evidence(tx,new_id,evidence).await?;}
    insert_memory_entities(tx,new_id,&proposal.entity_refs,"explicit_revision").await?;
    Ok(new_id)
}

pub(crate) async fn insert_memory_entities(tx:&mut sqlx::Transaction<'_,sqlx::Postgres>,revision:MemoryRevisionId,entities:&[EntityRef],operation:&str)->Result<()> {
    for entity in entities {
        EntityRef::new(entity.as_str())?;
        sqlx::query("INSERT INTO memory_revision_entities(memory_revision_id,entity_ref,role,provenance) VALUES($1,$2,'about',$3) ON CONFLICT DO NOTHING")
            .bind(revision.0).bind(entity.as_str()).bind(serde_json::json!({"operation":operation})).execute(&mut **tx).await.map_err(db)?;
    }
    Ok(())
}
