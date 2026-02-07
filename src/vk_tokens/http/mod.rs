use axum::{Router, routing::post};

use crate::AppState;

mod dto;
pub(crate) mod handlers;

pub use dto::{
    AddVkTokensRequest, AddVkTokensResponse, DeleteVkTokensRequest, DeleteVkTokensResponse,
};
pub use handlers::{add_vk_tokens, delete_vk_tokens};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(add_vk_tokens).delete(delete_vk_tokens))
}
