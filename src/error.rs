use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: &'static str,
    pub message: String,
}

#[derive(Debug)]
pub enum ApiError {
    EmailTaken,
    BadRequest(String),
    Db(sqlx::Error),
    Hash(String),
    Unauthorized,
    NotFound,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::EmailTaken => {
                let mut res = Json(ErrorBody {
                    error: "EMAIL_TAKEN",
                    message: "Email already exists".to_string(),
                })
                .into_response();

                *res.status_mut() = StatusCode::CONFLICT;
                res
            }
            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    error: "BAD_REQUEST",
                    message: msg,
                }),
            )
                .into_response(),
            ApiError::Db(e) => {
                tracing::error!("db error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: "INTERNAL",
                        message: "Internal server error".to_string(),
                    }),
                )
                    .into_response()
            }
            ApiError::Hash(msg) => {
                tracing::error!("hash error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: "INTERNAL",
                        message: "Internal server error".to_string(),
                    }),
                )
                    .into_response()
            }
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "UNAUTHORIZED",
                    message: "Invalid credentials".to_string(),
                }),
            )
                .into_response(),
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "NOT FOUND",
                    message: "Resource not found".to_string(),
                }),
            )
                .into_response(),
        }
    }
}
