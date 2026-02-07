use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};

use crate::{
    AppState,
    error::{ApiError, ApiResult},
    extractors::auth_user::AuthUser,
    groups::repo::NewGroup,
};

use super::dto::{CreateGroupRequest, GroupDto, GroupsQuery};

#[utoipa::path(
    post,
    path = "/groups",
    request_body = CreateGroupRequest,
    responses(
        (status = 201, description = "Group saved", body = GroupDto),
        (status = 400, description = "Invalid group payload", body = crate::error::ErrorBody),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Groups"
)]
pub async fn create_group(
    user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateGroupRequest>,
) -> ApiResult<(StatusCode, Json<GroupDto>)> {
    if request.group_id <= 0 {
        return Err(ApiError::BadRequest(
            "group_id must be greater than 0".to_string(),
        ));
    }

    let group = crate::groups::repo::save_group(
        &state.db,
        user.id,
        NewGroup {
            group_id: request.group_id,
            group_name: request.group_name,
            screen_name: request.screen_name,
            is_closed: request.is_closed,
            public_type: request.public_type,
            photo_200: request.photo_200,
            description: request.description,
            members_count: request.members_count,
        },
    )
    .await
    .map_err(ApiError::Db)?;

    Ok((
        StatusCode::CREATED,
        Json(GroupDto {
            group_id: group.group_id,
            group_name: group.group_name,
            screen_name: group.screen_name,
            is_closed: group.is_closed,
            public_type: group.public_type,
            photo_200: group.photo_200,
            description: group.description,
            members_count: group.members_count,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/groups",
    params(GroupsQuery),
    responses(
        (status = 200, description = "User groups", body = [GroupDto]),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Groups"
)]
pub async fn list_groups(
    user: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<GroupsQuery>,
) -> ApiResult<(StatusCode, Json<Vec<GroupDto>>)> {
    let limit = q.limit.unwrap_or(50).clamp(1, 100);
    let offset = q.offset.unwrap_or(0).max(0);

    let rows = crate::groups::repo::list_groups(&state.db, user.id, limit, offset)
        .await
        .map_err(ApiError::Db)?;

    let groups = rows
        .into_iter()
        .map(|group| GroupDto {
            group_id: group.group_id,
            group_name: group.group_name,
            screen_name: group.screen_name,
            is_closed: group.is_closed,
            public_type: group.public_type,
            photo_200: group.photo_200,
            description: group.description,
            members_count: group.members_count,
        })
        .collect();

    Ok((StatusCode::OK, Json(groups)))
}

#[utoipa::path(
    delete,
    path = "/groups/{group_id}",
    params(
        ("group_id" = i64, Path, description = "Group id")
    ),
    responses(
        (status = 204, description = "Group deleted"),
        (status = 400, description = "Invalid group id", body = crate::error::ErrorBody),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 404, description = "Group not found", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Groups"
)]
pub async fn delete_group(
    user: AuthUser,
    State(state): State<AppState>,
    Path(group_id): Path<i64>,
) -> ApiResult<StatusCode> {
    if group_id <= 0 {
        return Err(ApiError::BadRequest(
            "group_id must be greater than 0".to_string(),
        ));
    }

    let deleted = crate::groups::repo::delete_group_owned(&state.db, user.id, group_id)
        .await
        .map_err(ApiError::Db)?;

    if !deleted {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
