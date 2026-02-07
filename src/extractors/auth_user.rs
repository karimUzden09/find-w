use axum::{extract::FromRequestParts, http::header};
use jsonwebtoken::{Algorithm, Validation, decode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, error::ApiError};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // user_id
    iat: i64,
    exp: i64,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;
        let token = auth.strip_prefix("Bearer ").ok_or(ApiError::Unauthorized)?;
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let data = decode::<Claims>(token, &state.jwt_dec, &validation)
            .map_err(|_| ApiError::Unauthorized)?;

        let user_id = Uuid::parse_str(&data.claims.sub).map_err(|_| ApiError::Unauthorized)?;

        Ok(AuthUser { id: user_id })
    }
}
