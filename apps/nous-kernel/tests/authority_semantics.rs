use chrono::Utc;
use nous_authority_store::ProjectionInvalidation;
use nous_cognitive_runtime::{
    ConsumerProfile, ContextBudget, MaterializationPolicy, ResourceUpsert, UseKind,
    WorkingSetRequest,
};
use nous_cognitive_runtime::{UseFeedback, UseFeedbackEvent};
use nous_core::{
    API_VERSION, CapabilityOperation, CognitiveQuery, CognitiveRef, Cue, EmbeddingSpaceSignature,
    EntityCue, EntityRef, Modality, ProducerSignature, RepresentationKind, ServingNeed,
    SourceClass, SubjectId, TagCue, TextCue,
};
use nous_kernel::{NousRuntime, RuntimeOptions};
use nous_material::{
    ObservationInput, ObservationMaterial, OccurrenceDescriptor, ResolvedEntityMention,
    RuntimeDirective,
};
use nous_material_service::{
    DocumentExtractionOutput, DocumentExtractionProvider, DocumentExtractionRequest,
};
use nous_memory_domain::{
    AssociationPolarity, AssociationSupportClass, ConsolidationRequest, ConsolidationTarget,
    EvidenceRef, ExplicitMemoryInput, MemoryClass, MemoryRevisionEvidence,
    SupportRole, TopologyAnchorProposal, TopologyAnchorSupport, TopologyAssociationProposal,
    TopologyConsolidationProposal, TopologyRevisionProposal, TopologyTagProposal,
};
use nous_serving::{
    ServingOptions, TextEmbeddingOutput, TextEmbeddingProvider, TextEmbeddingRequest,
};
use nous_subject_core::{CharacterSeedInput, CreateSubject};
use postgresql_embedded::{PostgreSQL, SettingsBuilder, VersionReq};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

fn semantic_memory_input(subject:SubjectId,occurrence:nous_core::OccurrenceId,text:&str)->ExplicitMemoryInput {
    ExplicitMemoryInput{subject,memory_class:MemoryClass::Specific,semantic_role:"episode".into(),representation_text:text.into(),title:None,
        evidence:vec![MemoryRevisionEvidence{evidence_no:0,evidence:EvidenceRef::Occurrence{occurrence_id:occurrence},support_role:SupportRole::Direct}],
        entity_refs:vec![],tags:vec![],tag_order_provenance:None,valid_from:None,valid_to:None,epistemic_class:nous_core::EpistemicClass::Observed}
}
fn semantic_memory_query(subject:SubjectId,effort:nous_core::CognitiveEffort)->CognitiveQuery {
    CognitiveQuery{api_version:API_VERSION,subject,session:None,situation:Default::default(),targets:vec![nous_core::QueryTarget::Memory],cues:vec![Cue::Text(TextCue{text:"event".into()})],constraints:Default::default(),exploration:Default::default(),resources:Default::default(),result_need:Default::default(),effort,capabilities:Default::default(),diagnostics:Default::default()}
}
async fn semantic_observation(runtime:&NousRuntime,subject:SubjectId,when:chrono::DateTime<Utc>)->nous_material::AcceptedObservation {
    runtime.material.record_observation(ObservationInput{subject,session:None,occurrence:OccurrenceDescriptor{source_class:SourceClass::Message,external_object_ref:None,occurred_at:Some(when),observed_at:when,conversation_ref:None,actor_entity_ref:None,context:serde_json::json!({})},material:ObservationMaterial::InlineText{text:"event with no named participant".into(),media_type:"text/plain".into()},entities:vec![],runtime:RuntimeDirective::default()}).await.expect("observation")
}

