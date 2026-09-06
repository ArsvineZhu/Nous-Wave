use nous_core::SubjectId;
use nous_material::{Classification, EpistemicClass, OriginClass};
use nous_memory_domain::{RevisionRelation, learning::UseKind, recall::*};
use nous_memory_service::{
    LocalRuntime,
    cycles::UseFeedback,
    material::{IngestMaterial, MaterialContent},
    memory::CorrectMemory,
    subjects::{CreateSubject, SeedContent, SeedInput},
};
use uuid::Uuid;

fn intent(target: Option<Uuid>, text: Option<&str>, cycle: Option<Uuid>) -> RecallIntent {
    RecallIntent {
        api_version: 1,
        cycle_id: cycle,
        target: target
            .into_iter()
            .map(|id| NodeRef {
                kind: NodeKind::Memory,
                id,
            })
            .collect(),
        objective: RecallObjective::Current,
        temporal: Default::default(),
        cues: RecallCues {
            text: text.map(str::to_owned),
            ..Default::default()
        },
        constraints: Default::default(),
        result_need: ResultNeed {
            limit: 1,
            include_evidence: true,
        },
        effort: RecallEffort::Light,
    }
}
async fn ingest(runtime: &LocalRuntime, subject: SubjectId, text: &str, key: &str) -> Uuid {
    runtime
        .ingest(
            subject,
            IngestMaterial {
                api_version: 1,
                source_kind: "test:experience".into(),
                classification: Classification {
                    origin: OriginClass::Tool,
                    semantic: "TOOL_OBSERVATION".into(),
                    epistemic: EpistemicClass::Reported,
                },
                media_type: "text/plain".into(),
                scope: "private-test".into(),
                occurred: None,
                observed_at: None,
                actor: Some("test:instrument".into()),
                invocation_id: Some(key.into()),
                idempotency_key: Some(key.into()),
                parent_sources: vec![],
                metadata: serde_json::json!({"episode_key":"private-trip"}),
                content: MaterialContent::Text { text: text.into() },
            },
        )
        .await
        .unwrap()
        .source_id
}

