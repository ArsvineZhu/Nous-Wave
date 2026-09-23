use super::*;
use nous_authority_store::database_error as db;

impl MemoryService {
    pub async fn temporal_evidence(&self,revision:MemoryRevisionId)->Result<TemporalEvidence>{
        let row=sqlx::query("SELECT occurred_min,occurred_max,observed_min,observed_max FROM memory_temporal_evidence WHERE memory_revision_id=$1")
            .bind(revision.0).fetch_optional(self.store.pool()).await.map_err(db)?;
        row.map(|r|Ok(TemporalEvidence{occurred_min:r.try_get("occurred_min").map_err(db)?,occurred_max:r.try_get("occurred_max").map_err(db)?,observed_min:r.try_get("observed_min").map_err(db)?,observed_max:r.try_get("observed_max").map_err(db)?})).transpose().map(|v|v.unwrap_or_default())
    }
    pub(crate) async fn evidence_time_matches(&self,revision:MemoryRevisionId,occurred:Option<TimeInterval>,observed:Option<TimeInterval>)->Result<bool>{
        if occurred.is_none()&&observed.is_none(){return Ok(true);}
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memory_evidence_occurrences WHERE memory_revision_id=$1 AND (NOT $2 OR (occurred_at IS NOT NULL AND ($3::timestamptz IS NULL OR occurred_at>=$3) AND ($4::timestamptz IS NULL OR occurred_at<=$4))) AND (NOT $5 OR (observed_at IS NOT NULL AND ($6::timestamptz IS NULL OR observed_at>=$6) AND ($7::timestamptz IS NULL OR observed_at<=$7))))")
            .bind(revision.0).bind(occurred.is_some()).bind(occurred.and_then(|t|t.start)).bind(occurred.and_then(|t|t.end))
            .bind(observed.is_some()).bind(observed.and_then(|t|t.start)).bind(observed.and_then(|t|t.end))
            .fetch_one(self.store.pool()).await.map_err(db)
    }
}
