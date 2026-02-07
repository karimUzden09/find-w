use axum::{Router, routing::get};

use crate::AppState;
use crate::app::docs;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(crate::core::http::routes())
        .nest("/auth", crate::auth::http::routes())
        .nest("/notes", crate::notes::http::routes())
        .nest("/groups", crate::groups::http::routes())
        .nest("/vk-users", crate::vk_users::http::routes())
        .nest("/vk-tokens", crate::vk_tokens::http::routes())
        .route("/docs", get(docs::swagger_ui))
        .route("/api-docs/openapi.json", get(docs::openapi_spec))
        .with_state(state)
}