#[tokio::test]
#[ignore = "requires NOUS_WAVE_TEST_DATABASE_URL pointing at disposable PostgreSQL 18"]
async fn memory_formation_recall_learning_history_and_cycle() {
    let directory = tempfile::tempdir().unwrap();
    let runtime = LocalRuntime::open(
        &std::env::var("NOUS_WAVE_TEST_DATABASE_URL").expect("disposable database"),
        4,
        directory.path().join("objects").to_str().unwrap(),
        Some(directory.path().join("lance").to_str().unwrap()),
    )
    .await
    .unwrap();
    let subject = runtime
        .create_subject(CreateSubject {
            api_version: 1,
            label: Some("memory flow".into()),
            metadata: serde_json::json!({}),
            character_seed: SeedInput {
                content: SeedContent::Inline {
                    content: "A careful research subject.".into(),
                    media_type: "text/plain".into(),
                },
                authored_by: "test:author".into(),
            },
            memory_enabled: true,
        })
        .await
        .unwrap()
        .subject_id;
    let source = ingest(
        &runtime,
        subject,
        "Zorvella reported a violet instrument near Qindru.",
        "one",
    )
    .await;
    ingest(
        &runtime,
        subject,
        "At Qindru the private object was named Meltrax.",
        "two",
    )
    .await;
    let processed = runtime.process_pending(10).await.unwrap();
    assert!(processed.succeeded >= 2);
    let first = runtime
        .recall(subject, intent(None, Some("Zorvella"), None))
        .await
        .unwrap();
    assert_eq!(first.results.len(), 1);
    let object = first.results[0].memory.object_id;
    assert_eq!(
        first.results[0].memory.content.classification.epistemic,
        EpistemicClass::Reported
    );
    assert!(
        first.results[0]
            .memory
            .content
            .source_refs
            .contains(&source)
    );
    let cycle = runtime
        .begin_cycle(subject, serde_json::json!({"reason":"find private target"}))
        .await
        .unwrap();
    let again = runtime
        .recall(subject, intent(Some(object), None, Some(cycle)))
        .await
        .unwrap();
    assert_eq!(again.effort.candidate_channels, vec!["exact_reference"]);
    let reused = runtime
        .recall(subject, intent(Some(object), None, Some(cycle)))
        .await
        .unwrap();
    assert_eq!(reused.effort.index_queries, 0);
    for _ in 0..10 {
        runtime
            .recall(subject, intent(Some(object), None, Some(cycle)))
            .await
            .unwrap();
    }
    let uses: i64 =
        sqlx::query_scalar("SELECT meaningful_uses FROM accessibility_state WHERE object_id=$1")
            .bind(object)
            .fetch_one(runtime.store.pool())
            .await
            .unwrap();
    assert_eq!(uses, 0);
    let feedback = UseFeedback {
        api_version: 1,
        event_id: Uuid::now_v7(),
        kind: UseKind::ReferencedOrActedOn,
        object_refs: vec![object],
        followed_from: None,
        causation_id: None,
    };
    runtime
        .feedback(subject, cycle, feedback.clone())
        .await
        .unwrap();
    runtime.feedback(subject, cycle, feedback).await.unwrap();
    let uses: i64 =
        sqlx::query_scalar("SELECT meaningful_uses FROM accessibility_state WHERE object_id=$1")
            .bind(object)
            .fetch_one(runtime.store.pool())
            .await
            .unwrap();
    assert_eq!(uses, 1);
    let old = runtime.memory(subject, object, None).await.unwrap();
    let mut replacement = old.content.clone();
    replacement.text = "The instrument was blue; the earlier tool report was wrong.".into();
    let corrected = runtime
        .correct_memory(
            subject,
            object,
            CorrectMemory {
                api_version: 1,
                expected_revision: old.revision_id,
                replacement,
                reason: "explicit correction".into(),
                authority_metadata: Some(serde_json::json!({"author":"test:operator"})),
                relation: RevisionRelation::Correction,
            },
        )
        .await
        .unwrap();
    assert_ne!(corrected.revision_id, old.revision_id);
    assert_eq!(
        runtime.memory_history(subject, object).await.unwrap().len(),
        2
    );
    assert_eq!(
        runtime
            .recall(subject, intent(Some(object), None, None))
            .await
            .unwrap()
            .results[0]
            .memory
            .revision_id,
        corrected.revision_id
    );
    runtime
        .suppress(subject, object, true, "operator exclusion")
        .await
        .unwrap();
    assert!(
        runtime
            .recall(subject, intent(Some(object), None, None))
            .await
            .unwrap()
            .results
            .is_empty()
    );
    assert!(
        runtime
            .memory(subject, object, None)
            .await
            .unwrap()
            .suppressed
    );
    runtime
        .suppress(subject, object, false, "operator restoration")
        .await
        .unwrap();
    runtime
        .close_cycle(subject, cycle, "resolved")
        .await
        .unwrap();
    assert!(
        runtime
            .recall(subject, intent(Some(object), None, Some(cycle)))
            .await
            .is_err()
    );
    let episode: Uuid = sqlx::query_scalar(
        "SELECT object_id FROM memory_objects WHERE subject_id=$1 AND object_kind='EPISODE'",
    )
    .bind(subject.0)
    .fetch_one(runtime.store.pool())
    .await
    .unwrap();
    use nous_memory_service::purge::{PurgeRequest, PurgeTarget};
    let purge = PurgeRequest {
        api_version: 1,
        operation_id: Uuid::now_v7(),
        target: PurgeTarget::Source(source),
    };
    let purged = runtime.purge(subject, purge.clone()).await.unwrap();
    assert_eq!(purged.state, "completed");
    assert!(purged.regenerating_objects.contains(&episode));
    assert!(runtime.memory(subject, object, None).await.is_err());
    assert_eq!(
        runtime.purge(subject, purge).await.unwrap().operation_id,
        purged.operation_id
    );
    runtime.process_pending(20).await.unwrap();
    let reconstructed = runtime.memory(subject, episode, None).await.unwrap();
    assert!(!reconstructed.content.source_refs.contains(&source));
    assert!(!reconstructed.content.text.contains("Zorvella"));
    assert_eq!(reconstructed.object_id, episode);
    runtime.rebuild_projection(subject).await.unwrap();
    assert!(
        runtime
            .recall(subject, intent(Some(object), None, None))
            .await
            .unwrap()
            .results
            .is_empty()
    );
    runtime
        .purge(
            subject,
            PurgeRequest {
                api_version: 1,
                operation_id: Uuid::now_v7(),
                target: PurgeTarget::Subject,
            },
        )
        .await
        .unwrap();
    assert!(runtime.subject(subject).await.is_err());
    runtime.store.close().await;
}
