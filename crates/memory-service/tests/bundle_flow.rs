use nous_material::{Classification, EpistemicClass, OriginClass};
use nous_memory_domain::recall::{
    RecallCues, RecallEffort, RecallIntent, RecallObjective, ResultNeed,
};
use nous_memory_service::{
    LocalRuntime,
    bundle::ImportRequest,
    material::{IngestMaterial, MaterialContent},
    subjects::{CreateSubject, SeedContent, SeedInput},
};
use uuid::Uuid;
mod support;

#[tokio::test]
async fn bundle_round_trip_verifies_hashes_and_new_identity() {
    let (_postgres, url, _postgres_root) = support::database().await;
    let root = tempfile::tempdir().unwrap();
    let runtime = LocalRuntime::open(
        &url,
        4,
        root.path().join("objects").to_str().unwrap(),
        Some(root.path().join("lance").to_str().unwrap()),
    )
    .await
    .unwrap();
    let subject = runtime
        .create_subject(CreateSubject {
            api_version: 1,
            label: Some("bundle source".into()),
            metadata: serde_json::json!({"kind":"test"}),
            character_seed: SeedInput {
                content: SeedContent::Inline {
                    content: "Portable seed".into(),
                    media_type: "text/plain".into(),
                },
                authored_by: "test:author".into(),
            },
            memory_enabled: true,
        })
        .await
        .unwrap();
    runtime
        .ingest(
            subject.subject_id,
            IngestMaterial {
                api_version: 1,
                source_kind: "test:bundle".into(),
                classification: Classification {
                    origin: OriginClass::Human,
                    semantic: "DOCUMENT".into(),
                    epistemic: EpistemicClass::Observed,
                },
                media_type: "text/plain".into(),
                scope: "bundle".into(),
                occurred: None,
                observed_at: None,
                actor: None,
                invocation_id: None,
                idempotency_key: Some("bundle-source".into()),
                parent_sources: vec![],
                metadata: serde_json::json!({}),
                content: MaterialContent::Text {
                    text: "A portable bundle fact.".into(),
                },
            },
        )
        .await
        .unwrap();
    runtime.process_pending(10).await.unwrap();
    let bundle_path = root.path().join("bundle");
    let manifest = runtime
        .export_bundle(subject.subject_id, &bundle_path)
        .await
        .unwrap();
    assert_eq!(manifest.schema_version, 2);
    assert!(!manifest.object_hashes.is_empty());
    let imported = runtime
        .import_bundle(
            &bundle_path,
            ImportRequest {
                api_version: 1,
                preserve_identity: false,
                rebuild_projection: true,
            },
        )
        .await
        .unwrap();
    assert_ne!(imported.subject_id, subject.subject_id);
    assert!(imported.imported_artifacts >= manifest.object_hashes.len());
    assert_eq!(
        runtime
            .seed_history(imported.subject_id)
            .await
            .unwrap()
            .len(),
        1
    );
    let recalled = runtime
        .recall(
            imported.subject_id,
            RecallIntent {
                api_version: 1,
                cycle_id: None,
                target: vec![],
                objective: RecallObjective::Current,
                temporal: Default::default(),
                cues: RecallCues {
                    text: Some("portable bundle fact".into()),
                    ..Default::default()
                },
                constraints: Default::default(),
                result_need: ResultNeed {
                    limit: 1,
                    include_evidence: true,
                },
                effort: RecallEffort::Normal,
            },
        )
        .await
        .unwrap();
    assert_eq!(recalled.results.len(), 1);
    let bytes = tokio::fs::read(bundle_path.join("objects").join(&manifest.object_hashes[0]))
        .await
        .unwrap();
    tokio::fs::write(
        bundle_path.join("objects").join(&manifest.object_hashes[0]),
        [0_u8, 1, 2],
    )
    .await
    .unwrap();
    assert!(
        runtime
            .import_bundle(
                &bundle_path,
                ImportRequest {
                    api_version: 1,
                    preserve_identity: false,
                    rebuild_projection: false
                }
            )
            .await
            .is_err()
    );
    tokio::fs::write(
        bundle_path.join("objects").join(&manifest.object_hashes[0]),
        bytes,
    )
    .await
    .unwrap();
    runtime.store.close().await;
    let _ = Uuid::now_v7();
}
