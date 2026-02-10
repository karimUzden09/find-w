mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::common::TestApp;

#[sqlx::test]
async fn user_settings_are_created_with_defaults_for_registered_user(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let user = app.register_and_login().await;

    let row = sqlx::query!(
        r#"
        SELECT user_id, search_interval_minutes
        FROM user_settings
        WHERE user_id = $1
        "#,
        user.id
    )
    .fetch_one(&pool)
    .await
    .expect("failed to fetch user settings from db");

    assert_eq!(row.user_id, user.id);
    assert_eq!(row.search_interval_minutes, 60);

    let (status, body) = app.get_json("/settings", Some(&user.access_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body.get("search_interval_minutes")
            .and_then(serde_json::Value::as_i64),
        Some(60)
    );
}

#[sqlx::test]
async fn user_settings_can_be_updated_for_current_user(pool: PgPool) {
    let app = TestApp::new(pool);
    let user = app.register_and_login().await;

    let (status, patch_body) = app
        .patch_json(
            "/settings",
            json!({
                "search_interval_minutes": 90
            }),
            Some(&user.access_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        patch_body
            .get("search_interval_minutes")
            .and_then(serde_json::Value::as_i64),
        Some(90)
    );

    let (status, body) = app.get_json("/settings", Some(&user.access_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body.get("search_interval_minutes")
            .and_then(serde_json::Value::as_i64),
        Some(90)
    );
}

#[sqlx::test]
async fn user_settings_endpoint_validates_and_requires_auth(pool: PgPool) {
    let app = TestApp::new(pool);
    let user = app.register_and_login().await;

    let (status, _) = app
        .patch_json(
            "/settings",
            json!({
                "search_interval_minutes": 29
            }),
            Some(&user.access_token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = app
        .patch_json(
            "/settings",
            json!({
                "search_interval_minutes": null
            }),
            Some(&user.access_token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = app.get_json("/settings", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = app
        .patch_json(
            "/settings",
            json!({
                "search_interval_minutes": 60
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
