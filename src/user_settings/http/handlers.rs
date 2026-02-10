use axum::{Json, extract::State, http::StatusCode};

use crate::{
    AppState,
    error::{ApiError, ApiResult},
    extractors::auth_user::AuthUser,
};

use super::dto::{UpdateUserSettingsRequest, UserSettingsDto};

#[utoipa::path(
    get,
    path = "/settings",
    responses(
        (status = 200, description = "Current user settings", body = UserSettingsDto),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn get_user_settings(
    user: AuthUser,
    State(state): State<AppState>,
) -> ApiResult<(StatusCode, Json<UserSettingsDto>)> {
    let settings = crate::user_settings::repo::get_user_settings(&state.db, user.id)
        .await
        .map_err(ApiError::Db)?;

    Ok((
        StatusCode::OK,
        Json(UserSettingsDto {
            search_interval_minutes: settings.search_interval_minutes,
            updated_at: settings.updated_at,
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/settings",
    request_body = UpdateUserSettingsRequest,
    responses(
        (status = 200, description = "Current user settings updated", body = UserSettingsDto),
        (status = 400, description = "Invalid request payload", body = crate::error::ErrorBody),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Settings"
)]
pub async fn update_user_settings(
    user: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<UpdateUserSettingsRequest>,
) -> ApiResult<(StatusCode, Json<UserSettingsDto>)> {
    let search_interval_minutes = req.search_interval_minutes.ok_or(ApiError::BadRequest(
        "search_interval_minutes is required".to_string(),
    ))?;

    if search_interval_minutes < 30 {
        return Err(ApiError::BadRequest(
            "search_interval_minutes must be at least 30".to_string(),
        ));
    }

    let settings = crate::user_settings::repo::update_user_settings(
        &state.db,
        user.id,
        search_interval_minutes,
    )
    .await
    .map_err(ApiError::Db)?;

    Ok((
        StatusCode::OK,
        Json(UserSettingsDto {
            search_interval_minutes: settings.search_interval_minutes,
            updated_at: settings.updated_at,
        }),
    ))
}
