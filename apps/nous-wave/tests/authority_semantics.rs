use chrono::Utc;
use nous_cognitive_runtime::{
    ConsumerProfile, ContextBudget, MaterializationPolicy, UseKind, WorkingSetRequest,
};
use nous_cognitive_runtime::{UseFeedback, UseFeedbackEvent};
use nous_core::{
    API_VERSION, CapabilityOperation, CognitiveQuery, CognitiveRef, Cue, EmbeddingSpaceSignature,
    EntityCue, EntityRef, Modality, ProducerSignature, RepresentationKind, SourceClass, TagCue,
    TextCue,
};
use nous_material::{
    FormationDirective, ObservationInput, ObservationMaterial, OccurrenceDescriptor,
    ResolvedEntityMention, RuntimeDirective,
};
use nous_material_service::{
    DocumentExtractionOutput, DocumentExtractionProvider, DocumentExtractionRequest,
};
use nous_memory_domain::{
    EvidenceRef, ExplicitMemoryInput, MemoryClass, MemoryRevisionEvidence, SupportRole,
};
use nous_serving::{
    ServingOptions, TextEmbeddingOutput, TextEmbeddingProvider, TextEmbeddingRequest,
};
use nous_subject_core::{CharacterSeedInput, CreateSubject};
use nous_wave::{NousRuntime, RuntimeOptions};
use postgresql_embedded::{PostgreSQL, SettingsBuilder, VersionReq};
use std::sync::Arc;
use tempfile::TempDir;

struct TestExtractor {
    producer: ProducerSignature,
}

struct TestEmbedder;

#[async_trait::async_trait]
impl TextEmbeddingProvider for TestEmbedder {
    fn space(&self) -> EmbeddingSpaceSignature {
        EmbeddingSpaceSignature {
            space_hash: "test-text-space-v1".into(),
            model_identity: "test".into(),
            weights_revision: "1".into(),
            task: "retrieval".into(),
            input_representation: "text".into(),
            preprocessing_identity: "lowercase".into(),
            preprocessing_revision: "1".into(),
            dimension: 3,
            normalization: "none".into(),
            output_semantics: "dense_similarity".into(),
        }
    }
    fn producer(&self) -> ProducerSignature {
        ProducerSignature {
            signature_hash: "test-text-producer-v1".into(),
            provider_class: "deterministic-library".into(),
            operation: CapabilityOperation::TextEmbedding,
            implementation: "test-embedder".into(),
            model_identity: Some("test".into()),
            model_revision: Some("1".into()),
            preprocessing_identity: "lowercase".into(),
            preprocessing_revision: "1".into(),
            config_digest: "test".into(),
        }
    }

    async fn embed(&self, request: TextEmbeddingRequest) -> nous_core::Result<TextEmbeddingOutput> {
        let text = request.text.to_ascii_lowercase();
        let tag_index = text
            .strip_prefix("tag-")
            .and_then(|value| value.parse::<usize>().ok());
        Ok(TextEmbeddingOutput {
            vector: vec![
                tag_index
                    .map(|index| (index % 3 + 1) as f32)
                    .unwrap_or_else(|| if text.contains("coffee") { 1.0 } else { 0.0 }),
                tag_index
                    .map(|index| (index / 3 + 1) as f32)
                    .unwrap_or_else(|| if text.contains("alice") { 1.0 } else { 0.0 }),
                tag_index
                    .map(|index| 0.25 + index as f32 * 0.1)
                    .unwrap_or(if request.query { 0.5 } else { 0.25 }),
            ],
            space: EmbeddingSpaceSignature {
                space_hash: "test-text-space-v1".into(),
                model_identity: "test".into(),
                weights_revision: "1".into(),
                task: "retrieval".into(),
                input_representation: "text".into(),
                preprocessing_identity: "lowercase".into(),
                preprocessing_revision: "1".into(),
                dimension: 3,
                normalization: "none".into(),
                output_semantics: "dense_similarity".into(),
            },
            producer: ProducerSignature {
                signature_hash: "test-text-producer-v1".into(),
                provider_class: "deterministic-library".into(),
                operation: CapabilityOperation::TextEmbedding,
                implementation: "test-embedder".into(),
                model_identity: Some("test".into()),
                model_revision: Some("1".into()),
                preprocessing_identity: "lowercase".into(),
                preprocessing_revision: "1".into(),
                config_digest: "test".into(),
            },
        })
    }
}

