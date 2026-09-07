use axum::{
    Json, Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use futures::{StreamExt, stream};
use nous_core::{CognitiveQuery, Error, MemoryId, SubjectId};
use nous_material::ObservationInput;
use nous_memory_domain::ExplicitMemoryInput;
use nous_memory_service::{
    CreateSubject, LocalRuntime, ResourceUpsert, ReviseMemoryInput, UploadMetadata, UseFeedback,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

pub fn routes() -> Router<LocalRuntime> {
    Router::new()
        .route("/v1/subjects", post(create_subject))
        .route("/v1/subjects/{subject_id}/sessions", post(open_session))
        .route(
            "/v1/subjects/{subject_id}/sessions/{session_id}",
            get(show_session),
        )
        .route(
            "/v1/subjects/{subject_id}/sessions/{session_id}/close",
            post(close_session),
        )
        .route("/v1/subjects/{subject_id}/artifacts", post(upload_artifact))
        .route(
            "/v1/subjects/{subject_id}/artifacts/{artifact_id}",
            get(show_artifact),
        )
        .route(
            "/v1/subjects/{subject_id}/artifacts/{artifact_id}/content",
            get(artifact_content),
        )
        .route("/v1/subjects/{subject_id}/observations", post(observe))
        .route("/v1/subjects/{subject_id}/materialize", post(materialize))
        .route("/v1/subjects/{subject_id}/memories/form", post(form_memory))
        .route(
            "/v1/subjects/{subject_id}/memories/consolidate",
            post(consolidate),
        )
        .route(
            "/v1/subjects/{subject_id}/memories/{memory_id}",
            get(show_memory).delete(purge_memory),
        )
        .route(
            "/v1/subjects/{subject_id}/memories/{memory_id}/revisions",
            get(memory_history),
        )
        .route(
            "/v1/subjects/{subject_id}/memories/{memory_id}/revise",
            post(revise_memory),
        )
        .route(
            "/v1/subjects/{subject_id}/memories/{memory_id}/suppress",
            post(suppress_memory),
        )
        .route(
            "/v1/subjects/{subject_id}/memories/{memory_id}/restore",
            post(restore_memory),
        )
        .route("/v1/subjects/{subject_id}/query", post(query))
        .route("/v1/subjects/{subject_id}/use", post(use_feedback))
        .route("/v1/subjects/{subject_id}/resources", get(list_resources))
        .route(
            "/v1/subjects/{subject_id}/resources/{resource_ref}",
            put(upsert_resource).delete(delete_resource),
        )
        .route(
            "/v1/subjects/{subject_id}/projection/rebuild",
            post(rebuild_projection),
        )
        .route("/v1/subjects/{subject_id}/tags", post(create_tag))
        .route("/v1/subjects/{subject_id}/anchors", post(create_anchor))
        .route(
            "/v1/subjects/{subject_id}/associations",
            post(create_association),
        )
        .route(
            "/v1/subjects/{subject_id}/entity-bindings/rebind",
            post(rebind_entity),
        )
}

#[derive(Debug, serde::Serialize)]
struct Problem {
    error: String,
}

type Result<T, E = Problem> = std::result::Result<T, E>;

impl From<Error> for Problem {
    fn from(error: Error) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = if self.error.contains("not found") {
            StatusCode::NOT_FOUND
        } else if self.error.contains("unavailable") {
            StatusCode::SERVICE_UNAVAILABLE
        } else if self.error.contains("invalid") || self.error.contains("required") {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, Json(self)).into_response()
    }
}

async fn create_subject(
    State(runtime): State<LocalRuntime>,
    Json(input): Json<CreateSubject>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime.create_subject(input).await.map_err(Problem::from)?,
    ))
}

async fn open_session(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(metadata): Json<Value>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .open_session(SubjectId(subject), metadata)
            .await
            .map_err(Problem::from)?,
    ))
}

async fn show_session(
    State(runtime): State<LocalRuntime>,
    Path((subject, session)): Path<(Uuid, Uuid)>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .session(SubjectId(subject), nous_core::SessionId(session))
            .await
            .map_err(Problem::from)?,
    ))
}

