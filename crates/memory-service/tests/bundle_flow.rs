use nous_material::{Classification, EpistemicClass, OriginClass};
use nous_memory_domain::recall::{
    RecallCues, RecallEffort, RecallIntent, RecallObjective, ResultNeed,
};
use nous_memory_domain::{RevisionRelation, learning::UseKind};
use nous_memory_service::{
    LocalRuntime,
    bundle::ImportRequest,
    cycles::UseFeedback,
    material::{IngestMaterial, MaterialContent},
    memory::CorrectMemory,
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
                metadata: serde_json::json!({"episode_key":"bundle-episode"}),
                content: MaterialContent::Text {
                    text: "A portable bundle fact.".into(),
                },
            },
        )
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
                idempotency_key: Some("bundle-source-2".into()),
                parent_sources: vec![],
                metadata: serde_json::json!({"episode_key":"bundle-episode"}),
                content: MaterialContent::Text {
                    text: "A second episode fact.".into(),
                },
            },
        )
        .await
        .unwrap();
    runtime.process_pending(10).await.unwrap();
    let object: Uuid = sqlx::query_scalar(
        "SELECT o.object_id FROM memory_objects o JOIN memory_current_heads h USING(object_id) JOIN memory_revisions r USING(revision_id) WHERE o.subject_id=$1 AND o.object_kind='REFERENCE' AND r.representation_text LIKE '%portable bundle fact%' LIMIT 1",
    )
    .bind(subject.subject_id.0)
    .fetch_one(runtime.store.pool())
    .await
    .unwrap();
    let cycle = runtime
        .begin_cycle(
            subject.subject_id,
            serde_json::json!({"reason":"bundle evidence"}),
        )
        .await
        .unwrap();
    runtime
        .feedback(
            subject.subject_id,
            cycle,
            UseFeedback {
                api_version: 1,
                event_id: Uuid::now_v7(),
                kind: UseKind::ReferencedOrActedOn,
                object_refs: vec![object],
                followed_from: None,
                causation_id: None,
            },
        )
        .await
        .unwrap();
    runtime
        .close_cycle(subject.subject_id, cycle, "resolved")
        .await
        .unwrap();
    let old = runtime
        .memory(subject.subject_id, object, None)
        .await
        .unwrap();
    let mut replacement = old.content.clone();
    replacement.text.push_str(" corrected");
    runtime
        .correct_memory(
            subject.subject_id,
            object,
            CorrectMemory {
                api_version: 1,
                expected_revision: old.revision_id,
                replacement,
                reason: "bundle correction".into(),
                authority_metadata: None,
                relation: RevisionRelation::Correction,
            },
        )
        .await
        .unwrap();
    runtime
        .suppress(subject.subject_id, object, true, "portable suppression")
        .await
        .unwrap();
    let bundle_path = root.path().join("bundle");
    let manifest = runtime
        .export_bundle(subject.subject_id, &bundle_path)
        .await
        .unwrap();
    assert_eq!(manifest.schema_version, 2);
    assert!(!manifest.object_hashes.is_empty());
    assert!(
        tokio::fs::try_exists(bundle_path.join("use-events.json"))
            .await
            .unwrap()
    );
    assert!(
        tokio::fs::try_exists(bundle_path.join("associations.json"))
            .await
            .unwrap()
    );
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
    let imported_reference: Uuid = sqlx::query_scalar(
        "SELECT o.object_id FROM memory_objects o JOIN memory_current_heads h USING(object_id) JOIN memory_revisions r USING(revision_id) WHERE o.subject_id=$1 AND o.object_kind='REFERENCE' AND r.representation_text LIKE '%portable bundle fact%' LIMIT 1",
    )
    .bind(imported.subject_id.0)
    .fetch_one(runtime.store.pool())
    .await
    .unwrap();
    let imported_view = runtime
        .memory(imported.subject_id, imported_reference, None)
        .await
        .unwrap();
    assert!(imported_view.suppressed);
    assert_eq!(
        runtime
            .memory_history(imported.subject_id, imported_reference)
            .await
            .unwrap()
            .len(),
        2
    );
    let imported_episode: Uuid = sqlx::query_scalar(
        "SELECT object_id FROM memory_objects WHERE subject_id=$1 AND object_kind='EPISODE' LIMIT 1",
    )
    .bind(imported.subject_id.0)
    .fetch_one(runtime.store.pool())
    .await
    .unwrap();
    let membership = runtime
        .episode_membership(imported.subject_id, imported_episode)
        .await
        .unwrap();
    assert!(
        membership["members"]
            .as_array()
            .is_some_and(|members| !members.is_empty())
    );
    let associations: i64 =
        sqlx::query_scalar("SELECT count(*) FROM association_evidence WHERE subject_id=$1")
            .bind(imported.subject_id.0)
            .fetch_one(runtime.store.pool())
            .await
            .unwrap();
    assert!(associations > 0);
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
    assert!(
        recalled.results.is_empty(),
        "suppression survives import and removes ordinary recall"
    );
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