#[tokio::test]
async fn derivation_claim_and_provider_commit_preserve_textual_surrogate() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, true, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let observed = runtime
        .observe(ObservationInput {
            subject,
            session: None,
            occurrence: OccurrenceDescriptor {
                source_class: SourceClass::File,
                external_object_ref: None,
                occurred_at: None,
                observed_at: Utc::now(),
                conversation_ref: None,
                actor_entity_ref: None,
                context: serde_json::json!({}),
            },
            material: ObservationMaterial::InlineText {
                text: "raw document bytes".into(),
                media_type: "application/pdf".into(),
            },
            entities: Vec::new(),
            formation: FormationDirective::None,
            runtime: RuntimeDirective::default(),
        })
        .await
        .expect("observation");
    let producer = ProducerSignature {
        signature_hash: "test-document-producer-v1".into(),
        provider_class: "deterministic-library".into(),
        operation: CapabilityOperation::DocumentExtraction,
        implementation: "test-extractor".into(),
        model_identity: None,
        model_revision: None,
        preprocessing_identity: "identity".into(),
        preprocessing_revision: "1".into(),
        config_digest: "test".into(),
    };
    let _derivation = runtime
        .material
        .schedule_derivation(
            subject,
            observed
                .source_region
                .expect("source region")
                .source_region_id,
            RepresentationKind::ExtractedText,
            CapabilityOperation::DocumentExtraction,
            producer.clone(),
            "preferred",
        )
        .await
        .expect("schedule derivation");
    let run = runtime
        .material
        .run_pending_derivations(1, "test-worker", Arc::new(TestExtractor { producer }))
        .await
        .expect("run derivation");
    assert_eq!(run.claimed, 1);
    assert_eq!(run.succeeded, 1);

    let result = runtime
        .query(CognitiveQuery {
            api_version: API_VERSION,
            subject,
            session: None,
            situation: Default::default(),
            targets: vec![nous_core::QueryTarget::Evidence],
            cues: vec![Cue::Text(TextCue {
                text: "persisted extracted surrogate".into(),
            })],
            constraints: Default::default(),
            exploration: Default::default(),
            resources: Default::default(),
            result_need: Default::default(),
            effort: Default::default(),
            capabilities: Default::default(),
            diagnostics: Default::default(),
        })
        .await
        .expect("surrogate query");
    assert!(result.results.iter().any(|hit| {
        matches!(
            hit.reference,
            nous_core::CognitiveRef::DerivedRepresentation(_)
        )
    }));

    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[async_trait::async_trait]
impl DocumentExtractionProvider for TestExtractor {
    async fn extract(
        &self,
        _request: DocumentExtractionRequest,
    ) -> nous_core::Result<DocumentExtractionOutput> {
        Ok(DocumentExtractionOutput {
            representation_kind: RepresentationKind::ExtractedText,
            text: "persisted extracted surrogate".into(),
            producer: self.producer.clone(),
            derived_regions: Vec::new(),
            warnings: Vec::new(),
            coverage: serde_json::json!({"complete":true}),
        })
    }
}