async fn close_session(
    State(runtime): State<LocalRuntime>,
    Path((subject, session)): Path<(Uuid, Uuid)>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .close_session(SubjectId(subject), nous_core::SessionId(session))
            .await
            .map_err(Problem::from)?,
    ))
}

async fn upload_artifact(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<impl serde::Serialize>, Problem> {
    let mut metadata = UploadMetadata {
        media_type: "application/octet-stream".into(),
        metadata: serde_json::json!({}),
        source_class: nous_core::SourceClass::File,
        external_object_ref: None,
        occurred_at: None,
        observed_at: chrono::Utc::now(),
        conversation_ref: None,
        actor_entity_ref: None,
    };
    let mut uploaded = None;
    while let Some(field) = multipart.next_field().await.map_err(|error| Problem {
        error: error.to_string(),
    })? {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "metadata" {
            let value = field.bytes().await.map_err(|error| Problem {
                error: error.to_string(),
            })?;
            metadata = serde_json::from_slice(&value).map_err(|error| Problem {
                error: error.to_string(),
            })?;
        } else if name == "file" || name == "content" {
            if uploaded.is_some() {
                return Err(Problem {
                    error: "only one file field is supported".into(),
                });
            }
            let chunks = stream::unfold(Some(field), |field| async move {
                let mut field = field?;
                match field.chunk().await {
                    Ok(Some(chunk)) => {
                        Some((Ok::<Vec<u8>, nous_core::Error>(chunk.to_vec()), Some(field)))
                    }
                    Ok(None) => None,
                    Err(error) => Some((Err(nous_core::Error::Invalid(error.to_string())), None)),
                }
            });
            uploaded = Some(
                runtime
                    .ingest_stream(SubjectId(subject), metadata.clone(), chunks)
                    .await
                    .map_err(Problem::from)?,
            );
        }
    }
    let result = uploaded.ok_or_else(|| Problem {
        error: "file content is required".into(),
    })?;
    Ok(Json(result))
}

async fn show_artifact(
    State(runtime): State<LocalRuntime>,
    Path((subject, artifact)): Path<(Uuid, Uuid)>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .artifact(SubjectId(subject), nous_core::ArtifactId(artifact))
            .await
            .map_err(Problem::from)?,
    ))
}

async fn artifact_content(
    State(runtime): State<LocalRuntime>,
    Path((subject, artifact)): Path<(Uuid, Uuid)>,
) -> Result<Response, Problem> {
    let (metadata, stream) = runtime
        .artifact_stream(SubjectId(subject), nous_core::ArtifactId(artifact))
        .await
        .map_err(Problem::from)?;
    let stream =
        stream.map(|chunk| chunk.map_err(|error| std::io::Error::other(error.to_string())));
    let response = Response::builder()
        .header(header::CONTENT_TYPE, metadata.media_type)
        .body(Body::from_stream(stream))
        .map_err(|error| Problem {
            error: error.to_string(),
        })?;
    Ok(response)
}

async fn observe(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(mut input): Json<ObservationInput>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    input.subject = SubjectId(subject);
    Ok(Json(runtime.observe(input).await.map_err(Problem::from)?))
}

#[derive(Debug, Deserialize)]
struct MaterializeRequest {
    reference: nous_core::CognitiveRef,
}

async fn materialize(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<MaterializeRequest>,
) -> Result<Json<Value>, Problem> {
    runtime
        .validate_reference(SubjectId(subject), &input.reference)
        .await
        .map_err(Problem::from)?;
    Ok(Json(serde_json::json!({
        "reference": input.reference,
        "status": "handle_only",
        "note": "materialization is performed by the Host resolver or artifact content route"
    })))
}

async fn form_memory(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(mut input): Json<ExplicitMemoryInput>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    input.subject = SubjectId(subject);
    Ok(Json(
        runtime.form_memory(input).await.map_err(Problem::from)?,
    ))
}

async fn consolidate(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(mut input): Json<nous_memory_domain::ConsolidationRequest>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    input.subject = nous_core::SubjectId(subject);
    Ok(Json(
        runtime
            .consolidate(SubjectId(subject), input)
            .await
            .map_err(Problem::from)?,
    ))
}

