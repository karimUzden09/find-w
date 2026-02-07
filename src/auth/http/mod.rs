use axum::{Router, routing::post};

use crate::AppState;

mod dto;
pub(crate) mod handlers;

pub use dto::{
    LoginRequest, LoginResponse, LogoutRequest, RefreshRequest, RefreshResponse, RegisterRequest,
    RegisterResponse,
};
pub use handlers::{login, logout, refresh, register};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
}
