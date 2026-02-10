use axum::{Router, routing::get};

use crate::AppState;

mod dto;
pub(crate) mod handlers;

pub use dto::{UpdateUserSettingsRequest, UserSettingsDto};
pub use handlers::{get_user_settings, update_user_settings};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(get_user_settings).patch(update_user_settings))
}
