use axum::{Json, extract::State, http::StatusCode};

use crate::{
    AppState,
    error::{ApiError, ApiResult},
    extractors::auth_user::AuthUser,
};

use super::dto::{
    AddVkTokensRequest, AddVkTokensResponse, DeleteVkTokensRequest, DeleteVkTokensResponse,
};

fn normalize_tokens(tokens: Vec<String>) -> ApiResult<Vec<String>> {
    if tokens.is_empty() {
        return Err(ApiError::BadRequest(
            "tokens must contain at least one value".to_string(),
        ));
    }

    if tokens.len() > 100 {
        return Err(ApiError::BadRequest(
            "tokens can contain up to 100 values".to_string(),
        ));
    }

    let mut normalized = Vec::with_capacity(tokens.len());
    for token in tokens {
        let token = token.trim().to_string();
        if token.is_empty() {
            return Err(ApiError::BadRequest(
                "token value cannot be empty".to_string(),
            ));
        }
        normalized.push(token);
    }
    Ok(normalized)
}

#[utoipa::path(
    post,
    path = "/vk-tokens",
    request_body = AddVkTokensRequest,
    responses(
        (status = 201, description = "VK tokens saved", body = AddVkTokensResponse),
        (status = 400, description = "Invalid request payload", body = crate::error::ErrorBody),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "VK Tokens"
)]
pub async fn add_vk_tokens(
    user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<AddVkTokensRequest>,
) -> ApiResult<(StatusCode, Json<AddVkTokensResponse>)> {
    let tokens = normalize_tokens(request.tokens)?;

    let res =
        crate::vk_tokens::repo::add_vk_tokens(&state.db, user.id, &tokens, &state.vk_token_enc_key)
            .await
            .map_err(ApiError::Db)?;

    Ok((
        StatusCode::CREATED,
        Json(AddVkTokensResponse {
            inserted: res.inserted,
            skipped: res.skipped,
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/vk-tokens",
    request_body = DeleteVkTokensRequest,
    responses(
        (status = 200, description = "VK tokens deleted", body = DeleteVkTokensResponse),
        (status = 400, description = "Invalid request payload", body = crate::error::ErrorBody),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "VK Tokens"
)]
pub async fn delete_vk_tokens(
    user: AuthUser,
    State(state): State<AppState>,
    Json(request): Json<DeleteVkTokensRequest>,
) -> ApiResult<(StatusCode, Json<DeleteVkTokensResponse>)> {
    let tokens = normalize_tokens(request.tokens)?;
    let deleted = crate::vk_tokens::repo::delete_vk_tokens(&state.db, user.id, &tokens)
        .await
        .map_err(ApiError::Db)?;

    Ok((StatusCode::OK, Json(DeleteVkTokensResponse { deleted })))
}
