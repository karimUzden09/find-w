use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{
        Error as PHError, PasswordHasher, SaltString,
        rand_core::{OsRng, RngCore},
    },
};
use axum::{Json, extract::State, http::StatusCode};
use base64::Engine;
use jsonwebtoken::{Header, encode};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};

use crate::{
    AppState, Claims,
    auth::http::{
        LoginRequest, LoginResponse, RegisterRequest, RegisterResponse,
        dto::{LogoutRequest, RefreshRequest, RefreshResponse},
    },
    error::{ApiError, ApiResult},
};

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = RegisterResponse),
        (status = 409, description = "Email already exists", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(mut req): Json<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<RegisterResponse>)> {
    req.email = req.email.trim().to_string();
    let salt_string = SaltString::generate(OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt_string)
        .map_err(|error| ApiError::Hash(error.to_string()))?
        .to_string();

    let res = sqlx::query_scalar!(
        r#"
    INSERT INTO users (email,password_hash) VALUES ($1, $2) RETURNING id"#,
        req.email,
        password_hash,
    )
    .fetch_one(&state.db)
    .await;
    let user_id = match res {
        Ok(id) => id,
        Err(sqlx::Error::Database(db_error)) if db_error.code().as_deref() == Some("23505") => {
            return Err(ApiError::EmailTaken);
        }
        Err(e) => return Err(ApiError::Db(e)),
    };
    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            id: user_id,
            email: req.email,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Access and refresh tokens", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(login_reqest): Json<LoginRequest>,
) -> ApiResult<(StatusCode, Json<LoginResponse>)> {
    let email = login_reqest.email.trim().to_string();
    let row = sqlx::query!(
        r#"
    SELECT id, password_hash
    FROM users
    WHERE email = $1
    "#,
        email
    )
    .fetch_optional(&state.db)
    .await
    .map_err(ApiError::Db)?;
    let row = match row {
        Some(r) => r,
        None => return Err(ApiError::Unauthorized),
    };
    let parsed =
        PasswordHash::new(&row.password_hash).map_err(|e| ApiError::Hash(e.to_string()))?;
    match Argon2::default().verify_password(login_reqest.password.as_bytes(), &parsed) {
        Ok(()) => {}
        Err(PHError::Password) => return Err(ApiError::Unauthorized),
        Err(e) => return Err(ApiError::Hash(e.to_string())),
    }
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let exp = now + 60 * 30; // 30 minutes
    let claims = Claims {
        sub: row.id.to_string(),
        iat: now,
        exp,
    };

    let token = encode(&Header::default(), &claims, &state.jwt_enc)
        .map_err(|e| ApiError::Hash(e.to_string()))?;

    let refresh_token = new_refresh_token();
    let token_hash = hash_refresh_token(&refresh_token);
    let expaires_at = OffsetDateTime::now_utc() + time::Duration::days(30);

    let _row = sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2 , $3)
        RETURNING id
        "#,
        row.id,
        token_hash,
        expaires_at
    )
    .fetch_one(&state.db)
    .await
    .map_err(ApiError::Db)?;

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            access_token: token,
            refresh_token,
            token_type: "Bearer",
        }),
    ))
}

fn new_refresh_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn hash_refresh_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex::encode(digest)
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Access and refresh tokens rotated", body = RefreshResponse),
        (status = 401, description = "Invalid or expired refresh token", body = crate::error::ErrorBody),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    tag = "Auth"
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> ApiResult<(StatusCode, Json<RefreshResponse>)> {
    let now = OffsetDateTime::now_utc();
    let token_hash = hash_refresh_token(&req.refresh_token);

    let mut tx = state.db.begin().await.map_err(ApiError::Db)?;
    let old = sqlx::query!(
        r#"
        SELECT id, user_id, expires_at, revoked_at
        FROM refresh_tokens
        WHERE token_hash = $1
        "#,
        token_hash
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(ApiError::Db)?
    .ok_or(ApiError::Unauthorized)?;

    if old.revoked_at.is_some() || old.expires_at <= now {
        return Err(ApiError::Unauthorized);
    }

    // new refresh token
    let new_refresh_token = new_refresh_token();
    let new_hash = hash_refresh_token(&new_refresh_token);
    let new_expires_at = now + Duration::days(30);

    let new_row = sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        old.user_id,
        new_hash,
        new_expires_at
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(ApiError::Db)?;

    let upd = sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = $1, replaced_by = $2
        WHERE id = $3 AND revoked_at IS NULL
        "#,
        now,
        new_row.id,
        old.id
    )
    .execute(&mut *tx)
    .await
    .map_err(ApiError::Db)?;

    if upd.rows_affected() != 1 {
        return Err(ApiError::Unauthorized);
    }
    tx.commit().await.map_err(ApiError::Db)?;

    //new access JWT (15 minutes)
    let iat = now.unix_timestamp();
    let exp = iat + 15 * 60;

    let claims = Claims {
        sub: old.user_id.to_string(),
        iat,
        exp,
    };

    let access_token = encode(&Header::default(), &claims, &state.jwt_enc)
        .map_err(|e| ApiError::Hash(e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(RefreshResponse {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer",
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    request_body = LogoutRequest,
    responses(
        (status = 204, description = "Refresh token revoked"),
        (status = 500, description = "Internal server error", body = crate::error::ErrorBody)
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> ApiResult<StatusCode> {
    let token_hash = hash_refresh_token(&req.refresh_token);
    let now = OffsetDateTime::now_utc();

    let _res = sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = $1
        WHERE token_hash = $2 AND revoked_at IS NULL
        "#,
        now,
        token_hash
    )
    .execute(&state.db)
    .await
    .map_err(ApiError::Db)?;

    Ok(StatusCode::NO_CONTENT)
}