async fn database() -> (PostgreSQL, String, TempDir) {
    let root = TempDir::new().expect("temporary PostgreSQL root");
    let settings = SettingsBuilder::new()
        .version(VersionReq::parse("=18.6.0").expect("version"))
        .host("127.0.0.1")
        .port(0)
        .username("postgres")
        .password("nous_wave")
        .installation_dir(root.path().join("install"))
        .data_dir(root.path().join("data"))
        .password_file(root.path().join("postgres.pgpass"))
        .temporary(false)
        .build();
    let mut postgres = PostgreSQL::new(settings);
    postgres.setup().await.expect("setup PostgreSQL");
    postgres.start().await.expect("start PostgreSQL");
    postgres
        .create_database("nous_semantics")
        .await
        .expect("create test database");
    let url = postgres.settings().url("nous_semantics");
    (postgres, url, root)
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn same_artifact_keeps_distinct_occurrences_and_use_is_explicit() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, true, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let session = runtime
        .cognition
        .open_session(subject, serde_json::json!({}))
        .await
        .expect("session")
        .session_id;
    let input = ObservationInput {
        subject,
        session: Some(session),
        occurrence: OccurrenceDescriptor {
            source_class: SourceClass::Message,
            external_object_ref: None,
            occurred_at: None,
            observed_at: Utc::now(),
            conversation_ref: Some("first".into()),
            actor_entity_ref: None,
            context: serde_json::json!({"turn":1}),
        },
        material: ObservationMaterial::InlineText {
            text: "same bytes, different encounter".into(),
            media_type: "text/plain".into(),
        },
        entities: Vec::<ResolvedEntityMention>::new(),
        formation: FormationDirective::None,
        runtime: RuntimeDirective::default(),
    };
    let first = runtime
        .observe(input.clone())
        .await
        .expect("first observation");
    let second = runtime.observe(input).await.expect("second observation");
    assert_eq!(
        first.artifact.as_ref().map(|artifact| artifact.artifact_id),
        second
            .artifact
            .as_ref()
            .map(|artifact| artifact.artifact_id)
    );
    assert_ne!(
        first.occurrence.occurrence_id,
        second.occurrence.occurrence_id
    );
    assert!(
        runtime
            .cognition
            .session(subject, session)
            .await
            .expect("session after observe")
            .resident
            .len()
            >= 2
    );

    let memory = runtime
        .require_memory()
        .expect("memory")
        .form_memory(ExplicitMemoryInput {
            subject,
            memory_class: MemoryClass::Specific,
            semantic_role: "fact".into(),
            representation_text: "The Subject encountered the same text twice.".into(),
            title: None,
            evidence: vec![MemoryRevisionEvidence {
                evidence_no: 0,
                evidence: EvidenceRef::Occurrence {
                    occurrence_id: first.occurrence.occurrence_id,
                },
                support_role: SupportRole::Direct,
                weight: Some(1.0),
            }],
            entity_refs: Vec::new(),
            tags: Vec::new(),
            occurred_at: None,
            observed_at: Utc::now(),
            valid_from: None,
            valid_to: None,
            epistemic_class: nous_core::EpistemicClass::Observed,
            confidence: None,
        })
        .await
        .expect("explicit memory");
    runtime
        .cognition
        .use_feedback(UseFeedback {
            subject,
            session_id: Some(session),
            consumer: Some("test".into()),
            events: vec![UseFeedbackEvent {
                reference: nous_core::CognitiveRef::Memory(memory.object.memory_id),
                use_kind: UseKind::Surfaced,
                context: serde_json::json!({}),
            }],
        })
        .await
        .expect("surface feedback");
    assert!(
        runtime
            .cognition
            .session(subject, session)
            .await
            .expect("surface session")
            .last_meaningful_use_at
            .is_none()
    );
    runtime
        .cognition
        .use_feedback(UseFeedback {
            subject,
            session_id: Some(session),
            consumer: Some("test".into()),
            events: vec![UseFeedbackEvent {
                reference: nous_core::CognitiveRef::Memory(memory.object.memory_id),
                use_kind: UseKind::Referenced,
                context: serde_json::json!({}),
            }],
        })
        .await
        .expect("meaningful feedback");
    assert!(
        runtime
            .cognition
            .session(subject, session)
            .await
            .expect("meaningful session")
            .last_meaningful_use_at
            .is_some()
    );

    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn query_uses_rebuilt_entity_tag_and_lexical_projections() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, true, Some(Arc::new(TestEmbedder))).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let entity = EntityRef::new("entity:host-person:alice").expect("entity");
    let observed = runtime
        .observe(ObservationInput {
            subject,
            session: None,
            occurrence: OccurrenceDescriptor {
                source_class: SourceClass::Message,
                external_object_ref: None,
                occurred_at: None,
                observed_at: Utc::now(),
                conversation_ref: None,
                actor_entity_ref: Some(entity.clone()),
                context: serde_json::json!({}),
            },
            material: ObservationMaterial::InlineText {
                text: "Alice prefers Ethiopian coffee".into(),
                media_type: "text/plain".into(),
            },
            entities: vec![ResolvedEntityMention {
                surface: "Alice".into(),
                entity_ref: Some(entity.clone()),
                semantic_role: Some("subject".into()),
            }],
            formation: FormationDirective::None,
            runtime: RuntimeDirective::default(),
        })
        .await
        .expect("observation");
    let tag = runtime
        .require_memory()
        .expect("memory")
        .create_tag(
            subject,
            nous_memory_service::CreateTagRequest {
                label: "coffee".into(),
                description: None,
                kind_hint: Some("preference".into()),
                origin: "explicit".into(),
            },
        )
        .await
        .expect("tag");
    for index in 0..8 {
        runtime
            .require_memory()
            .expect("memory")
            .create_tag(
                subject,
                nous_memory_service::CreateTagRequest {
                    label: format!("tag-{index}"),
                    description: None,
                    kind_hint: None,
                    origin: "explicit".into(),
                },
            )
            .await
            .expect("EPA tag");
    }
    let memory = runtime
        .require_memory()
        .expect("memory")
        .form_memory(ExplicitMemoryInput {
            subject,
            memory_class: MemoryClass::Specific,
            semantic_role: "preference".into(),
            representation_text: "Alice prefers Ethiopian coffee".into(),
            title: None,
            evidence: vec![MemoryRevisionEvidence {
                evidence_no: 0,
                evidence: EvidenceRef::Occurrence {
                    occurrence_id: observed.occurrence.occurrence_id,
                },
                support_role: SupportRole::Direct,
                weight: Some(1.0),
            }],
            entity_refs: vec![entity.clone()],
            tags: vec![tag.tag_id],
            occurred_at: None,
            observed_at: Utc::now(),
            valid_from: None,
            valid_to: None,
            epistemic_class: nous_core::EpistemicClass::Reported,
            confidence: Some(0.9),
        })
        .await
        .expect("memory");
    let query = CognitiveQuery {
        api_version: API_VERSION,
        subject,
        session: None,
        situation: Default::default(),
        targets: vec![nous_core::QueryTarget::Memory],
        cues: vec![
            Cue::Text(TextCue {
                text: "Ethiopian coffee".into(),
            }),
            Cue::Entity(EntityCue { entity_ref: entity }),
            Cue::Tag(TagCue { tag: tag.tag_id }),
        ],
        constraints: Default::default(),
        exploration: Default::default(),
        resources: Default::default(),
        result_need: Default::default(),
        effort: Default::default(),
        capabilities: Default::default(),
        diagnostics: Default::default(),
    };
    let result = runtime.query(query).await.expect("query");
    assert!(result.generation.epa_basis.is_some());
    assert!(
        result
            .results
            .iter()
            .any(|hit| hit.reference == nous_core::CognitiveRef::Memory(memory.object.memory_id))
    );
    let hit = result
        .results
        .iter()
        .find(|hit| hit.reference == nous_core::CognitiveRef::Memory(memory.object.memory_id))
        .expect("memory hit");
    assert!(
        hit.match_evidence
            .families
            .contains(&nous_core::EvidenceFamily::Entity)
    );
    assert!(
        hit.match_evidence
            .families
            .contains(&nous_core::EvidenceFamily::Lexical)
    );
    assert!(
        hit.match_evidence
            .families
            .contains(&nous_core::EvidenceFamily::TagDirect)
    );
    assert!(
        hit.match_evidence
            .families
            .contains(&nous_core::EvidenceFamily::SemanticDense)
    );

    let evidence_result = runtime
        .query(CognitiveQuery {
            api_version: API_VERSION,
            subject,
            session: None,
            situation: Default::default(),
            targets: vec![nous_core::QueryTarget::Evidence],
            cues: vec![Cue::Text(TextCue {
                text: "Alice prefers Ethiopian".into(),
            })],
            constraints: Default::default(),
            exploration: Default::default(),
            resources: Default::default(),
            result_need: Default::default(),
            effort: Default::default(),
            capabilities: Default::default(),
            diagnostics: Default::default(),
        })
        .await
        .expect("evidence query");
    assert!(evidence_result.results.iter().any(|hit| {
        hit.reference == nous_core::CognitiveRef::Occurrence(observed.occurrence.occurrence_id)
    }));

    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

