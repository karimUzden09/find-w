use std::net::SocketAddr;

use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app::router::build_router;

mod app;
mod auth;
mod core;
mod error;
mod extractors;
mod groups;
mod notes;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_enc: EncodingKey,
    pub jwt_dec: DecodingKey,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // user_id
    iat: i64,
    exp: i64,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let state = AppState {
        db,
        jwt_enc: EncodingKey::from_secret(jwt_secret.as_bytes()),
        jwt_dec: DecodingKey::from_secret(jwt_secret.as_bytes()),
    };
    let app = build_router(state);
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    tracing::info!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
