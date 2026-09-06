use futures::StreamExt;
use nous_core::{Error, Readiness};
use nous_material::{Classification, EpistemicClass, OriginClass};
use nous_memory_service::{
    LocalRuntime,
    material::UploadMetadata,
    subjects::{CreateSubject, SeedContent, SeedInput},
};
use uuid::Uuid;
mod support;

fn envelope(key: &str) -> UploadMetadata {
    UploadMetadata {
        api_version: 1,
        source_kind: "host:file".into(),
        classification: Classification {
            origin: OriginClass::Host,
            semantic: "test:binary".into(),
            epistemic: EpistemicClass::Observed,
        },
        media_type: "application/octet-stream".into(),
        scope: "stream".into(),
        occurred: None,
        observed_at: None,
        actor: None,
        invocation_id: None,
        idempotency_key: Some(key.into()),
        parent_sources: vec![],
        metadata: serde_json::json!({}),
    }
}

#[tokio::test]
async fn bounded_upload_roundtrip_and_absent_memory() {
    let (_postgres, url, _postgres_root) = support::database().await;
    let root = tempfile::tempdir().unwrap();
    let mut runtime = LocalRuntime::open_with_options(
        &url,
        4,
        root.path().join("objects").to_str().unwrap(),
        None,
        false,
        64 * 1024 * 1024,
    )
    .await
    .unwrap();
    let subject = runtime
        .create_subject(CreateSubject {
            api_version: 1,
            label: Some("流式材料".into()),
            metadata: serde_json::json!({}),
            memory_enabled: true,
            character_seed: SeedInput {
                content: SeedContent::Inline {
                    content: "独立的 Subject".into(),
                    media_type: "text/plain".into(),
                },
                authored_by: "host:test".into(),
            },
        })
        .await
        .unwrap()
        .subject_id;
    assert!(
        runtime
            .status()
            .await
            .capabilities
            .iter()
            .any(|c| c.capability_id == "memory" && c.status == Readiness::Unavailable)
    );
    assert!(matches!(
        runtime.memory(subject, Uuid::now_v7(), None).await,
        Err(Error::Unavailable(_))
    ));
    assert!(matches!(
        runtime.process_pending_for_subject(subject, 1).await,
        Err(Error::Unavailable(_))
    ));
    assert!(matches!(
        runtime.begin_cycle(subject, serde_json::json!({})).await,
        Err(Error::Unavailable(_))
    ));
    assert!(matches!(
        runtime.rebuild_projection(subject).await,
        Err(Error::Unavailable(_))
    ));

    // 64 MiB is generated, written and read one MiB at a time.
    let mut hasher = blake3::Hasher::new();
    let chunks = futures::stream::iter((0..64).map(|i| {
        let chunk = vec![i as u8; 1024 * 1024];
        hasher.update(&chunk);
        Ok(chunk)
    }));
    let accepted = runtime
        .ingest_stream(subject, envelope("large"), chunks)
        .await
        .unwrap();
    assert!(accepted.obligation_id.is_none());
    let expected_hash = hasher.finalize().to_hex().to_string();
    let (metadata, stream) = runtime
        .artifact_stream(subject, accepted.artifact_id)
        .await
        .unwrap();
    assert_eq!(metadata.byte_size, 64 * 1024 * 1024);
    assert_eq!(metadata.content_hash, expected_hash);
    let mut read_hash = blake3::Hasher::new();
    let mut size = 0;
    futures::pin_mut!(stream);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        assert!(chunk.len() <= 1024 * 1024);
        size += chunk.len();
        read_hash.update(&chunk);
    }
    assert_eq!(size, metadata.byte_size as usize);
    assert_eq!(read_hash.finalize().to_hex().as_str(), expected_hash);
    assert_eq!(
        runtime
            .source_show(subject, accepted.source_id)
            .await
            .unwrap()["artifact_ids"][0],
        serde_json::json!(accepted.artifact_id)
    );
    assert_eq!(
        runtime
            .artifact_lineage(subject, accepted.artifact_id)
            .await
            .unwrap()["source_ids"][0],
        serde_json::json!(accepted.source_id)
    );

    let replay = runtime
        .ingest_stream(subject, envelope("large"), futures::stream::empty())
        .await
        .unwrap();
    assert_eq!(replay.source_id, accepted.source_id);
    let before: i64 = sqlx::query_scalar("SELECT count(*) FROM source_records WHERE subject_id=$1")
        .bind(subject.0)
        .fetch_one(runtime.store.pool())
        .await
        .unwrap();
    runtime.max_upload_bytes = 1024;
    assert!(
        runtime
            .ingest_stream(
                subject,
                envelope("oversized"),
                futures::stream::iter([Ok(vec![0; 1025])])
            )
            .await
            .is_err()
    );
    assert!(
        runtime
            .ingest_stream(
                subject,
                envelope("interrupted"),
                futures::stream::iter([
                    Ok(vec![1; 100]),
                    Err(Error::Infrastructure("stream interrupted".into()))
                ])
            )
            .await
            .is_err()
    );
    let after: i64 = sqlx::query_scalar("SELECT count(*) FROM source_records WHERE subject_id=$1")
        .bind(subject.0)
        .fetch_one(runtime.store.pool())
        .await
        .unwrap();
    assert_eq!(
        before, after,
        "failed uploads cannot create canonical claims"
    );
    let artifact_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM artifacts WHERE subject_id=$1")
            .bind(subject.0)
            .fetch_one(runtime.store.pool())
            .await
            .unwrap();
    assert_eq!(artifact_count, 2, "only seed and successful upload exist");
    runtime.store.close().await;
}