struct FailingEmbedder;

#[async_trait::async_trait]
impl TextEmbeddingProvider for FailingEmbedder {
    fn space(&self) -> EmbeddingSpaceSignature {
        EmbeddingSpaceSignature {
            space_hash: "failing-space-v1".into(),
            model_identity: "failing".into(),
            weights_revision: "1".into(),
            task: "retrieval".into(),
            input_representation: "text".into(),
            preprocessing_identity: "identity".into(),
            preprocessing_revision: "1".into(),
            dimension: 3,
            normalization: "none".into(),
            output_semantics: "dense_similarity".into(),
        }
    }

    fn producer(&self) -> ProducerSignature {
        ProducerSignature {
            signature_hash: "failing-producer-v1".into(),
            provider_class: "test".into(),
            operation: CapabilityOperation::TextEmbedding,
            implementation: "failing".into(),
            model_identity: None,
            model_revision: None,
            preprocessing_identity: "identity".into(),
            preprocessing_revision: "1".into(),
            config_digest: "test".into(),
        }
    }

    async fn embed(
        &self,
        _request: TextEmbeddingRequest,
    ) -> nous_core::Result<TextEmbeddingOutput> {
        Err(nous_core::Error::Unavailable(
            "test embedding failure".into(),
        ))
    }
}

