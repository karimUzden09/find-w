use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    error::{ApiError, ApiResult},
    extractors::auth_user::AuthUser,
};

#[derive(Serialize, ToSchema)]
pub struct MeResponse {
    id: Uuid,
    email: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is alive", body = String, content_type = "text/plain")
    ),
    tag = "Core"
)]
pub async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(
    get,
    path = "/db-health",
    responses(
        (status = 200, description = "Database is alive", body = String, content_type = "text/plain"),
        (status = 400, description = "Unexpected db result", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    tag = "Core"
)]
pub async fn db_health(State(state): State<AppState>) -> ApiResult<&'static str> {
    let one = sqlx::query_scalar!("SELECT 1")
        .fetch_one(&state.db)
        .await
        .map_err(ApiError::Db)?
        .ok_or(ApiError::BadRequest("unexpected db result".to_string()))?;

    if one == 1 {
        Ok("ok")
    } else {
        // теоретически не случится
        Err(ApiError::BadRequest("unexpected db result".to_string()))
    }
}

#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, description = "Current user profile", body = MeResponse),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Core"
)]
pub async fn me(
    user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<(StatusCode, Json<MeResponse>)> {
    let row = sqlx::query!(r#"SELECT id, email FROM users WHERE id = $1"#, user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(ApiError::Db)?;

    let row = row.ok_or(ApiError::Unauthorized)?;

    Ok((
        StatusCode::OK,
        Json(MeResponse {
            id: row.id,
            email: row.email,
        }),
    ))
}
