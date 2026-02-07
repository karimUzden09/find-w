use axum::{Router, routing::get};

use crate::AppState;

mod dto;
pub(crate) mod handlers;

pub use dto::VkUserDto;
pub use handlers::list_vk_users;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(list_vk_users))
}
