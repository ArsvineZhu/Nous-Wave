use super::*;

impl MemoryService {
    pub fn new(
        store: AuthorityStore,
        objects: ObjectStore,
        cognition: nous_cognitive_runtime::CognitiveRuntimeService,
        serving: nous_serving::ServingService,
    ) -> Self {
        Self {
            accessibility_policy: AccessibilityPolicy::default(),
            store,
            objects,
            cognition,
            serving,
            capabilities: Arc::new(Vec::new()),
            cue_sensing: Arc::new(EpaResidualCueSensing),
            expansion: Arc::new(BoundedWaveExpansion),
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<CapabilityDescriptor>) -> Self {
        self.capabilities = Arc::new(capabilities);
        self
    }

    pub async fn status(&self) -> RuntimeStatus {
        let snapshot = self.serving.publisher.snapshot();
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
        RuntimeStatus {
            api_version: API_VERSION,
            ready: true,
            authority: "postgresql".into(),
            capabilities,
            serving_generation: self.serving.publisher.snapshot().generation,
        }
    }
}
