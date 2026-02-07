use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

use crate::{
    AppState,
    error::{ApiError, ApiResult},
    extractors::auth_user::AuthUser,
};

use super::dto::{VkUserDto, VkUsersQuery};

#[utoipa::path(
    get,
    path = "/vk-users",
    params(VkUsersQuery),
    responses(
        (status = 200, description = "User VK users", body = [VkUserDto]),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "VK Users"
)]
pub async fn list_vk_users(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<VkUsersQuery>,
) -> ApiResult<(StatusCode, Json<Vec<VkUserDto>>)> {
    let limit = q.limit.unwrap_or(50).clamp(1, 100);
    let offset = q.offset.unwrap_or(0).max(0);

    let rows = crate::vk_users::repo::list_vk_users(&state.db, user.id, limit, offset)
        .await
        .map_err(ApiError::Db)?;

    let vk_users = rows
        .into_iter()
        .map(|row| VkUserDto {
            vk_user_id: row.vk_user_id,
            sex: row.sex,
            first_name: row.first_name,
            last_name: row.last_name,
            city: row.city,
            finded_date: row.finded_date,
            is_closed: row.is_closed,
            screen_name: row.screen_name,
            can_access_closed: row.can_access_closed,
            about: row.about,
            status: row.status,
            bdate: row.bdate,
            photo: row.photo,
        })
        .collect();

    Ok((StatusCode::OK, Json(vk_users)))
}
