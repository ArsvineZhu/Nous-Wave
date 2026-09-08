use crate::*;
use nous_authority_store::database_error as db;

impl CognitiveRuntimeService {
    pub async fn use_feedback(&self, input: UseFeedback) -> Result<()> {
        self.require_subject(input.subject).await?;
        if let Some(session) = input.session_id {
            self.require_session(input.subject, session).await?;
        }
        for event in input.events {
            if !self
                .reference_in_subject(input.subject, &event.reference)
                .await?
            {
                return Err(Error::Invalid(
                    "use feedback reference is outside Subject".into(),
                ));
            }
            let (kind, value) = reference_parts(&event.reference);
            let now = Utc::now();
            sqlx::query("INSERT INTO cognitive_use_events(use_event_id,subject_id,session_id,ref_kind,ref_value,use_kind,consumer_ref,occurred_at,context) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)")
                .bind(Uuid::now_v7()).bind(input.subject.0).bind(input.session_id.map(|id|id.0)).bind(&kind).bind(&value).bind(event.use_kind.as_str()).bind(&input.consumer).bind(now).bind(event.context).execute(self.store.pool()).await.map_err(db)?;
            if let Some(session) = input.session_id.filter(|_| event.use_kind.meaningful()) {
                let updated = sqlx::query("UPDATE resident_refs SET last_meaningful_use_at=$4,state='resident' WHERE session_id=$1 AND ref_kind=$2 AND ref_value=$3")
                    .bind(session.0).bind(&kind).bind(&value).bind(now).execute(self.store.pool()).await.map_err(db)?;
                if updated.rows_affected() == 0 {
                    self.admit_with_state(
                        session,
                        event.reference.clone(),
                        "recalled",
                        None,
                        "provisional",
                    )
                    .await?;
                    sqlx::query("UPDATE resident_refs SET last_meaningful_use_at=$4 WHERE session_id=$1 AND ref_kind=$2 AND ref_value=$3")
                        .bind(session.0)
                        .bind(&kind)
                        .bind(&value)
                        .bind(now)
                        .execute(self.store.pool())
                        .await
                        .map_err(db)?;
                }
                sqlx::query("UPDATE cognitive_sessions SET last_activity_at=$2,last_meaningful_use_at=$2,state_revision=state_revision+1 WHERE session_id=$1").bind(session.0).bind(now).execute(self.store.pool()).await.map_err(db)?;
            }
        }
        Ok(())
    }
}
