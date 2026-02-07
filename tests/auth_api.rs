mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::common::TestApp;

#[sqlx::test]
async fn register_login_and_me_flow(pool: PgPool) {
    let app = TestApp::new(pool);
    let user = app.register_and_login().await;

    let (status, me_json) = app.get_json("/me", Some(&user.access_token)).await;
    assert_eq!(status, StatusCode::OK);

    let expected_id = user.id.to_string();
    assert_eq!(
        me_json.get("id").and_then(serde_json::Value::as_str),
        Some(expected_id.as_str())
    );
    assert_eq!(
        me_json.get("email").and_then(serde_json::Value::as_str),
        Some(user.email.as_str())
    );
}

#[sqlx::test]
async fn register_with_same_email_returns_conflict(pool: PgPool) {
    let app = TestApp::new(pool);
    let email = "same-user@example.test";

    let (status, _) = app
        .post_json(
            "/auth/register",
            json!({
                "email": email,
                "password": "password-123"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, error_json) = app
        .post_json(
            "/auth/register",
            json!({
                "email": email,
                "password": "password-123"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        error_json.get("error").and_then(serde_json::Value::as_str),
        Some("EMAIL_TAKEN")
    );
}

#[sqlx::test]
async fn login_with_wrong_password_returns_unauthorized(pool: PgPool) {
    let app = TestApp::new(pool);
    let email = "wrong-password@example.test";

    let (status, _) = app
        .post_json(
            "/auth/register",
            json!({
                "email": email,
                "password": "correct-password"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, error_json) = app
        .post_json(
            "/auth/login",
            json!({
                "email": email,
                "password": "wrong-password"
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        error_json.get("error").and_then(serde_json::Value::as_str),
        Some("UNAUTHORIZED")
    );
}

#[sqlx::test]
async fn refresh_rotates_tokens_and_old_refresh_becomes_invalid(pool: PgPool) {
    let app = TestApp::new(pool);
    let user = app.register_and_login().await;

    let (status, refresh_json) = app
        .post_json(
            "/auth/refresh",
            json!({
                "refresh_token": user.refresh_token
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let new_access_token = refresh_json
        .get("access_token")
        .and_then(serde_json::Value::as_str)
        .expect("refresh response misses access_token");
    let new_refresh_token = refresh_json
        .get("refresh_token")
        .and_then(serde_json::Value::as_str)
        .expect("refresh response misses refresh_token");

    assert_ne!(new_refresh_token, user.refresh_token);
    assert!(!new_access_token.is_empty());

    let (status, _) = app
        .post_json(
            "/auth/refresh",
            json!({
                "refresh_token": user.refresh_token
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, me_json) = app.get_json("/me", Some(new_access_token)).await;
    assert_eq!(status, StatusCode::OK);
    let expected_id = user.id.to_string();
    assert_eq!(
        me_json.get("id").and_then(serde_json::Value::as_str),
        Some(expected_id.as_str())
    );
}

#[sqlx::test]
async fn logout_revokes_refresh_token(pool: PgPool) {
    let app = TestApp::new(pool);
    let user = app.register_and_login().await;

    let (status, _) = app
        .post_json(
            "/auth/logout",
            json!({
                "refresh_token": user.refresh_token
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = app
        .post_json(
            "/auth/refresh",
            json!({
                "refresh_token": user.refresh_token
            }),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
