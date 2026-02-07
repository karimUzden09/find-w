use axum::{Router, routing::get};

use crate::AppState;

pub(crate) mod handlers;

pub use handlers::{MeResponse, db_health, health, me};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/db-health", get(db_health))
        .route("/me", get(me))
}
