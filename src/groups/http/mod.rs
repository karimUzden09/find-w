use axum::{
    Router,
    routing::{delete, post},
};

use crate::AppState;

mod dto;
pub(crate) mod handlers;

pub use dto::{CreateGroupRequest, GroupDto};
pub use handlers::{create_group, delete_group, list_groups};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_group).get(list_groups))
        .route("/{group_id}", delete(delete_group))
}
