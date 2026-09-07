use chrono::Utc;
use nous_core::{
    API_VERSION, CapabilityOperation, CognitiveQuery, Cue, EmbeddingSpaceSignature, EntityCue,
    EntityRef, ProducerSignature, RepresentationKind, SourceClass, TagCue, TextCue,
};
use nous_material::{
    FormationDirective, ObservationInput, ObservationMaterial, OccurrenceDescriptor,
    ResolvedEntityMention, RuntimeDirective,
};
use nous_memory_domain::{
    EvidenceRef, ExplicitMemoryInput, MemoryClass, MemoryRevisionEvidence, SupportRole, UseKind,
};
use nous_memory_service::{
    CreateSubject, DocumentExtractionOutput, DocumentExtractionProvider, DocumentExtractionRequest,
    LocalRuntime, TextEmbeddingOutput, TextEmbeddingProvider, TextEmbeddingRequest, UseFeedback,
    UseFeedbackEvent,
};
use postgresql_embedded::{PostgreSQL, SettingsBuilder, VersionReq};
use std::sync::Arc;
use tempfile::TempDir;

struct TestExtractor {
    producer: ProducerSignature,
}

struct TestEmbedder;

#[async_trait::async_trait]
impl TextEmbeddingProvider for TestEmbedder {
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
    let runtime = LocalRuntime::open_with_options(
        &url,
        4,
        root.path().join("objects").to_string_lossy().as_ref(),
        1024 * 1024,
    )
    .await
    .expect("open runtime");
    let subject = runtime
        .create_subject(CreateSubject::default())
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
async fn same_artifact_keeps_distinct_occurrences_and_use_is_explicit() {
    let (postgres, url, root) = database().await;
    let runtime = LocalRuntime::open_with_options(
        &url,
        4,
        root.path().join("objects").to_string_lossy().as_ref(),
        1024 * 1024,
    )
    .await
    .expect("open runtime");
    let subject = runtime
        .create_subject(CreateSubject::default())
        .await
        .expect("subject")
        .subject_id;
    let session = runtime
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
            .session(subject, session)
            .await
            .expect("session after observe")
            .resident
            .len()
            >= 2
    );

    let memory = runtime
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
            .session(subject, session)
            .await
            .expect("surface session")
            .last_meaningful_use_at
            .is_none()
    );
    runtime
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
async fn query_uses_rebuilt_entity_tag_and_lexical_projections() {
    let (postgres, url, root) = database().await;
    let runtime = LocalRuntime::open_with_options(
        &url,
        4,
        root.path().join("objects").to_string_lossy().as_ref(),
        1024 * 1024,
    )
    .await
    .expect("open runtime")
    .with_text_embedding_provider(Arc::new(TestEmbedder));
    let subject = runtime
        .create_subject(CreateSubject::default())
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