async fn show_memory(
    State(runtime): State<LocalRuntime>,
    Path((subject, memory)): Path<(Uuid, Uuid)>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .memory(SubjectId(subject), MemoryId(memory), None)
            .await
            .map_err(Problem::from)?,
    ))
}

async fn memory_history(
    State(runtime): State<LocalRuntime>,
    Path((subject, memory)): Path<(Uuid, Uuid)>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .memory_history(SubjectId(subject), MemoryId(memory))
            .await
            .map_err(Problem::from)?,
    ))
}

async fn revise_memory(
    State(runtime): State<LocalRuntime>,
    Path((subject, memory)): Path<(Uuid, Uuid)>,
    Json(mut input): Json<ReviseMemoryInput>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    input.subject = SubjectId(subject);
    input.memory_id = MemoryId(memory);
    Ok(Json(
        runtime.revise_memory(input).await.map_err(Problem::from)?,
    ))
}

async fn suppress_memory(
    State(runtime): State<LocalRuntime>,
    Path((subject, memory)): Path<(Uuid, Uuid)>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .suppress(SubjectId(subject), MemoryId(memory))
            .await
            .map_err(Problem::from)?,
    ))
}

async fn restore_memory(
    State(runtime): State<LocalRuntime>,
    Path((subject, memory)): Path<(Uuid, Uuid)>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .restore(SubjectId(subject), MemoryId(memory))
            .await
            .map_err(Problem::from)?,
    ))
}

async fn purge_memory(
    State(runtime): State<LocalRuntime>,
    Path((subject, memory)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Problem> {
    runtime
        .purge_memory(SubjectId(subject), MemoryId(memory))
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn query(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(mut input): Json<CognitiveQuery>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    input.subject = SubjectId(subject);
    Ok(Json(runtime.query(input).await.map_err(Problem::from)?))
}

async fn use_feedback(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(mut input): Json<UseFeedback>,
) -> Result<StatusCode, Problem> {
    input.subject = SubjectId(subject);
    runtime.use_feedback(input).await.map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn upsert_resource(
    State(runtime): State<LocalRuntime>,
    Path((subject, resource_ref)): Path<(Uuid, String)>,
    Json(mut input): Json<ResourceUpsert>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    input.resource_ref = nous_core::ResourceRef::new(resource_ref).map_err(Problem::from)?;
    Ok(Json(
        runtime
            .upsert_resource(SubjectId(subject), input)
            .await
            .map_err(Problem::from)?,
    ))
}

async fn list_resources(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .list_resources(SubjectId(subject))
            .await
            .map_err(Problem::from)?,
    ))
}

async fn delete_resource(
    State(runtime): State<LocalRuntime>,
    Path((subject, resource_ref)): Path<(Uuid, String)>,
) -> Result<StatusCode, Problem> {
    let reference = nous_core::ResourceRef::new(resource_ref).map_err(Problem::from)?;
    runtime
        .delete_resource(SubjectId(subject), reference)
        .await
        .map_err(Problem::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn rebuild_projection(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .rebuild_projection(SubjectId(subject))
            .await
            .map_err(Problem::from)?,
    ))
}

async fn create_tag(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<nous_memory_service::CreateTagRequest>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .create_tag(SubjectId(subject), input)
            .await
            .map_err(Problem::from)?,
    ))
}

async fn create_anchor(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<nous_memory_service::CreateAnchorRequest>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .create_anchor(SubjectId(subject), input)
            .await
            .map_err(Problem::from)?,
    ))
}

async fn create_association(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<nous_memory_service::CreateAssociationRequest>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .create_association(SubjectId(subject), input)
            .await
            .map_err(Problem::from)?,
    ))
}

async fn rebind_entity(
    State(runtime): State<LocalRuntime>,
    Path(subject): Path<Uuid>,
    Json(input): Json<nous_memory_service::RebindEntityRequest>,
) -> Result<Json<impl serde::Serialize>, Problem> {
    Ok(Json(
        runtime
            .rebind_entity(SubjectId(subject), input)
            .await
            .map_err(Problem::from)?,
    ))
}