#[tokio::test]
async fn subject_core_and_runtime_survive_without_memory() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, false, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let seed = runtime
        .subjects
        .character_seed(subject, None)
        .await
        .expect("seed");
    assert!(seed.text.contains("原始种子"));
    let session = runtime
        .cognition
        .open_session(subject, serde_json::json!({}))
        .await
        .expect("session");
    let observed = runtime
        .observe(ObservationInput {
            subject,
            session: Some(session.session_id),
            occurrence: OccurrenceDescriptor {
                source_class: SourceClass::Message,
                external_object_ref: None,
                occurred_at: None,
                observed_at: Utc::now(),
                conversation_ref: None,
                actor_entity_ref: None,
                context: serde_json::json!({}),
            },
            material: ObservationMaterial::InlineText {
                text: "runtime survives without memory".into(),
                media_type: "text/plain".into(),
            },
            entities: Vec::new(),
            formation: FormationDirective::None,
            runtime: RuntimeDirective::default(),
        })
        .await
        .expect("observation");
    assert!(observed.resident);
    let session = runtime
        .cognition
        .session(subject, session.session_id)
        .await
        .expect("session view");
    assert!(session.resident.iter().any(|resident| resident.reference
        == CognitiveRef::Occurrence(observed.occurrence.occurrence_id)));
    let error = runtime
        .query(CognitiveQuery {
            api_version: API_VERSION,
            subject,
            session: Some(session.session_id),
            situation: Default::default(),
            targets: vec![nous_core::QueryTarget::Memory],
            cues: Vec::new(),
            constraints: Default::default(),
            exploration: Default::default(),
            resources: Default::default(),
            result_need: Default::default(),
            effort: Default::default(),
            capabilities: Default::default(),
            diagnostics: Default::default(),
        })
        .await
        .expect_err("memory should be unavailable");
    assert!(matches!(error, nous_core::Error::Unavailable(_)));
    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[tokio::test]
