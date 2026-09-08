use crate::*;
use nous_authority_store::database_error as db;
use sqlx::Row;

impl CognitiveRuntimeService {
    pub async fn open_session(
        &self,
        subject: SubjectId,
        metadata: serde_json::Value,
    ) -> Result<SessionView> {
        self.require_subject(subject).await?;
        let session = SessionId::new();
        let now = Utc::now();
        sqlx::query("INSERT INTO cognitive_sessions(session_id,subject_id,opened_at,last_activity_at,metadata) VALUES($1,$2,$3,$3,$4)")
            .bind(session.0).bind(subject.0).bind(now).bind(metadata)
            .execute(self.store.pool()).await.map_err(db)?;
        self.session(subject, session).await
    }

    pub async fn close_session(
        &self,
        subject: SubjectId,
        session: SessionId,
    ) -> Result<SessionView> {
        self.require_session(subject, session).await?;
        sqlx::query("UPDATE cognitive_sessions SET closed_at=$3,last_activity_at=$3,state_revision=state_revision+1 WHERE subject_id=$1 AND session_id=$2")
            .bind(subject.0).bind(session.0).bind(Utc::now())
            .execute(self.store.pool()).await.map_err(db)?;
        self.session(subject, session).await
    }

    pub async fn session(&self, subject: SubjectId, session: SessionId) -> Result<SessionView> {
        let row = sqlx::query("SELECT session_id,subject_id,opened_at,last_activity_at,last_meaningful_use_at,closed_at,state_revision FROM cognitive_sessions WHERE subject_id=$1 AND session_id=$2")
            .bind(subject.0).bind(session.0).fetch_optional(self.store.pool()).await.map_err(db)?
            .ok_or_else(|| Error::NotFound("session not found".into()))?;
        let refs = sqlx::query("SELECT ref_kind,ref_value,entry_reason,state,entered_at,last_meaningful_use_at,hold_until FROM resident_refs WHERE session_id=$1 AND state <> 'evicted' ORDER BY entered_at")
            .bind(session.0).fetch_all(self.store.pool()).await.map_err(db)?;
        let resident = refs
            .into_iter()
            .filter_map(|row| {
                let reference = parse_reference(
                    &row.try_get::<String, _>("ref_kind").ok()?,
                    &row.try_get::<String, _>("ref_value").ok()?,
                )
                .ok()?;
                Some(ResidentView {
                    reference,
                    entry_reason: row.try_get("entry_reason").ok()?,
                    state: row.try_get("state").ok()?,
                    entered_at: row.try_get("entered_at").ok()?,
                    last_meaningful_use_at: row.try_get("last_meaningful_use_at").ok()?,
                    hold_until: row.try_get("hold_until").ok()?,
                })
            })
            .collect();
        Ok(SessionView {
            session_id: SessionId(row.try_get("session_id").map_err(db)?),
            subject_id: SubjectId(row.try_get("subject_id").map_err(db)?),
            opened_at: row.try_get("opened_at").map_err(db)?,
            last_activity_at: row.try_get("last_activity_at").map_err(db)?,
            last_meaningful_use_at: row.try_get("last_meaningful_use_at").map_err(db)?,
            closed_at: row.try_get("closed_at").map_err(db)?,
            state_revision: row.try_get("state_revision").map_err(db)?,
            resident,
        })
    }
    pub async fn require_session(&self, subject: SubjectId, session: SessionId) -> Result<()> {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM cognitive_sessions WHERE subject_id=$1 AND session_id=$2 AND closed_at IS NULL)")
            .bind(subject.0)
            .bind(session.0)
            .fetch_one(self.store.pool())
            .await
            .map_err(db)?;
        if exists {
            Ok(())
        } else {
            Err(Error::NotFound("open session not found".into()))
        }
    }

    pub async fn admit(
        &self,
        session: SessionId,
        reference: CognitiveRef,
        reason: &str,
        hold_until: Option<DateTime<Utc>>,
    ) -> Result<()> {
        self.admit_with_state(session, reference, reason, hold_until, "resident")
            .await
    }

    pub async fn admit_with_state(
        &self,
        session: SessionId,
        reference: CognitiveRef,
        reason: &str,
        hold_until: Option<DateTime<Utc>>,
        state: &str,
    ) -> Result<()> {
        if !matches!(state, "resident" | "provisional") {
            return Err(Error::Invalid("invalid resident state".into()));
        }
        let (kind, value) = reference_parts(&reference);
        let now = Utc::now();
        sqlx::query("INSERT INTO resident_refs(session_id,ref_kind,ref_value,entered_at,entry_reason,hold_until,state,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,'{}') ON CONFLICT(session_id,ref_kind,ref_value) DO UPDATE SET state=excluded.state,entry_reason=excluded.entry_reason,hold_until=COALESCE(excluded.hold_until,resident_refs.hold_until)")
            .bind(session.0).bind(kind).bind(value).bind(now).bind(reason).bind(hold_until).bind(state)
            .execute(self.store.pool()).await.map_err(db)?;
        sqlx::query("UPDATE cognitive_sessions SET last_activity_at=$2,state_revision=state_revision+1 WHERE session_id=$1")
            .bind(session.0).bind(now).execute(self.store.pool()).await.map_err(db)?;
        Ok(())
    }

    pub async fn evict_if_needed(&self, session: SessionId) -> Result<()> {
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM resident_refs WHERE session_id=$1 AND state IN ('resident','provisional')",
        )
        .bind(session.0)
        .fetch_one(self.store.pool())
        .await
        .map_err(db)?;
        if count as usize <= self.resident_limit {
            return Ok(());
        }
        let excess = count as usize - self.resident_limit;
        sqlx::query("WITH victims AS (SELECT session_id,ref_kind,ref_value FROM resident_refs WHERE session_id=$1 AND state IN ('resident','provisional') AND (hold_until IS NULL OR hold_until < now()) ORDER BY (state='provisional') DESC,(last_meaningful_use_at IS NOT NULL) ASC,last_meaningful_use_at ASC NULLS FIRST,entered_at ASC LIMIT $2) UPDATE resident_refs r SET state='evicted' FROM victims v WHERE r.session_id=v.session_id AND r.ref_kind=v.ref_kind AND r.ref_value=v.ref_value")
            .bind(session.0).bind(excess as i64).execute(self.store.pool()).await.map_err(db)?;
        Ok(())
    }

    pub async fn resident_reference_strings(&self, session: SessionId) -> Result<HashSet<String>> {
        Ok(sqlx::query(
            "SELECT ref_kind,ref_value FROM resident_refs WHERE session_id=$1 AND state IN ('resident','provisional')",
        )
        .bind(session.0)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?
        .into_iter()
        .map(|row| {
            format!(
                "{}:{}",
                row.get::<String, _>("ref_kind"),
                row.get::<String, _>("ref_value")
            )
        })
        .collect())
    }
}
