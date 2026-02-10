use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub mod app;
pub mod auth;
pub mod core;
pub mod error;
mod extractors;
pub mod groups;
pub mod notes;
pub mod user_settings;
pub mod vk_comment_likes;
pub mod vk_comments;
pub mod vk_post_likes;
pub mod vk_posts;
pub mod vk_tokens;
pub mod vk_users;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_enc: EncodingKey,
    pub jwt_dec: DecodingKey,
    pub vk_token_enc_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    sub: String, // user_id
    iat: i64,
    exp: i64,
}
