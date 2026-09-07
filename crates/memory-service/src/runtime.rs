use super::support::*;
use super::*;

impl LocalRuntime {
    pub async fn open_with_options(
        postgres_url: &str,
        max_connections: u32,
        object_root: &str,
        max_upload_bytes: u64,
    ) -> Result<Self> {
        let store = MemoryStore::connect(postgres_url, max_connections).await?;
        store.migrate().await?;
        let objects = ObjectStore::open(object_root).await?;
        let runtime = Self {
            store,
            objects,
            publisher: ServingPublisher::default(),
            max_upload_bytes,
            resident_limit: 256,
            capabilities: Arc::new(Vec::new()),
            resource_resolvers: Arc::new(RwLock::new(HashMap::new())),
            text_embedding_provider: None,
            memory_formation_provider: None,
        };
        // Serving files are rebuildable projections. The in-process snapshot is
        // republished from Authority on startup so a process restart never
        // makes durable Memory disappear merely because the old process ended.
        let subjects = sqlx::query_scalar::<_, Uuid>(
            "SELECT subject_id FROM subjects WHERE status <> 'purging' ORDER BY subject_id",
        )
        .fetch_all(runtime.store.pool())
        .await
        .map_err(db)?;
        for subject in subjects {
            runtime.rebuild_projection(SubjectId(subject)).await?;
        }
        Ok(runtime)
    }

    pub fn with_capabilities(mut self, capabilities: Vec<CapabilityDescriptor>) -> Self {
        self.capabilities = Arc::new(capabilities);
        self
    }

    pub fn with_resource_resolver(
        self,
        resolver_key: impl Into<String>,
        resolver: Arc<dyn ResourceResolver>,
    ) -> Self {
        if let Ok(mut resolvers) = self.resource_resolvers.write() {
            resolvers.insert(resolver_key.into(), resolver);
        }
        self
    }

    pub fn with_text_embedding_provider(
        mut self,
        provider: Arc<dyn TextEmbeddingProvider>,
    ) -> Self {
        self.text_embedding_provider = Some(provider);
        self
    }

    pub fn with_memory_formation_provider(
        mut self,
        provider: Arc<dyn MemoryFormationProvider>,
    ) -> Self {
        self.memory_formation_provider = Some(provider);
        self
    }

    pub fn with_resident_limit(mut self, resident_limit: usize) -> Result<Self> {
        if resident_limit == 0 {
            return Err(Error::Invalid("resident_limit must be positive".into()));
        }
        self.resident_limit = resident_limit;
        Ok(self)
    }

    pub async fn status(&self) -> RuntimeStatus {
        let snapshot = self.publisher.snapshot();
        let mut capabilities = self
            .capabilities
            .iter()
            .map(|capability| CapabilityStatus {
                capability_id: capability.operation.as_str().into(),
                status: match capability.readiness {
                    CapabilityReadiness::Ready => Readiness::Ready,
                    CapabilityReadiness::Degraded => Readiness::Degraded,
                    CapabilityReadiness::Unavailable => Readiness::Unavailable,
                },
                reason: None,
            })
            .collect::<Vec<_>>();
        capabilities.extend([
            CapabilityStatus {
                capability_id: "authority.postgresql".into(),
                status: Readiness::Ready,
                reason: None,
            },
            CapabilityStatus {
                capability_id: "object.cas".into(),
                status: Readiness::Ready,
                reason: None,
            },
            CapabilityStatus {
                capability_id: "retrieval.lexical".into(),
                status: if snapshot.lexical.is_some() {
                    Readiness::Ready
                } else {
                    Readiness::Unavailable
                },
                reason: (!snapshot.lexical.is_some())
                    .then_some("no published Tantivy generation".into()),
            },
            CapabilityStatus {
                capability_id: "retrieval.wave".into(),
                status: if snapshot.wave.is_some() {
                    Readiness::Ready
                } else {
                    Readiness::Unavailable
                },
                reason: (!snapshot.wave.is_some()).then_some("no published Wave generation".into()),
            },
            CapabilityStatus {
                capability_id: "retrieval.dense".into(),
                status: if snapshot.dense.is_empty() {
                    Readiness::Unavailable
                } else {
                    Readiness::Ready
                },
                reason: snapshot
                    .dense
                    .is_empty()
                    .then_some("no compatible dense generation".into()),
            },
            CapabilityStatus {
                capability_id: "retrieval.epa".into(),
                status: if snapshot.epa.is_empty() {
                    Readiness::Unavailable
                } else {
                    Readiness::Ready
                },
                reason: snapshot
                    .epa
                    .is_empty()
                    .then_some("fewer than eight compatible Tag vectors".into()),
            },
        ]);
        if self.memory_formation_provider.is_some() {
            capabilities.push(CapabilityStatus {
                capability_id: "memory.formation.text".into(),
                status: Readiness::Ready,
                reason: None,
            });
        }
        RuntimeStatus {
            api_version: API_VERSION,
            ready: true,
            authority: "postgresql".into(),
            capabilities,
            serving_generation: self.publisher.snapshot().generation,
        }
    }

    pub async fn create_subject(&self, input: CreateSubject) -> Result<SubjectView> {
        let subject_id = input.subject_id.unwrap_or_default();
        let now = Utc::now();
        sqlx::query("INSERT INTO subjects(subject_id,created_at,metadata) VALUES($1,$2,$3)")
            .bind(subject_id.0)
            .bind(now)
            .bind(input.metadata)
            .execute(self.store.pool())
            .await
            .map_err(db)?;
        self.subject(subject_id).await
    }

    pub async fn subject(&self, subject: SubjectId) -> Result<SubjectView> {
        let row = sqlx::query(
            "SELECT subject_id,created_at,state_revision,status,metadata FROM subjects WHERE subject_id=$1",
        )
        .bind(subject.0)
        .fetch_optional(self.store.pool())
        .await
        .map_err(db)?
        .ok_or_else(|| Error::NotFound("subject not found".into()))?;
        Ok(SubjectView {
            subject_id: SubjectId(row.try_get("subject_id").map_err(db)?),
            created_at: row.try_get("created_at").map_err(db)?,
            state_revision: row.try_get("state_revision").map_err(db)?,
            status: row.try_get("status").map_err(db)?,
            metadata: row.try_get("metadata").map_err(db)?,
        })
    }

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
}
