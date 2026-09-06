use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, Request, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use nous_core::{Error, SubjectId};
use nous_memory_service::{
    LocalRuntime,
    bundle::{ImportRequest, ImportResult},
    cycles::UseFeedback,
    material::{IngestMaterial, SubmitDerivation, ToolObservation, UploadMetadata},
    memory::CorrectMemory,
    operations::ConsolidateRequest,
    purge::PurgeRequest,
    recall::RecallResponse,
    subjects::{CreateSubject, ReplaceSeed},
};
use serde::Deserialize;
use uuid::Uuid;

pub fn routes() -> Router<LocalRuntime> {
    Router::new()
        .route("/v1/subjects", post(create).get(list))
        .route("/v1/subjects/{subject}", get(show))
        .route("/v1/subjects/{subject}/seed", post(replace))
        .route("/v1/subjects/{subject}/seed/history", get(history))
        .route("/v1/subjects/{subject}/material", post(ingest))
        .route(
            "/v1/subjects/{subject}/material/upload",
            post(upload).layer(axum::extract::DefaultBodyLimit::disable()),
        )
        .route(
            "/v1/subjects/{subject}/material/{source}/status",
            get(material_status),
        )
        .route("/v1/subjects/{subject}/artifacts/{artifact}", get(artifact))
        .route(
            "/v1/subjects/{subject}/artifacts/{artifact}/content",
            get(artifact_content),
        )
        .route(
            "/v1/subjects/{subject}/artifacts/{artifact}/lineage",
            get(artifact_lineage),
        )
        .route("/v1/subjects/{subject}/sources/{source}", get(source_show))
        .route("/v1/subjects/{subject}/derivations", post(derivation))
        .route(
            "/v1/subjects/{subject}/tool-observation",
            post(tool_observation),
        )
        .route("/v1/subjects/{subject}/recall", post(recall))
        .route("/v1/subjects/{subject}/cycles", post(begin_cycle))
        .route(
            "/v1/subjects/{subject}/cycles/{cycle}/inspect",
            post(inspect_cycle),
        )
        .route(
            "/v1/subjects/{subject}/cycles/{cycle}/feedback",
            post(feedback),
        )
        .route(
            "/v1/subjects/{subject}/cycles/{cycle}/close",
            post(close_cycle),
        )
        .route("/v1/subjects/{subject}/memory/{object}", get(memory))
        .route(
            "/v1/subjects/{subject}/memory/{object}/history",
            get(memory_history),
        )
        .route(
            "/v1/subjects/{subject}/memory/{object}/episode-membership",
            get(episode_membership),
        )
        .route(
            "/v1/subjects/{subject}/memory/{object}/associations",
            get(associations),
        )
        .route(
            "/v1/subjects/{subject}/memory/{object}/correct",
            post(correct_memory),
        )
        .route(
            "/v1/subjects/{subject}/memory/{object}/suppress",
            post(suppress),
        )
        .route(
            "/v1/subjects/{subject}/memory/consolidate",
            post(consolidate),
        )
        .route("/v1/subjects/{subject}/memory/purge", post(purge))
        .route(
            "/v1/subjects/{subject}/projection/rebuild",
            post(rebuild_projection),
        )
        .route("/v1/subjects/{subject}/process", post(process))
        .route("/v1/subjects/{subject}/bundle/export", post(export_bundle))
        .route("/v1/bundle/import", post(import_bundle))
}

struct Problem(Error);
impl From<Error> for Problem {
    fn from(error: Error) -> Self {
        Self(error)
    }
}
impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let (status, code, message) = match self.0 {
            Error::Invalid(message) => (StatusCode::BAD_REQUEST, "invalid_request", message),
            Error::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            Error::Conflict(message) => (StatusCode::CONFLICT, "revision_conflict", message),
            Error::Unavailable(message) => {
                (StatusCode::SERVICE_UNAVAILABLE, "unavailable", message)
            }
            Error::Infrastructure(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "dependency_failure",
                "A required dependency failed".into(),
            ),
        };
        (status, Json(serde_json::json!({"api_version": 1, "code": code, "message": message, "request_id": Uuid::now_v7()}))).into_response()
    }
}

async fn create(
    State(runtime): State<LocalRuntime>,
    Json(input): Json<CreateSubject>,
) -> Result<impl IntoResponse, Problem> {
    Ok((
        StatusCode::CREATED,
        Json(runtime.create_subject(input).await?),
    ))
}
async fn show(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(runtime.subject(SubjectId(subject)).await?))
}
#[derive(Deserialize)]
struct Page {
    after: Option<Uuid>,
    limit: Option<u32>,
}
async fn list(
    State(runtime): State<LocalRuntime>,
    Query(page): Query<Page>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime
            .subjects(page.after, page.limit.unwrap_or(100))
            .await?,
    ))
}
async fn replace(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<ReplaceSeed>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(runtime.replace_seed(SubjectId(subject), input).await?))
}
async fn history(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(runtime.seed_history(SubjectId(subject)).await?))
}

