use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use nous_core::{Error, SubjectId};
use nous_memory_service::{
    LocalRuntime,
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