async fn consumer_profiles_produce_different_bounded_working_sets() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, false, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let session = runtime
        .cognition
        .open_session(subject, serde_json::json!({}))
        .await
        .expect("session");
    runtime
        .observe(ObservationInput {
            subject,
            session: Some(session.session_id),
            occurrence: OccurrenceDescriptor {
                source_class: SourceClass::Message,
                external_object_ref: None,
                occurred_at: None,
                observed_at: Utc::now(),
                conversation_ref: None,
                actor_entity_ref: None,
                context: serde_json::json!({}),
            },
            material: ObservationMaterial::InlineText {
                text: "bounded consumer context".into(),
                media_type: "text/plain".into(),
            },
            entities: Vec::new(),
            formation: FormationDirective::None,
            runtime: RuntimeDirective::default(),
        })
        .await
        .expect("observation");
    let references_only = runtime
        .cognition
        .working_set(
            WorkingSetRequest {
                subject,
                session_id: session.session_id,
                consumer: ConsumerProfile {
                    consumer_id: "handles".into(),
                    context_budget: ContextBudget {
                        max_items: 4,
                        max_text_bytes: 0,
                    },
                    accepted_modalities: vec![Modality::Text],
                    materialization_policy: MaterializationPolicy::ReferencesOnly,
                },
                query_results: Vec::new(),
                references: Vec::new(),
            },
            &runtime,
        )
        .await
        .expect("handles working set");
    let text_consumer = runtime
        .cognition
        .working_set(
            WorkingSetRequest {
                subject,
                session_id: session.session_id,
                consumer: ConsumerProfile {
                    consumer_id: "text".into(),
                    context_budget: ContextBudget {
                        max_items: 1,
                        max_text_bytes: 8,
                    },
                    accepted_modalities: vec![Modality::Text],
                    materialization_policy: MaterializationPolicy::AvailableText,
                },
                query_results: Vec::new(),
                references: Vec::new(),
            },
            &runtime,
        )
        .await
        .expect("text working set");
    assert_ne!(references_only.consumer_id, text_consumer.consumer_id);
    assert!(
        references_only
            .contributions
            .iter()
            .all(|item| item.text.is_none())
    );
    assert!(text_consumer.budget_used.items <= 1 && text_consumer.budget_used.text_bytes <= 8);
    assert!(
        text_consumer
            .contributions
            .iter()
            .any(|item| item.text.is_some())
    );
    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[tokio::test]
async fn observation_admission_survives_embedding_failure() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, true, Some(Arc::new(FailingEmbedder))).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let session = runtime
        .cognition
        .open_session(subject, serde_json::json!({}))
        .await
        .expect("session");
    let observed = runtime
        .observe(ObservationInput {
            subject,
            session: Some(session.session_id),
            occurrence: OccurrenceDescriptor {
                source_class: SourceClass::Message,
                external_object_ref: None,
                occurred_at: None,
                observed_at: Utc::now(),
                conversation_ref: None,
                actor_entity_ref: None,
                context: serde_json::json!({}),
            },
            material: ObservationMaterial::InlineText {
                text: "provider failure must not erase observation".into(),
                media_type: "text/plain".into(),
            },
            entities: Vec::new(),
            formation: FormationDirective::None,
            runtime: RuntimeDirective::default(),
        })
        .await
        .expect("observation admission");
    let error = runtime
        .query(CognitiveQuery {
            api_version: API_VERSION,
            subject,
            session: None,
            situation: Default::default(),
            targets: vec![nous_core::QueryTarget::Evidence],
            cues: vec![Cue::Text(TextCue {
                text: "provider failure".into(),
            })],
            constraints: Default::default(),
            exploration: Default::default(),
            resources: Default::default(),
            result_need: Default::default(),
            effort: Default::default(),
            capabilities: nous_core::CapabilityPolicy {
                text_embedding: nous_core::RequirementStrength::Required,
                ..Default::default()
            },
            diagnostics: Default::default(),
        })
        .await
        .expect_err("required provider should fail truthfully");
    assert!(matches!(error, nous_core::Error::Unavailable(_)));
    let session = runtime
        .cognition
        .session(subject, session.session_id)
        .await
        .expect("session view");
    assert!(session.resident.iter().any(|resident| resident.reference
        == CognitiveRef::Occurrence(observed.occurrence.occurrence_id)));
    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

fn seed_input() -> CreateSubject {
    CreateSubject {
        subject_id: None,
        character_seed: CharacterSeedInput {
            text: "Human authored initial character. 原始种子。".into(),
            media_type: "text/plain".into(),
            provenance: serde_json::json!({"author":"human"}),
        },
        config: serde_json::json!({}),
    }
}
async fn open_runtime(
    url: &str,
    root: &TempDir,
    memory_enabled: bool,
    embedding: Option<Arc<dyn TextEmbeddingProvider>>,
) -> NousRuntime {
    NousRuntime::open(RuntimeOptions {
        postgres_url: url.into(),
        max_connections: 4,
        object_root: root.path().join("objects").to_string_lossy().into_owned(),
        max_upload_bytes: 1024 * 1024,
        resident_limit: 256,
        memory_enabled,
        serving_options: ServingOptions {
            root: root.path().join("serving"),
            lexical: true,
            dense: true,
            topology: true,
            memory_enabled,
        },
        embedding,
    })
    .await
    .expect("open runtime")
}