#[tokio::test]
async fn semantic_closure_lineage_does_not_fabricate_mentions_times_or_identity_merges(){
    use nous_memory_domain::{RevisionLifecycle,MemoryFormationProposal,TagProposal};
    let (postgres,url,root)=database().await;let runtime=open_runtime(&url,&root,true,None).await;
    let subject=runtime.subjects.create_subject(seed_input()).await.unwrap().subject_id;
    let early=chrono::DateTime::from_timestamp_micros((Utc::now()-chrono::Duration::days(50)).timestamp_micros()).unwrap();let late=chrono::DateTime::from_timestamp_micros((Utc::now()-chrono::Duration::days(10)).timestamp_micros()).unwrap();
    let first=semantic_observation(&runtime,subject,early).await;let second=semantic_observation(&runtime,subject,late).await;
    let memory=runtime.require_memory().unwrap();
    let mut input=semantic_memory_input(subject,first.occurrence.occurrence_id,"event reported as Friday");
    input.entity_refs=vec![EntityRef::new("entity:test:alice").unwrap()];
    input.evidence.push(MemoryRevisionEvidence{evidence_no:1,evidence:EvidenceRef::Occurrence{occurrence_id:second.occurrence.occurrence_id},support_role:SupportRole::Contradiction});
    let m1=memory.form_memory(input).await.unwrap();
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM entity_mentions WHERE subject_id=$1").bind(subject.0).fetch_one(runtime.store.pool()).await.unwrap();
    assert_eq!(count,0);assert_eq!(m1.entities.len(),1);
    assert_eq!(m1.temporal_evidence.observed_min,Some(early));assert_eq!(m1.temporal_evidence.observed_max,Some(late));
    let m2=memory.form_memory(semantic_memory_input(subject,second.occurrence.occurrence_id,"event reported as Thursday")).await.unwrap();
    memory.link_revisions(subject,m2.revision.memory_revision_id,m1.revision.memory_revision_id,nous_memory_domain::MemoryRelation::Contradicts).await.unwrap();
    let m1=memory.memory(subject,m1.object.memory_id,None).await.unwrap();
    assert_eq!(m1.revision.revision_lifecycle,RevisionLifecycle::Current);
    assert!(m1.relations.iter().any(|r|r.relation==nous_memory_domain::MemoryRelation::Contradicts));
    let mut query=semantic_memory_query(subject,nous_core::CognitiveEffort::Normal);
    query.constraints.observed=Some(nous_core::TimeInterval{start:Some(early+chrono::Duration::days(10)),end:Some(late-chrono::Duration::days(10))});
    assert!(runtime.query(query.clone()).await.unwrap().results.is_empty(),"aggregate ranges must not fill a gap in actual encounters");
    query.constraints.observed=Some(nous_core::TimeInterval{start:Some(late-chrono::Duration::minutes(1)),end:Some(late+chrono::Duration::minutes(1))});
    query.constraints.occurred=Some(nous_core::TimeInterval{start:Some(early-chrono::Duration::minutes(1)),end:Some(early+chrono::Duration::minutes(1))});
    assert!(runtime.query(query).await.unwrap().results.is_empty(),"time axes must match one encounter");
    let proposal=||MemoryFormationProposal{memory_class:MemoryClass::Specific,semantic_role:"episode".into(),representation_text:"event with proposed tag".into(),title:None,evidence:m1.evidence.clone(),entity_refs:vec![],tag_proposals:vec![TagProposal::New{label:"school".into(),description:None,kind_hint:None}],valid_from:None,valid_to:None,epistemic_class:nous_core::EpistemicClass::Derived};
    let tagged1=memory.commit_formation_proposal(subject,proposal(),&[]).await.unwrap();
    let tagged2=memory.commit_formation_proposal(subject,proposal(),&[]).await.unwrap();
    assert_ne!(tagged1.tags,tagged2.tags,"a shared label is not a shared Tag identity");
    runtime.store.close().await;postgres.stop().await.unwrap();
}