async fn ingest(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<IngestMaterial>,
) -> Result<impl IntoResponse, Problem> {
    Ok((
        StatusCode::ACCEPTED,
        Json(runtime.ingest(SubjectId(subject), input).await?),
    ))
}

async fn upload(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    request: Request,
) -> Result<impl IntoResponse, Problem> {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Problem(Error::Invalid("multipart content type required".into())))?;
    let boundary =
        multer::parse_boundary(content_type).map_err(|e| Problem(Error::Invalid(e.to_string())))?;
    let mut multipart = multer::Multipart::new(request.into_body().into_data_stream(), boundary);
    let mut metadata = multipart
        .next_field()
        .await
        .map_err(multipart_problem)?
        .ok_or_else(|| Problem(Error::Invalid("metadata part required".into())))?;
    if metadata.name() != Some("metadata") {
        return Err(Problem(Error::Invalid(
            "metadata must precede content".into(),
        )));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = metadata.chunk().await.map_err(multipart_problem)? {
        if bytes.len().saturating_add(chunk.len()) > 65536 {
            return Err(Problem(Error::Invalid("metadata exceeds 64 KiB".into())));
        }
        bytes.extend_from_slice(&chunk);
    }
    let input: UploadMetadata =
        serde_json::from_slice(&bytes).map_err(|e| Problem(Error::Invalid(e.to_string())))?;
    drop(metadata);
    let content = multipart
        .next_field()
        .await
        .map_err(multipart_problem)?
        .ok_or_else(|| Problem(Error::Invalid("content part required".into())))?;
    if content.name() != Some("content") {
        return Err(Problem(Error::Invalid(
            "exactly one content part required".into(),
        )));
    }
    let stream = futures::stream::try_unfold(
        (content, multipart),
        |(mut content, mut multipart)| async move {
            match content
                .chunk()
                .await
                .map_err(|e| Error::Invalid(e.to_string()))?
            {
                Some(bytes) => Ok(Some((bytes.to_vec(), (content, multipart)))),
                None => {
                    drop(content);
                    if multipart
                        .next_field()
                        .await
                        .map_err(|e| Error::Invalid(e.to_string()))?
                        .is_some()
                    {
                        return Err(Error::Invalid(
                            "upload accepts only one metadata and one content part".into(),
                        ));
                    }
                    Ok(None)
                }
            }
        },
    );
    let accepted = runtime
        .ingest_stream(SubjectId(subject), input, stream)
        .await?;
    Ok((StatusCode::ACCEPTED, Json(accepted)))
}
fn multipart_problem(error: multer::Error) -> Problem {
    Problem(Error::Invalid(format!("invalid multipart body: {error}")))
}
async fn tool_observation(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<ToolObservation>,
) -> Result<impl IntoResponse, Problem> {
    Ok((
        StatusCode::ACCEPTED,
        Json(
            runtime
                .record_tool_observation(SubjectId(subject), input)
                .await?,
        ),
    ))
}
async fn material_status(
    State(runtime): State<LocalRuntime>,
    Path((subject, source)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime.material_status(SubjectId(subject), source).await?,
    ))
}
async fn artifact(
    State(runtime): State<LocalRuntime>,
    Path((subject, artifact)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(runtime.artifact(SubjectId(subject), artifact).await?))
}
async fn source_show(
    State(runtime): State<LocalRuntime>,
    Path((subject, source)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(runtime.source_show(SubjectId(subject), source).await?))
}
async fn artifact_lineage(
    State(runtime): State<LocalRuntime>,
    Path((subject, artifact)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime
            .artifact_lineage(SubjectId(subject), artifact)
            .await?,
    ))
}
async fn artifact_content(
    State(runtime): State<LocalRuntime>,
    Path((subject, artifact)): Path<(Uuid, Uuid)>,
) -> Result<Response, Problem> {
    let (metadata, stream) = runtime
        .artifact_stream(SubjectId(subject), artifact)
        .await?;
    let mut response = Response::new(Body::from_stream(stream));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&metadata.media_type)
            .map_err(|error| Problem(Error::Infrastructure(error.to_string())))?,
    );
    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&metadata.byte_size.to_string())
            .map_err(|error| Problem(Error::Infrastructure(error.to_string())))?,
    );
    response.headers_mut().insert(
        header::ETAG,
        HeaderValue::from_str(&format!("\"{}\"", metadata.content_hash))
            .map_err(|error| Problem(Error::Infrastructure(error.to_string())))?,
    );
    Ok(response)
}
async fn derivation(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<SubmitDerivation>,
) -> Result<impl IntoResponse, Problem> {
    Ok((
        StatusCode::CREATED,
        Json(runtime.submit_derivation(SubjectId(subject), input).await?),
    ))
}
async fn recall(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<RecallResponse>, Problem> {
    Ok(Json(
        runtime
            .recall(SubjectId(subject), runtime.recall_input(input)?)
            .await?,
    ))
}
#[derive(Deserialize)]
struct CycleBegin {
    context: serde_json::Value,
}
async fn begin_cycle(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<CycleBegin>,
) -> Result<impl IntoResponse, Problem> {
    Ok((
        StatusCode::CREATED,
        Json(
            serde_json::json!({"api_version":1,"cycle_id":runtime.begin_cycle(SubjectId(subject),input.context).await?}),
        ),
    ))
}
#[derive(Deserialize)]
struct ObjectRefs {
    object_refs: Vec<Uuid>,
}
async fn inspect_cycle(
    State(runtime): State<LocalRuntime>,
    Path((subject, cycle)): Path<(Uuid, Uuid)>,
    Json(input): Json<ObjectRefs>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime
            .inspect_cycle(SubjectId(subject), cycle, &input.object_refs)
            .await?,
    ))
}
async fn feedback(
    State(runtime): State<LocalRuntime>,
    Path((subject, cycle)): Path<(Uuid, Uuid)>,
    Json(input): Json<UseFeedback>,
) -> Result<impl IntoResponse, Problem> {
    runtime.feedback(SubjectId(subject), cycle, input).await?;
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Deserialize)]
struct CloseCycle {
    outcome: String,
}
async fn close_cycle(
    State(runtime): State<LocalRuntime>,
    Path((subject, cycle)): Path<(Uuid, Uuid)>,
    Json(input): Json<CloseCycle>,
) -> Result<impl IntoResponse, Problem> {
    runtime
        .close_cycle(SubjectId(subject), cycle, &input.outcome)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Deserialize)]
