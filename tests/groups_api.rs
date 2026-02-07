mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::common::TestApp;

#[sqlx::test]
async fn groups_crud_is_scoped_to_current_user(pool: PgPool) {
    let app = TestApp::new(pool);
    let user_one = app.register_and_login().await;
    let user_two = app.register_and_login().await;

    let (status, created) = app
        .post_json(
            "/groups",
            json!({
                "group_id": 12345,
                "group_name": "Rustaceans",
                "screen_name": "rustaceans",
                "is_closed": 0,
                "public_type": "group",
                "photo_200": "https://cdn.test/rustaceans.jpg",
                "description": "All about Rust",
                "members_count": 1337
            }),
            Some(&user_one.access_token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        created.get("group_id").and_then(serde_json::Value::as_i64),
        Some(12345)
    );

    let (status, list_json) = app.get_json("/groups", Some(&user_one.access_token)).await;
    assert_eq!(status, StatusCode::OK);
    let groups = list_json
        .as_array()
        .expect("groups response is not an array");
    assert_eq!(groups.len(), 1);

    let delete_status = app
        .delete("/groups/12345", Some(&user_two.access_token))
        .await;
    assert_eq!(delete_status, StatusCode::NOT_FOUND);

    let delete_status = app
        .delete("/groups/12345", Some(&user_one.access_token))
        .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let (status, list_json) = app.get_json("/groups", Some(&user_one.access_token)).await;
    assert_eq!(status, StatusCode::OK);
    let groups = list_json
        .as_array()
        .expect("groups response is not an array");
    assert!(groups.is_empty());
}

#[sqlx::test]
async fn groups_endpoints_validate_payload_and_require_auth(pool: PgPool) {
    let app = TestApp::new(pool);
    let user = app.register_and_login().await;

    let (status, _) = app
        .post_json(
            "/groups",
            json!({
                "group_id": 0
            }),
            Some(&user.access_token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let delete_status = app.delete("/groups/0", Some(&user.access_token)).await;
    assert_eq!(delete_status, StatusCode::BAD_REQUEST);

    let (status, _) = app.get_json("/groups", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
