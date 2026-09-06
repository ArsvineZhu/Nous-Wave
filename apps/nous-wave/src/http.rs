use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use nous_core::{Error, SubjectId};
use nous_memory_domain::recall::RecallIntent;
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
        .route("/v1/subjects/{subject}/material/upload", post(upload))
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
    mut multipart: Multipart,
) -> Result<impl IntoResponse, Problem> {
    let mut metadata: Option<UploadMetadata> = None;
    let mut accepted = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|error| Problem(Error::Invalid(format!("invalid multipart body: {error}"))))?
    {
        let name = field
            .name()
            .ok_or_else(|| Problem(Error::Invalid("multipart parts need names".into())))?
            .to_owned();
        match name.as_str() {
            "metadata" => {
                if metadata.is_some() {
                    return Err(Problem(Error::Invalid(
                        "multipart metadata part must occur once".into(),
                    )));
                }
                let mut bytes = Vec::new();
                while let Some(chunk) = field.chunk().await.map_err(|error| {
                    Problem(Error::Invalid(format!("metadata part failed: {error}")))
                })? {
                    if bytes.len().saturating_add(chunk.len()) > 64 * 1024 {
                        return Err(Problem(Error::Invalid(
                            "multipart metadata exceeds 64 KiB".into(),
                        )));
                    }
                    bytes.extend_from_slice(&chunk);
                }
                metadata = Some(serde_json::from_slice(&bytes).map_err(|error| {
                    Problem(Error::Invalid(format!("invalid upload metadata: {error}")))
                })?);
            }
            "content" => {
                if accepted.is_some() {
                    return Err(Problem(Error::Invalid(
                        "multipart content part must occur once".into(),
                    )));
                }
                let metadata = metadata.clone().ok_or_else(|| {
                    Problem(Error::Invalid(
                        "multipart metadata must precede content".into(),
                    ))
                })?;
                let stream = futures::stream::try_unfold(field, |mut field| async move {
                    match field.chunk().await {
                        Ok(Some(bytes)) => Ok(Some((bytes.to_vec(), field))),
                        Ok(None) => Ok(None),
                        Err(error) => Err(Error::Invalid(format!("content part failed: {error}"))),
                    }
                });
                accepted = Some(
                    runtime
                        .ingest_stream(SubjectId(subject), metadata, stream)
                        .await
                        .map_err(Problem)?,
                );
            }
            _ => {
                return Err(Problem(Error::Invalid(
                    "multipart accepts only metadata and content parts".into(),
                )));
            }
        }
    }
    let accepted = accepted.ok_or_else(|| {
        Problem(Error::Invalid(
            "multipart upload requires metadata and content parts".into(),
        ))
    })?;
    Ok((StatusCode::ACCEPTED, Json(accepted)))
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
    let metadata = runtime.artifact(SubjectId(subject), artifact).await?;
    let bytes = runtime.artifact_bytes(SubjectId(subject), artifact).await?;
    let mut response = Response::new(Body::from(bytes));
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
    Json(input): Json<RecallIntent>,
) -> Result<Json<RecallResponse>, Problem> {
    Ok(Json(runtime.recall(SubjectId(subject), input).await?))
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