struct MemoryQuery {
    revision: Option<Uuid>,
}
async fn memory(
    State(runtime): State<LocalRuntime>,
    Path((subject, object)): Path<(Uuid, Uuid)>,
    Query(query): Query<MemoryQuery>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime
            .memory(SubjectId(subject), object, query.revision)
            .await?,
    ))
}
async fn memory_history(
    State(runtime): State<LocalRuntime>,
    Path((subject, object)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime.memory_history(SubjectId(subject), object).await?,
    ))
}
async fn episode_membership(
    State(runtime): State<LocalRuntime>,
    Path((subject, object)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime
            .episode_membership(SubjectId(subject), object)
            .await?,
    ))
}
async fn associations(
    State(runtime): State<LocalRuntime>,
    Path((subject, object)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime
            .association_evidence(SubjectId(subject), object)
            .await?,
    ))
}
async fn correct_memory(
    State(runtime): State<LocalRuntime>,
    Path((subject, object)): Path<(Uuid, Uuid)>,
    Json(input): Json<CorrectMemory>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime
            .correct_memory(SubjectId(subject), object, input)
            .await?,
    ))
}
#[derive(Deserialize)]
struct SuppressInput {
    suppressed: bool,
    reason: String,
}
async fn suppress(
    State(runtime): State<LocalRuntime>,
    Path((subject, object)): Path<(Uuid, Uuid)>,
    Json(input): Json<SuppressInput>,
) -> Result<impl IntoResponse, Problem> {
    runtime
        .suppress(SubjectId(subject), object, input.suppressed, &input.reason)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn consolidate(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<ConsolidateRequest>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(runtime.consolidate(SubjectId(subject), input).await?))
}
async fn purge(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<PurgeRequest>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(runtime.purge(SubjectId(subject), input).await?))
}
async fn rebuild_projection(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        serde_json::json!({"rebuilt_rows":runtime.rebuild_projection(SubjectId(subject)).await?}),
    ))
}
async fn process(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(limit): Json<Option<usize>>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime
            .process_pending_for_subject(SubjectId(subject), limit.unwrap_or(16))
            .await?,
    ))
}
#[derive(Deserialize)]
struct BundlePath {
    path: String,
}
async fn export_bundle(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<BundlePath>,
) -> Result<impl IntoResponse, Problem> {
    Ok(Json(
        runtime
            .export_bundle(SubjectId(subject), input.path)
            .await?,
    ))
}
#[derive(Deserialize)]
struct BundleImport {
    path: String,
    request: ImportRequest,
}
async fn import_bundle(
    State(runtime): State<LocalRuntime>,
    Json(input): Json<BundleImport>,
) -> Result<Json<ImportResult>, Problem> {
    Ok(Json(
        runtime.import_bundle(input.path, input.request).await?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use nous_memory_service::subjects::{SeedContent, SeedInput};

    #[tokio::test]
    async fn http_upload_streams_and_rejects_malformed_envelopes() {
        let (_postgres, url, _postgres_root) = crate::support::database().await;
        let root = tempfile::tempdir().unwrap();
        let runtime = LocalRuntime::open_with_options(
            &url,
            4,
            root.path().join("objects").to_str().unwrap(),
            None,
            false,
            4 * 1024 * 1024,
        )
        .await
        .unwrap();
        let subject = runtime
            .create_subject(CreateSubject {
                api_version: 1,
                label: None,
                metadata: serde_json::json!({}),
                memory_enabled: true,
                character_seed: SeedInput {
                    content: SeedContent::Inline {
                        content: "seed".into(),
                        media_type: "text/plain".into(),
                    },
                    authored_by: "host:test".into(),
                },
            })
            .await
            .unwrap()
            .subject_id;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = routes().with_state(runtime.clone());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let client = reqwest::Client::new();
        let endpoint = format!("http://{address}/v1/subjects/{}/material/upload", subject.0);
        let metadata = serde_json::json!({"api_version":1,"source_kind":"host:file","classification":{"origin":"HOST","semantic":"test:file","epistemic":"OBSERVED"},"media_type":"application/octet-stream","scope":"test","metadata":{}});
        let header = format!(
            "--nw\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n{metadata}\r\n--nw\r\nContent-Disposition: form-data; name=\"content\"; filename=\"data.bin\"\r\nContent-Type: application/octet-stream\r\n\r\n"
        );
        let chunks = futures::stream::iter([Ok::<_, std::io::Error>(header.into_bytes())])
            .chain(futures::stream::iter(
                (0..4).map(|_| Ok(vec![91; 1024 * 1024])),
            ))
            .chain(futures::stream::iter([Ok(b"\r\n--nw--\r\n".to_vec())]));
        let response = client
            .post(&endpoint)
            .header("content-type", "multipart/form-data; boundary=nw")
            .body(reqwest::Body::wrap_stream(chunks))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        let accepted: serde_json::Value = response.json().await.unwrap();
        let artifact = accepted["artifact_id"].as_str().unwrap();
        let mut response = client
            .get(format!(
                "http://{address}/v1/subjects/{}/artifacts/{artifact}/content",
                subject.0
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-length"], "4194304");
        assert_eq!(
            response.headers()["content-type"],
            "application/octet-stream"
        );
        assert!(response.headers().contains_key("etag"));
        let mut size = 0;
        while let Some(chunk) = response.chunk().await.unwrap() {
            assert!(chunk.iter().all(|b| *b == 91));
            size += chunk.len();
        }
        assert_eq!(size, 4 * 1024 * 1024);
        let source_count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM source_records WHERE subject_id=$1")
                .bind(subject.0)
                .fetch_one(runtime.store.pool())
                .await
                .unwrap();
        let malformed = format!(
            "--nw\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n{metadata}\r\n--nw\r\nContent-Disposition: form-data; name=\"content\"\r\n\r\nsecret\r\n--nw\r\nContent-Disposition: form-data; name=\"extra\"\r\n\r\ninvalid\r\n--nw--\r\n"
        );
        let response = client
            .post(&endpoint)
            .header("content-type", "multipart/form-data; boundary=nw")
            .body(malformed)
            .send()
            .await
            .unwrap();
        assert!(response.status().is_client_error());
        // No source ID is returned, and malformed payloads never reach canonical commit.
        let count_after: i64 =
            sqlx::query_scalar("SELECT count(*) FROM source_records WHERE subject_id=$1")
                .bind(subject.0)
                .fetch_one(runtime.store.pool())
                .await
                .unwrap();
        assert_eq!(count_after, source_count);
        let invalid = client
            .post(&endpoint)
            .header("content-type", "multipart/form-data; boundary=nw")
            .body(
                "--nw\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n".to_owned()
                    + &"x".repeat(65537)
                    + "\r\n--nw--\r\n",
            )
            .send()
            .await
            .unwrap();
        assert!(invalid.status().is_client_error());
        server.abort();
        runtime.store.close().await;
    }
}
