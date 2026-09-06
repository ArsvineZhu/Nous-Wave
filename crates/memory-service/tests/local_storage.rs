use nous_memory_service::LocalRuntime;

/// Use a disposable PostgreSQL 18 database. This test installs the real schema.
#[tokio::test]
#[ignore = "requires NOUS_WAVE_TEST_DATABASE_URL pointing at a disposable PostgreSQL 18 database"]
async fn migrations_cas_projection_and_restart() {
    let url =
        std::env::var("NOUS_WAVE_TEST_DATABASE_URL").expect("explicit disposable database URL");
    let directory = tempfile::tempdir().unwrap();
    let objects = directory.path().join("objects");
    let index = directory.path().join("lancedb");
    let runtime = LocalRuntime::open(
        &url,
        2,
        objects.to_str().unwrap(),
        Some(index.to_str().unwrap()),
    )
    .await
    .unwrap();
    assert!(runtime.projection.is_some(), "real LanceDB must open");
    use nous_memory_retrieval::dense::{VectorRow, model_table};
    let vector_subject = nous_core::SubjectId::default();
    let vector_revision = uuid::Uuid::now_v7();
    let table_name = model_table("test:numeric-vectors", "1", "identity", 3);
    let vector = VectorRow {
        subject: vector_subject,
        object: uuid::Uuid::now_v7(),
        revision: vector_revision,
        vector: vec![1.0, 0.0, 0.0],
    };
    let projection = runtime.projection.as_ref().unwrap();
    projection
        .upsert(&table_name, std::slice::from_ref(&vector))
        .await
        .unwrap();
    projection
        .upsert(&table_name, std::slice::from_ref(&vector))
        .await
        .unwrap();
    assert_eq!(
        projection
            .search(&table_name, vector_subject, &[1.0, 0.0, 0.0], 10)
            .await
            .unwrap(),
        vec![vector_revision]
    );
    assert!(
        projection
            .search(
                &table_name,
                nous_core::SubjectId::default(),
                &[1.0, 0.0, 0.0],
                10
            )
            .await
            .is_err()
    );
    projection.remove_subject(vector_subject).await.unwrap();
    assert!(
        projection
            .search(&table_name, vector_subject, &[1.0, 0.0, 0.0], 10)
            .await
            .is_err()
    );
    projection.upsert(&table_name, &[vector]).await.unwrap();
    let payload = b"source bytes: \x00\xff\x01".to_vec();
    let hash = runtime.objects.put(payload.clone()).await.unwrap();
    assert_eq!(runtime.objects.get(&hash).await.unwrap(), payload);
    assert_eq!(runtime.objects.put(payload.clone()).await.unwrap(), hash);
    let table: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('character_seed_revisions')::text")
            .fetch_one(runtime.store.pool())
            .await
            .unwrap();
    assert!(table.is_some());
    runtime.store.close().await;
    drop(runtime);

    let restarted = LocalRuntime::open(
        &url,
        2,
        objects.to_str().unwrap(),
        Some(index.to_str().unwrap()),
    )
    .await
    .unwrap();
    assert!(restarted.projection.is_some());
    assert_eq!(restarted.objects.get(&hash).await.unwrap(), payload);
    restarted.store.close().await;
    drop(restarted);

    let minimal = LocalRuntime::open(&url, 2, objects.to_str().unwrap(), None)
        .await
        .unwrap();
    assert!(minimal.projection.is_none());
    assert_eq!(minimal.objects.get(&hash).await.unwrap(), payload);
    use nous_memory_service::subjects::{CreateSubject, ReplaceSeed, SeedContent, SeedInput};
    let seed = SeedInput {
        content: SeedContent::Inline {
            content: "# Character\nCurious, careful subject.".into(),
            media_type: "text/markdown".into(),
        },
        authored_by: "test:author".into(),
    };
    let subject = minimal
        .create_subject(CreateSubject {
            api_version: 1,
            label: Some("storage integration".into()),
            metadata: serde_json::json!({}),
            character_seed: seed.clone(),
            memory_enabled: false,
        })
        .await
        .unwrap();
    assert_eq!(subject.revision, 1);
    assert!(!subject.memory_enabled);
    let changed = minimal
        .replace_seed(
            subject.subject_id,
            ReplaceSeed {
                api_version: 1,
                expected_subject_revision: 1,
                character_seed: SeedInput {
                    content: SeedContent::Inline {
                        content: "# Revised authored seed".into(),
                        media_type: "text/markdown".into(),
                    },
                    ..seed.clone()
                },
            },
        )
        .await
        .unwrap();
    assert_eq!(changed.revision, 2);
    let stale = minimal
        .replace_seed(
            subject.subject_id,
            ReplaceSeed {
                api_version: 1,
                expected_subject_revision: 1,
                character_seed: seed,
            },
        )
        .await;
    assert!(matches!(stale, Err(nous_core::Error::Conflict(_))));
    let history = minimal.seed_history(subject.subject_id).await.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].parent_revision, Some(history[0].revision_id));
    assert_eq!(
        minimal
            .artifact_bytes(subject.subject_id, history[0].artifact_id)
            .await
            .unwrap(),
        b"# Character\nCurious, careful subject."
    );

    use nous_material::{Classification, EpistemicClass, OriginClass, TemporalRange};
    use nous_memory_service::material::{IngestMaterial, MaterialContent};
    let source = IngestMaterial {
        api_version: 1,
        source_kind: "test:document".into(),
        classification: Classification {
            origin: OriginClass::Human,
            semantic: "DOCUMENT".into(),
            epistemic: EpistemicClass::Reported,
        },
        media_type: "text/markdown".into(),
        scope: "test".into(),
        occurred: Some(TemporalRange {
            start: None,
            end: Some(chrono::Utc::now()),
            approximate: true,
        }),
        observed_at: None,
        actor: None,
        invocation_id: None,
        idempotency_key: Some("one-source".into()),
        parent_sources: vec![],
        metadata: serde_json::json!({}),
        content: MaterialContent::Text {
            text: "Long heterogeneous material.\n".repeat(10000),
        },
    };
    let accepted = minimal
        .ingest(subject.subject_id, source.clone())
        .await
        .unwrap();
    let repeated = minimal.ingest(subject.subject_id, source).await.unwrap();
    assert_eq!(accepted.source_id, repeated.source_id);
    assert_eq!(accepted.status, "recorded");
    assert!(accepted.obligation_id.is_none());
    assert_eq!(
        minimal
            .artifact(subject.subject_id, accepted.artifact_id)
            .await
            .unwrap()
            .classification
            .epistemic,
        EpistemicClass::Reported
    );
    assert_eq!(
        minimal
            .artifact_bytes(subject.subject_id, accepted.artifact_id)
            .await
            .unwrap()
            .len(),
        "Long heterogeneous material.\n".len() * 10000
    );
    minimal.store.close().await;
}