#[tokio::test]
async fn semantic_closure_accessibility_changes_recall_without_changing_authority_or_truth(){
    use nous_memory_domain::{AccessibilityMode as Mode,AccessibilityLevel as Level};
    use nous_core::CognitiveEffort as Effort;
    let (postgres,url,root)=database().await;let runtime=open_runtime(&url,&root,true,None).await;
    let subject=runtime.subjects.create_subject(seed_input()).await.unwrap().subject_id;
    let observation=semantic_observation(&runtime,subject,Utc::now()).await;let memory=runtime.require_memory().unwrap();
    let formed=memory.form_memory(semantic_memory_input(subject,observation.occurrence.occurrence_id,"event accessibility fixture")).await.unwrap();
    let deep=memory.set_accessibility(subject,formed.object.memory_id,formed.object.head_revision,Mode::Deep).await.unwrap();
    assert!(runtime.query(semantic_memory_query(subject,Effort::Normal)).await.unwrap().results.is_empty());
    assert_eq!(runtime.query(semantic_memory_query(subject,Effort::Deep)).await.unwrap().results.len(),1);
    let explicit=memory.set_accessibility(subject,deep.object.memory_id,deep.object.head_revision,Mode::Explicit).await.unwrap();
    assert!(runtime.query(semantic_memory_query(subject,Effort::Maximum)).await.unwrap().results.is_empty());
    let mut exact=semantic_memory_query(subject,Effort::Light);exact.cues.clear();exact.targets=vec![nous_core::QueryTarget::Exact{reference:CognitiveRef::Memory(explicit.object.memory_id)}];
    assert_eq!(runtime.query(exact.clone()).await.unwrap().results.len(),1);
    let suppressed=memory.suppress(subject,explicit.object.memory_id,explicit.object.head_revision).await.unwrap();
    assert!(runtime.query(exact).await.unwrap().results.is_empty(),"exact recall cannot bypass suppression");
    let restored=memory.restore(subject,suppressed.object.memory_id,suppressed.object.head_revision).await.unwrap();
    let auto=memory.set_accessibility(subject,restored.object.memory_id,restored.object.head_revision,Mode::Auto).await.unwrap();
    sqlx::query("UPDATE memory_objects SET created_at=now()-interval '400 days' WHERE memory_id=$1").bind(auto.object.memory_id.0).execute(runtime.store.pool()).await.unwrap();
    assert_eq!(memory.accessibility_level(subject,auto.object.memory_id,Utc::now()).await.unwrap(),Level::Explicit);
    let feedback=|kind|UseFeedback{subject,session_id:None,consumer:Some("semantic-test".into()),events:vec![UseFeedbackEvent{reference:CognitiveRef::MemoryRevision(auto.revision.memory_revision_id),use_kind:kind,context:serde_json::json!({})}]};
    runtime.cognition.use_feedback(feedback(UseKind::Exposed)).await.unwrap();
    assert_eq!(memory.accessibility_level(subject,auto.object.memory_id,Utc::now()).await.unwrap(),Level::Explicit);
    runtime.cognition.use_feedback(feedback(UseKind::Referenced)).await.unwrap();
    assert_eq!(memory.accessibility_level(subject,auto.object.memory_id,Utc::now()).await.unwrap(),Level::Normal);
    let after=memory.memory(subject,auto.object.memory_id,None).await.unwrap();
    assert_eq!(after.revision.memory_revision_id,formed.revision.memory_revision_id);
    assert_eq!(after.revision.representation_text,formed.revision.representation_text);
    runtime.store.close().await;postgres.stop().await.unwrap();
}

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
#[expect(
    clippy::too_many_lines,
    reason = "integration scenario proves derivation, dense evidence visibility, and target isolation together"
)]
async fn derivation_claim_and_provider_commit_preserve_textual_surrogate() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, true, Some(Arc::new(TestEmbedder))).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let observed = runtime
        .material
        .record_observation(ObservationInput {
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
            capabilities: nous_core::CapabilityPolicy {
                text_embedding: nous_core::RequirementStrength::Required,
                ..Default::default()
            },
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
    assert!(
        result
            .results
            .iter()
            .any(|hit| { matches!(hit.reference, nous_core::CognitiveRef::DerivedRegion(_)) })
    );
    assert!(
        result
            .results
            .iter()
            .all(|hit| !matches!(hit.reference, nous_core::CognitiveRef::Memory(_)))
    );
    let memory_result = runtime
        .query(CognitiveQuery {
            api_version: API_VERSION,
            subject,
            session: None,
            situation: Default::default(),
            targets: vec![nous_core::QueryTarget::Memory],
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
        .expect("memory-only evidence query");
    assert!(memory_result.results.iter().all(|hit| !matches!(
        hit.reference,
        nous_core::CognitiveRef::DerivedRepresentation(_)
    )));

    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[tokio::test]
async fn stale_derivation_claim_cannot_publish_semantic_rows() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, true, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let observed = runtime
        .material
        .record_observation(ObservationInput {
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
                text: "document".into(),
                media_type: "application/pdf".into(),
            },
            entities: Vec::new(),
            runtime: RuntimeDirective::default(),
        })
        .await
        .expect("observation");
    let producer = ProducerSignature {
        signature_hash: "stale-document-producer-v1".into(),
        provider_class: "deterministic-library".into(),
        operation: CapabilityOperation::DocumentExtraction,
        implementation: "test-extractor".into(),
        model_identity: None,
        model_revision: None,
        preprocessing_identity: "identity".into(),
        preprocessing_revision: "1".into(),
        config_digest: "test".into(),
    };
    runtime
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
        .expect("schedule");
    let claims = runtime
        .material
        .claim_derivations(1, "worker-A", Duration::from_secs(60))
        .await
        .expect("claim A");
    assert_eq!(claims.len(), 1);
    let claim = &claims[0];
    sqlx::query("UPDATE derivation_attempts SET lease_until=now() - interval '1 second' WHERE attempt_id=$1")
        .bind(claim.attempt_id)
        .execute(runtime.store.pool())
        .await
        .expect("expire A");
    let stale_representation = nous_material::DerivedRepresentation {
        derived_representation_id: nous_core::DerivedRepresentationId::new(),
        subject_id: claim.subject_id,
        source_region_id: claim.source_region_id,
        representation_kind: RepresentationKind::ExtractedText,
        producer: producer.clone(),
        revision: 1,
        payload_text: Some("stale".into()),
        payload_artifact_id: None,
        quality: serde_json::json!({}),
        created_at: Utc::now(),
        supersedes: None,
    };
    let stale_error = runtime
        .material
        .commit_derivation_claim(claim, stale_representation, Vec::new())
        .await
        .expect_err("stale commit rejected");
    assert!(matches!(stale_error, nous_core::Error::Conflict(_)));
    let run = runtime
        .material
        .run_pending_derivations(1, "worker-B", Arc::new(TestExtractor { producer }))
        .await
        .expect("run B");
    assert_eq!(run.succeeded, 1);
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM derived_representations WHERE subject_id=$1 AND representation_kind=$2",
    )
    .bind(subject.0)
    .bind("extracted_text")
    .fetch_one(runtime.store.pool())
    .await
    .expect("count derived");
    assert_eq!(count, 1);
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
            derived_regions: vec![nous_material::DerivedRegion {
                derived_region_id: nous_core::DerivedRegionId::new(),
                subject_id: SubjectId::new(),
                derived_representation_id: nous_core::DerivedRepresentationId::new(),
                coordinate_kind: "text_span".into(),
                coordinate: serde_json::json!({"start":0,"end":8}),
                coordinate_hash: "test-derived-region".into(),
                parent_derived_region_id: None,
                created_at: Utc::now(),
            }],
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
        .timeout(Some(Duration::from_secs(30)))
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
#[expect(
    clippy::too_many_lines,
    reason = "integration scenario proves artifact identity and explicit use semantics"
)]
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
        runtime: RuntimeDirective::default(),
    };
    let first = runtime
        .material
        .record_observation(input.clone())
        .await
        .expect("first observation");
    let second = runtime
        .material
        .record_observation(input)
        .await
        .expect("second observation");
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
            }],
            entity_refs: Vec::new(),
            tags: Vec::new(),
            tag_order_provenance: None,
            valid_from: None,
            valid_to: None,
            epistemic_class: nous_core::EpistemicClass::Observed,
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
#[expect(
    clippy::too_many_lines,
    reason = "integration scenario proves rebuilt serving lanes and target isolation"
)]
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
        .material
        .record_observation(ObservationInput {
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
            }],
            entity_refs: vec![entity.clone()],
            tags: vec![tag.tag_id],
            tag_order_provenance: None,
            valid_from: None,
            valid_to: None,
            epistemic_class: nous_core::EpistemicClass::Reported,
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

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "integration scenario builds a bounded dense corpus and proves field-dense admission"
)]
async fn field_dense_only_candidate_enters_final_results() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, true, Some(Arc::new(TestEmbedder))).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let observed = runtime
        .material
        .record_observation(ObservationInput {
            subject,
            session: None,
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
                text: "witness document".into(),
                media_type: "text/plain".into(),
            },
            entities: Vec::new(),
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
                label: "cluster-zero".into(),
                description: None,
                kind_hint: None,
                origin: "explicit".into(),
            },
        )
        .await
        .expect("tag");
    let candidate = runtime
        .require_memory()
        .expect("memory")
        .form_memory(ExplicitMemoryInput {
            subject,
            memory_class: MemoryClass::Specific,
            semantic_role: "candidate".into(),
            representation_text: "topic alpha".into(),
            title: None,
            evidence: vec![MemoryRevisionEvidence {
                evidence_no: 0,
                evidence: EvidenceRef::Occurrence {
                    occurrence_id: observed.occurrence.occurrence_id,
                },
                support_role: SupportRole::Direct,
            }],
            entity_refs: Vec::new(),
            tags: vec![tag.tag_id],
            tag_order_provenance: None,
            valid_from: None,
            valid_to: None,
            epistemic_class: nous_core::EpistemicClass::Reported,
        })
        .await
        .expect("candidate memory");
    for _ in 0..120 {
        runtime
            .require_memory()
            .expect("memory")
            .form_memory(ExplicitMemoryInput {
                subject,
                memory_class: MemoryClass::Specific,
                semantic_role: "noise".into(),
                representation_text: "tag-1".into(),
                title: None,
                evidence: vec![MemoryRevisionEvidence {
                    evidence_no: 0,
                    evidence: EvidenceRef::Occurrence {
                        occurrence_id: observed.occurrence.occurrence_id,
                    },
                    support_role: SupportRole::Direct,
                }],
                entity_refs: Vec::new(),
                tags: Vec::new(),
                tag_order_provenance: None,
                valid_from: None,
                valid_to: None,
                epistemic_class: nous_core::EpistemicClass::Reported,
            })
            .await
            .expect("noise memory");
    }
    let result = runtime
        .query(CognitiveQuery {
            api_version: API_VERSION,
            subject,
            session: None,
            situation: Default::default(),
            targets: vec![nous_core::QueryTarget::Memory],
            cues: vec![
                Cue::Text(TextCue {
                    text: "tag-0".into(),
                }),
                Cue::Tag(TagCue { tag: tag.tag_id }),
            ],
            constraints: Default::default(),
            exploration: Default::default(),
            resources: Default::default(),
            result_need: nous_core::ResultNeed {
                limit: 32,
                need_evidence: true,
                need_materialization_handles: false,
            },
            effort: nous_core::CognitiveEffort::Light,
            capabilities: Default::default(),
            diagnostics: nous_core::DiagnosticsRequest::Full,
        })
        .await
        .expect("field dense query");
    let hit = result
        .results
        .iter()
        .find(|hit| hit.reference == CognitiveRef::Memory(candidate.object.memory_id))
        .expect("candidate entered final results");
    assert!(
        hit.match_evidence
            .variants
            .contains(&"local_field_dense".into())
            || hit
                .match_evidence
                .variants
                .contains(&"transfer_field_dense".into())
    );
    assert!(!hit.match_evidence.variants.contains(&"direct_dense".into()));
    assert!(
        result
            .diagnostics
            .as_ref()
            .is_some_and(|diagnostics| diagnostics.candidate_counts.get("field_dense") >= Some(&1))
    );
    let light_work = result
        .diagnostics
        .as_ref()
        .and_then(|diagnostics| {
            diagnostics
                .candidate_counts
                .get("executed_candidate_bound")
                .copied()
        })
        .expect("light work count");
    let maximum = runtime
        .query(CognitiveQuery {
            api_version: API_VERSION,
            subject,
            session: None,
            situation: Default::default(),
            targets: vec![nous_core::QueryTarget::Memory],
            cues: vec![
                Cue::Text(TextCue {
                    text: "tag-0".into(),
                }),
                Cue::Tag(TagCue { tag: tag.tag_id }),
            ],
            constraints: Default::default(),
            exploration: Default::default(),
            resources: Default::default(),
            result_need: nous_core::ResultNeed {
                limit: 32,
                need_evidence: true,
                need_materialization_handles: false,
            },
            effort: nous_core::CognitiveEffort::Maximum,
            capabilities: Default::default(),
            diagnostics: nous_core::DiagnosticsRequest::Full,
        })
        .await
        .expect("maximum query");
    let maximum_work = maximum
        .diagnostics
        .as_ref()
        .and_then(|diagnostics| {
            diagnostics
                .candidate_counts
                .get("executed_candidate_bound")
                .copied()
        })
        .expect("maximum work count");
    assert!(maximum_work > light_work);
    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "integration scenario proves topology-only proposal validation and atomic commit"
)]
async fn topology_only_consolidation_proposal_is_validated_and_atomic() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, true, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let observed = runtime
        .material
        .record_observation(ObservationInput {
            subject,
            session: None,
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
                text: "consolidation source".into(),
                media_type: "text/plain".into(),
            },
            entities: Vec::new(),
            runtime: RuntimeDirective::default(),
        })
        .await
        .expect("observation");
    let make_memory = |representation: &str| ExplicitMemoryInput {
        subject,
        memory_class: MemoryClass::Specific,
        semantic_role: "source".into(),
        representation_text: representation.into(),
        title: None,
        evidence: vec![MemoryRevisionEvidence {
            evidence_no: 0,
            evidence: EvidenceRef::Occurrence {
                occurrence_id: observed.occurrence.occurrence_id,
            },
            support_role: SupportRole::Direct,
        }],
        entity_refs: Vec::new(),
        tags: Vec::new(),
        tag_order_provenance: None,
        valid_from: None,
        valid_to: None,
        epistemic_class: nous_core::EpistemicClass::Observed,
    };
    let first = runtime
        .require_memory()
        .expect("memory")
        .form_memory(make_memory("first source"))
        .await
        .expect("first source");
    let second = runtime
        .require_memory()
        .expect("memory")
        .form_memory(make_memory("second source"))
        .await
        .expect("second source");
    let source_revisions = vec![
        first.revision.memory_revision_id,
        second.revision.memory_revision_id,
    ];
    let invalid = ConsolidationRequest {
        subject,
        source_memories: source_revisions.clone(),
        target: ConsolidationTarget::TopologyOnly,
        capability: nous_core::CapabilityRequirement {
            operation: CapabilityOperation::MemoryConsolidationText,
            strength: nous_core::RequirementStrength::Optional,
        },
        representation_text: None,
        semantic_role: None,
        topology: Some(TopologyConsolidationProposal {
            tags: Vec::new(),
            anchors: Vec::new(),
            associations: vec![TopologyAssociationProposal {
                from: CognitiveRef::Memory(nous_core::MemoryId::new()),
                to: CognitiveRef::MemoryRevision(second.revision.memory_revision_id),
                association_kind: "external".into(),
                polarity: AssociationPolarity::Positive,
                support_class: AssociationSupportClass::HostExplicit,
                support_value: 1.0,
                occurrence_id: None,
                memory_revision_id: None,
                bridge_hint: false,
            }],
            revisions: Vec::new(),
        }),
    };
    let invalid_error = runtime
        .require_memory()
        .expect("memory")
        .consolidate(subject, invalid)
        .await
        .expect_err("invalid cross-subject proposal");
    assert!(matches!(invalid_error, nous_core::Error::Invalid(_)));
    let invalid_counts: (i64, i64, i64) = (
        sqlx::query_scalar("SELECT count(*) FROM tags WHERE subject_id=$1")
            .bind(subject.0)
            .fetch_one(runtime.store.pool())
            .await
            .expect("tag count"),
        sqlx::query_scalar("SELECT count(*) FROM anchors WHERE subject_id=$1")
            .bind(subject.0)
            .fetch_one(runtime.store.pool())
            .await
            .expect("anchor count"),
        sqlx::query_scalar("SELECT count(*) FROM association_evidence WHERE subject_id=$1")
            .bind(subject.0)
            .fetch_one(runtime.store.pool())
            .await
            .expect("association count"),
    );
    assert_eq!(invalid_counts, (0, 0, 0));

    let valid = ConsolidationRequest {
        subject,
        source_memories: source_revisions.clone(),
        target: ConsolidationTarget::TopologyOnly,
        capability: nous_core::CapabilityRequirement {
            operation: CapabilityOperation::MemoryConsolidationText,
            strength: nous_core::RequirementStrength::Optional,
        },
        representation_text: None,
        semantic_role: None,
        topology: Some(TopologyConsolidationProposal {
            tags: vec![TopologyTagProposal {
                label: "topology-tag".into(),
                description: None,
                kind_hint: None,
                tag_id: None,
                attach_to: source_revisions.clone(),
            }],
            anchors: vec![TopologyAnchorProposal {
                label: Some("topology-anchor".into()),
                description: "consolidated anchor".into(),
                supports: vec![TopologyAnchorSupport {
                    reference: CognitiveRef::MemoryRevision(first.revision.memory_revision_id),
                    role: "support".into(),
                }],
                confirmed: true,
            }],
            associations: vec![TopologyAssociationProposal {
                from: CognitiveRef::MemoryRevision(first.revision.memory_revision_id),
                to: CognitiveRef::MemoryRevision(second.revision.memory_revision_id),
                association_kind: "experiential".into(),
                polarity: AssociationPolarity::Positive,
                support_class: AssociationSupportClass::HostExplicit,
                support_value: 1.0,
                occurrence_id: None,
                memory_revision_id: None,
                bridge_hint: false,
            }],
            revisions: vec![TopologyRevisionProposal {
                memory_id: first.object.memory_id,
                representation_text: "revised first source".into(),
                semantic_role: Some("revised".into()),
                title: None,
                evidence: first.evidence.clone(),
                expected_head_revision:first.object.head_revision,
                intent:nous_memory_domain::RevisionIntent::Reinterpret,
                entity_refs:first.entities.clone(),
                valid_from: None,
                valid_to: None,
                epistemic_class: nous_core::EpistemicClass::Observed,
            }],
        }),
    };
    let result = runtime
        .require_memory()
        .expect("memory")
        .consolidate(subject, valid)
        .await
        .expect("valid topology proposal");
    assert!(result.memory.is_none());
    assert_eq!(result.topology_changes, 6);
    let tag_count: i64 = sqlx::query_scalar("SELECT count(*) FROM tags WHERE subject_id=$1")
        .bind(subject.0)
        .fetch_one(runtime.store.pool())
        .await
        .expect("tag count");
    let anchor_count: i64 = sqlx::query_scalar("SELECT count(*) FROM anchors WHERE subject_id=$1")
        .bind(subject.0)
        .fetch_one(runtime.store.pool())
        .await
        .expect("anchor count");
    let association_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM association_evidence WHERE subject_id=$1")
            .bind(subject.0)
            .fetch_one(runtime.store.pool())
            .await
            .expect("association count");
    assert_eq!(tag_count, 1);
    assert_eq!(anchor_count, 1);
    assert_eq!(association_count, 1);
    let original = runtime
        .require_memory()
        .expect("memory")
        .memory(
            subject,
            first.object.memory_id,
            Some(first.revision.memory_revision_id),
        )
        .await
        .expect("original source remains readable");
    assert!(!original.evidence.is_empty());
    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "integration scenario proves derived cognition and CAS reachability retention"
)]
async fn purge_respects_reachability_for_derived_representations() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, true, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let observed = runtime
        .material
        .record_observation(ObservationInput {
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
                text: "shared derived source".into(),
                media_type: "application/pdf".into(),
            },
            entities: Vec::new(),
            runtime: RuntimeDirective::default(),
        })
        .await
        .expect("observation");
    let producer = ProducerSignature {
        signature_hash: "purge-derived-producer-v1".into(),
        provider_class: "deterministic-library".into(),
        operation: CapabilityOperation::DocumentExtraction,
        implementation: "test-extractor".into(),
        model_identity: None,
        model_revision: None,
        preprocessing_identity: "identity".into(),
        preprocessing_revision: "1".into(),
        config_digest: "test".into(),
    };
    runtime
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
        .expect("schedule");
    runtime
        .material
        .run_pending_derivations(1, "purge-worker", Arc::new(TestExtractor { producer }))
        .await
        .expect("derive");
    let derived_id: uuid::Uuid = sqlx::query_scalar(
        "SELECT derived_representation_id FROM derived_representations WHERE subject_id=$1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(subject.0)
    .fetch_one(runtime.store.pool())
    .await
    .expect("derived id");
    let make_memory = |text: &str| ExplicitMemoryInput {
        subject,
        memory_class: MemoryClass::Specific,
        semantic_role: "uses-derived".into(),
        representation_text: text.into(),
        title: None,
        evidence: vec![MemoryRevisionEvidence {
            evidence_no: 0,
            evidence: EvidenceRef::DerivedRepresentation {
                derived_representation_id: nous_core::DerivedRepresentationId(derived_id),
            },
            support_role: SupportRole::Direct,
        }],
        entity_refs: Vec::new(),
        tags: Vec::new(),
        tag_order_provenance: None,
        valid_from: None,
        valid_to: None,
        epistemic_class: nous_core::EpistemicClass::Observed,
    };
    let first = runtime
        .require_memory()
        .expect("memory")
        .form_memory(make_memory("first derived consumer"))
        .await
        .expect("first");
    let second = runtime
        .require_memory()
        .expect("memory")
        .form_memory(make_memory("second derived consumer"))
        .await
        .expect("second");
    runtime
        .require_memory()
        .expect("memory")
        .purge_memory(subject, first.object.memory_id, first.object.head_revision)
        .await
        .expect("purge first");
    let retained: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM derived_representations WHERE derived_representation_id=$1",
    )
    .bind(derived_id)
    .fetch_one(runtime.store.pool())
    .await
    .expect("derived retained count");
    assert_eq!(retained, 1);
    sqlx::query("UPDATE coverage_needs SET current_representation_id=NULL,state='missing' WHERE current_representation_id=$1")
        .bind(derived_id)
        .execute(runtime.store.pool())
        .await
        .expect("clear coverage");
    sqlx::query("UPDATE derivations SET successful_representation_id=NULL,state='pending' WHERE successful_representation_id=$1")
        .bind(derived_id)
        .execute(runtime.store.pool())
        .await
        .expect("clear derivation");
    runtime
        .require_memory()
        .expect("memory")
        .purge_memory(
            subject,
            second.object.memory_id,
            second.object.head_revision,
        )
        .await
        .expect("purge second");
    let removable: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM derived_representations WHERE derived_representation_id=$1",
    )
    .bind(derived_id)
    .fetch_one(runtime.store.pool())
    .await
    .expect("derived removed count");
    assert_eq!(removable, 0);
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
        .material
        .record_observation(ObservationInput {
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
        .material
        .record_observation(ObservationInput {
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
        .material
        .record_observation(ObservationInput {
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

#[tokio::test]
async fn resource_delete_is_subject_scoped() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, false, None).await;
    let subject_a = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject A")
        .subject_id;
    let subject_b = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject B")
        .subject_id;
    let resource = nous_core::ResourceRef::new("resource:shared:same").expect("resource");
    let input = ResourceUpsert {
        resource_ref: resource.clone(),
        display_label: Some("shared".into()),
        authority_class: "external".into(),
        coverage: serde_json::json!({}),
        query_dimensions: serde_json::json!({}),
        modalities: serde_json::json!([]),
        freshness_policy: serde_json::json!({}),
        access_cost_class: "low".into(),
        readiness: "ready".into(),
    };
    runtime
        .cognition
        .upsert_resource(subject_a, input.clone())
        .await
        .expect("upsert A");
    runtime
        .cognition
        .upsert_resource(subject_b, input)
        .await
        .expect("upsert B");
    let session_a = runtime
        .cognition
        .open_session(subject_a, serde_json::json!({}))
        .await
        .expect("session A")
        .session_id;
    let session_b = runtime
        .cognition
        .open_session(subject_b, serde_json::json!({}))
        .await
        .expect("session B")
        .session_id;
    runtime
        .cognition
        .admit(
            session_a,
            CognitiveRef::Resource(resource.clone()),
            "test",
            None,
        )
        .await
        .expect("admit A");
    runtime
        .cognition
        .admit(
            session_b,
            CognitiveRef::Resource(resource.clone()),
            "test",
            None,
        )
        .await
        .expect("admit B");

    runtime
        .cognition
        .delete_resource(subject_a, resource.clone())
        .await
        .expect("delete A");
    let view_a = runtime
        .cognition
        .session(subject_a, session_a)
        .await
        .expect("view A");
    let view_b = runtime
        .cognition
        .session(subject_b, session_b)
        .await
        .expect("view B");
    assert!(
        !view_a
            .resident
            .iter()
            .any(|item| item.reference == CognitiveRef::Resource(resource.clone()))
    );
    assert!(
        view_b
            .resident
            .iter()
            .any(|item| item.reference == CognitiveRef::Resource(resource.clone()))
    );
    assert_eq!(
        runtime
            .cognition
            .list_resources(subject_a)
            .await
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        runtime
            .cognition
            .list_resources(subject_b)
            .await
            .unwrap()
            .len(),
        1
    );
    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[tokio::test]
async fn resource_plan_returns_bounded_external_actions_without_callbacks() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, false, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let first = nous_core::ResourceRef::new("resource:test:first").expect("first");
    let second = nous_core::ResourceRef::new("resource:test:second").expect("second");
    for resource_ref in [first.clone(), second.clone()] {
        runtime
            .cognition
            .upsert_resource(
                subject,
                ResourceUpsert {
                    resource_ref,
                    display_label: Some("mock".into()),
                    authority_class: "external".into(),
                    coverage: serde_json::json!({}),
                    query_dimensions: serde_json::json!({}),
                    modalities: serde_json::json!([]),
                    freshness_policy: serde_json::json!({}),
                    access_cost_class: "low".into(),
                    readiness: "ready".into(),
                },
            )
            .await
            .expect("upsert resource");
    }
    let query = |synopsis_only: bool,
                 current: bool,
                 exploration: nous_core::ExplorationIntent,
                 effort: nous_core::CognitiveEffort| CognitiveQuery {
        api_version: API_VERSION,
        subject,
        session: None,
        situation: Default::default(),
        targets: vec![nous_core::QueryTarget::Resource],
        cues: Vec::new(),
        constraints: Default::default(),
        exploration,
        resources: nous_core::ResourceIntent {
            current_authority: if current {
                nous_core::CurrentAuthorityNeed::Prefer
            } else {
                nous_core::CurrentAuthorityNeed::None
            },
            synopsis_only,
        },
        result_need: Default::default(),
        effort,
        capabilities: Default::default(),
        diagnostics: Default::default(),
    };
    let synopsis = runtime
        .query(query(
            true,
            false,
            nous_core::ExplorationIntent::None,
            nous_core::CognitiveEffort::Light,
        ))
        .await
        .expect("synopsis query");
    assert_eq!(synopsis.resource_actions.len(), 1);
    assert_eq!(synopsis.resource_actions[0].action, "inspect_synopsis");
    assert!(!synopsis.resource_actions[0].current_authority);

    let global = runtime
        .query(query(
            false,
            false,
            nous_core::ExplorationIntent::Global,
            nous_core::CognitiveEffort::Maximum,
        ))
        .await
        .expect("global query");
    assert_eq!(global.resource_actions.len(), 2);
    let local = runtime
        .query(query(
            false,
            true,
            nous_core::ExplorationIntent::None,
            nous_core::CognitiveEffort::Light,
        ))
        .await
        .expect("local query");
    assert_eq!(local.resource_actions.len(), 1);
    assert_eq!(local.resource_actions[0].action, "query_current_authority");
    assert!(local.resource_actions[0].records.is_empty());
    let maximum_local = runtime
        .query(query(
            false,
            true,
            nous_core::ExplorationIntent::None,
            nous_core::CognitiveEffort::Maximum,
        ))
        .await
        .expect("maximum local query");
    assert_eq!(maximum_local.resource_actions.len(), 2);
    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[tokio::test]
async fn projection_invalidation_commits_and_rolls_back_atomically() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, false, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    let before: i64 = sqlx::query_scalar("SELECT state_revision FROM subjects WHERE subject_id=$1")
        .bind(subject.0)
        .fetch_one(runtime.store.pool())
        .await
        .expect("state before");
    let mut tx = runtime.store.begin().await.expect("tx");
    sqlx::query("UPDATE subjects SET metadata=$2 WHERE subject_id=$1")
        .bind(subject.0)
        .bind(serde_json::json!({"atomic":true}))
        .execute(&mut *tx)
        .await
        .expect("authority write");
    nous_authority_store::AuthorityStore::invalidate_in(
        &mut tx,
        subject,
        ProjectionInvalidation::text(),
    )
    .await
    .expect("invalidate in tx");
    tx.commit().await.expect("commit");
    let after_commit: i64 =
        sqlx::query_scalar("SELECT state_revision FROM subjects WHERE subject_id=$1")
            .bind(subject.0)
            .fetch_one(runtime.store.pool())
            .await
            .expect("state after");
    assert!(after_commit > before);
    assert!(
        runtime
            .store
            .projection_watermark(subject, "lexical", "")
            .await
            .expect("watermark")
            > 0
    );

    let mut tx = runtime.store.begin().await.expect("rollback tx");
    nous_authority_store::AuthorityStore::invalidate_in(
        &mut tx,
        subject,
        ProjectionInvalidation::text(),
    )
    .await
    .expect("invalidate rollback");
    tx.rollback().await.expect("rollback");
    let after_rollback: i64 =
        sqlx::query_scalar("SELECT state_revision FROM subjects WHERE subject_id=$1")
            .bind(subject.0)
            .fetch_one(runtime.store.pool())
            .await
            .expect("state after rollback");
    assert_eq!(after_rollback, after_commit);
    runtime.store.close().await;
    postgres.stop().await.expect("stop PostgreSQL");
}

#[tokio::test]
async fn selective_serving_preparation_does_not_build_unrelated_families() {
    let (postgres, url, root) = database().await;
    let runtime = open_runtime(&url, &root, false, None).await;
    let subject = runtime
        .subjects
        .create_subject(seed_input())
        .await
        .expect("subject")
        .subject_id;
    runtime
        .serving
        .prepare(
            subject,
            ServingNeed {
                exact: true,
                lexical: false,
                dense: false,
                topology: false,
            },
        )
        .await
        .expect("prepare exact");
    let after_exact = runtime
        .store
        .serving_current(subject)
        .await
        .expect("current exact");
    assert!(after_exact.iter().all(|record| record.family == "exact"));
    runtime
        .serving
        .prepare(
            subject,
            ServingNeed {
                exact: false,
                lexical: true,
                dense: false,
                topology: false,
            },
        )
        .await
        .expect("prepare lexical");
    let after_lexical = runtime
        .store
        .serving_current(subject)
        .await
        .expect("current lexical");
    assert!(
        after_lexical
            .iter()
            .all(|record| matches!(record.family.as_str(), "exact" | "lexical"))
    );
    runtime
        .serving
        .prepare(
            subject,
            ServingNeed {
                exact: false,
                lexical: false,
                dense: false,
                topology: true,
            },
        )
        .await
        .expect("prepare topology");
    let after_topology = runtime
        .store
        .serving_current(subject)
        .await
        .expect("current topology");
    assert!(
        after_topology
            .iter()
            .all(|record| matches!(record.family.as_str(), "exact" | "lexical" | "topology"))
    );
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
        accessibility_policy:Default::default(),
        stored_embedding: None,
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
