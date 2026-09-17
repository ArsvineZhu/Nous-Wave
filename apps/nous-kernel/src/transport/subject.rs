use super::*;
use nous_authority_store::database_error as db;
use nous_core::{Result, SessionId, SubjectId};
use nous_subject_core::{CharacterSeedInput, CharacterSeedView, CreateSubject, SubjectView};
use uuid::Uuid;

fn seed(input: p::CharacterSeed) -> CharacterSeedInput {
    CharacterSeedInput {
        text: input.text,
        media_type: input.media_type,
        provenance: serde_json::json!({"source_ref":input.source_ref,"metadata":object(input.metadata)}),
    }
}
fn seed_view(input: CharacterSeedView) -> p::SeedRevision {
    p::SeedRevision {
        revision: input.revision_no,
        created_at: Some(timestamp(input.created_at)),
        seed: Some(p::CharacterSeed {
            text: input.text,
            media_type: input.media_type,
            source_ref: input
                .provenance
                .get("source_ref")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .into(),
            metadata: to_object(input.provenance),
        }),
    }
}
fn subject(input: SubjectView) -> p::Subject {
    p::Subject {
        subject_id: input.subject_id.0.to_string(),
        created_at: Some(timestamp(input.created_at)),
        state_revision: input.state_revision,
        status: input.status,
        config: to_object(input.config),
    }
}
impl KernelService {
    pub(super) async fn create_subject(
        &self,
        input: p::CreateSubjectRequest,
    ) -> Result<p::Subject> {
        Ok(subject(
            self.0
                .subjects
                .create_subject(CreateSubject {
                    subject_id: input
                        .subject_id
                        .as_deref()
                        .map(id)
                        .transpose()?
                        .map(SubjectId),
                    character_seed: seed(required(input.character_seed, "character_seed")?),
                    config: object(input.config),
                })
                .await?,
        ))
    }
    pub(super) async fn get_subject(&self, input: p::SubjectRequest) -> Result<p::Subject> {
        Ok(subject(
            self.0
                .subjects
                .subject(SubjectId(id(&input.subject_id)?))
                .await?,
        ))
    }
    pub(super) async fn list_subjects(
        &self,
        input: p::ListRequest,
    ) -> Result<p::ListSubjectsResponse> {
        let scope = format!("subjects:{}", input.status);
        let (limit, last) = page(input.page, &scope)?;
        let mut ids = sqlx::query_scalar::<_,Uuid>("SELECT subject_id FROM subjects WHERE ($1::uuid IS NULL OR subject_id>$1) AND ($2='' OR status=$2) ORDER BY subject_id LIMIT $3")
            .bind(last).bind(&input.status).bind(limit+1).fetch_all(self.0.store.pool()).await.map_err(db)?;
        let more = ids.len() > limit as usize;
        ids.truncate(limit as usize);
        let next_page_token = if more {
            next_token(&scope, *ids.last().expect("nonempty page"))
        } else {
            String::new()
        };
        let mut items = Vec::new();
        for id in ids {
            items.push(subject(self.0.subjects.subject(SubjectId(id)).await?));
        }
        Ok(p::ListSubjectsResponse {
            items,
            next_page_token,
        })
    }
    pub(super) async fn get_character_seed(
        &self,
        input: p::SubjectRequest,
    ) -> Result<p::SeedRevision> {
        Ok(seed_view(
            self.0
                .subjects
                .character_seed(SubjectId(id(&input.subject_id)?), None)
                .await?,
        ))
    }
    pub(super) async fn revise_character_seed(
        &self,
        input: p::ReviseCharacterSeedRequest,
    ) -> Result<p::SeedRevision> {
        Ok(seed_view(
            self.0
                .subjects
                .revise_character_seed(
                    SubjectId(id(&input.subject_id)?),
                    seed(required(input.seed, "seed")?),
                )
                .await?,
        ))
    }
    pub(super) async fn session_view(
        &self,
        subject: SubjectId,
        session: SessionId,
    ) -> Result<p::Session> {
        let view = self.0.cognition.session(subject, session).await?;
        let active_focus_id: Option<String> = sqlx::query_scalar(
            "SELECT active_focus_key FROM cognitive_sessions WHERE subject_id=$1 AND session_id=$2",
        )
        .bind(subject.0)
        .bind(session.0)
        .fetch_one(self.0.store.pool())
        .await
        .map_err(db)?;
        Ok(p::Session {
            session_id: session.0.to_string(),
            subject_id: subject.0.to_string(),
            runtime_revision: view.state_revision,
            closed: view.closed_at.is_some(),
            resident_refs: view
                .resident
                .into_iter()
                .map(|r| to_ref(r.reference))
                .collect(),
            active_focus_id,
        })
    }
    pub(super) async fn open_session(&self, input: p::SubjectRequest) -> Result<p::Session> {
        let subject = SubjectId(id(&input.subject_id)?);
        let view = self
            .0
            .cognition
            .open_session(subject, serde_json::json!({}))
            .await?;
        self.session_view(subject, view.session_id).await
    }
    pub(super) async fn get_session(&self, input: p::ObjectRequest) -> Result<p::Session> {
        self.session_view(SubjectId(id(&input.subject_id)?), SessionId(id(&input.id)?))
            .await
    }
    pub(super) async fn close_session(&self, input: p::ObjectRequest) -> Result<p::Session> {
        let subject = SubjectId(id(&input.subject_id)?);
        let session = SessionId(id(&input.id)?);
        self.0.cognition.close_session(subject, session).await?;
        self.session_view(subject, session).await
    }
    pub(super) async fn list_sessions(
        &self,
        input: p::ListRequest,
    ) -> Result<p::ListSessionsResponse> {
        let subject = SubjectId(id(&input.subject_id)?);
        self.0.store.require_subject(subject).await?;
        if !matches!(input.status.as_str(), "" | "open" | "closed") {
            return Err(Error::Invalid("invalid session status".into()));
        }
        let scope = format!("sessions:{}:{}", input.subject_id, input.status);
        let (limit, last) = page(input.page, &scope)?;
        let mut ids=sqlx::query_scalar::<_,Uuid>("SELECT session_id FROM cognitive_sessions WHERE subject_id=$1 AND ($2::uuid IS NULL OR session_id>$2) AND ($3='' OR ($3='open' AND closed_at IS NULL) OR ($3='closed' AND closed_at IS NOT NULL)) ORDER BY session_id LIMIT $4")
            .bind(subject.0).bind(last).bind(input.status).bind(limit+1).fetch_all(self.0.store.pool()).await.map_err(db)?;
        let more = ids.len() > limit as usize;
        ids.truncate(limit as usize);
        let next_page_token = if more {
            next_token(&scope, *ids.last().expect("nonempty page"))
        } else {
            String::new()
        };
        let mut items = Vec::new();
        for id in ids {
            items.push(self.session_view(subject, SessionId(id)).await?);
        }
        Ok(p::ListSessionsResponse {
            items,
            next_page_token,
        })
    }
}
