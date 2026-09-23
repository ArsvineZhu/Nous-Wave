use super::*;
use nous_authority_store::{ProjectionInvalidation,database_error as db};
impl MemoryService {
    pub async fn link_revisions(&self,subject:SubjectId,from:MemoryRevisionId,to:MemoryRevisionId,relation:MemoryRelation)->Result<()>{
        let source=self.revision(subject,from).await?;let target=self.revision(subject,to).await?;
        if source.object.memory_id==target.object.memory_id||relation==MemoryRelation::Supersedes{return Err(Error::Invalid("same-Memory lineage is created only by explicit revision".into()));}
        if (relation==MemoryRelation::Integrates&&source.object.memory_class!=MemoryClass::Integrative)||(relation==MemoryRelation::Proceduralizes&&source.object.memory_class!=MemoryClass::Procedural){return Err(Error::Invalid("revision relation does not match the source Memory class".into()));}
        let mut tx=self.store.begin().await?;
        sqlx::query("INSERT INTO memory_revision_relations(from_revision_id,to_revision_id,relation,created_at) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING")
            .bind(from.0).bind(to.0).bind(relation.as_str()).bind(Utc::now()).execute(&mut *tx).await.map_err(db)?;
        AuthorityStore::invalidate_in(&mut tx,subject,ProjectionInvalidation::topology()).await?;
        tx.commit().await.map_err(db)
    }
    pub(crate) async fn revision_relations(&self,subject:SubjectId,revision:MemoryRevisionId)->Result<(Vec<MemoryRevisionRelation>,bool)>{
        let rows=sqlx::query("SELECT x.* FROM memory_revision_relations x JOIN memory_revisions r ON r.memory_revision_id=x.from_revision_id WHERE r.subject_id=$1 AND (x.from_revision_id=$2 OR x.to_revision_id=$2) ORDER BY x.created_at,x.from_revision_id,x.to_revision_id LIMIT 257")
            .bind(subject.0).bind(revision.0).fetch_all(self.store.pool()).await.map_err(db)?;
        let truncated=rows.len()>256;
        let relations=rows.into_iter().take(256).map(|r|Ok(MemoryRevisionRelation{from_revision_id:MemoryRevisionId(r.try_get("from_revision_id").map_err(db)?),to_revision_id:MemoryRevisionId(r.try_get("to_revision_id").map_err(db)?),relation:serde_json::from_value(serde_json::Value::String(r.try_get("relation").map_err(db)?)).map_err(|e|Error::Infrastructure(e.to_string()))?,created_at:r.try_get("created_at").map_err(db)?})).collect::<Result<_>>()?;
        Ok((relations,truncated))
    }
}
