mod common;

use axum::http::StatusCode;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::common::{TEST_VK_TOKEN_ENC_KEY, TestApp};

#[sqlx::test]
async fn vk_tokens_are_encrypted_and_can_be_listed_deleted(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let user = app.register_and_login().await;

    let (status, add_json) = app
        .post_json(
            "/vk-tokens",
            json!({
                "tokens": [
                    "vk_token_1",
                    "vk_token_2",
                    "vk_token_1"
                ]
            }),
            Some(&user.access_token),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        add_json.get("inserted").and_then(serde_json::Value::as_i64),
        Some(2)
    );
    assert_eq!(
        add_json.get("skipped").and_then(serde_json::Value::as_i64),
        Some(1)
    );

    let rows = sqlx::query!(
        r#"
        SELECT token_hash, token_encrypted
        FROM vk_tokens
        WHERE user_id = $1
        "#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .expect("failed to query vk_tokens");

    assert_eq!(rows.len(), 2);
    let expected_hash = hex::encode(Sha256::digest("vk_token_1".as_bytes()));
    assert!(rows.iter().any(|row| row.token_hash == expected_hash));
    assert!(
        rows.iter()
            .all(|row| row.token_encrypted != b"vk_token_1".to_vec())
    );

    let listed =
        find_w::vk_tokens::repo::list_vk_tokens_for_user(&pool, user.id, TEST_VK_TOKEN_ENC_KEY)
            .await
            .expect("failed to list vk tokens for user");

    assert_eq!(listed.len(), 2);
    assert!(listed.iter().any(|token| token.token == "vk_token_1"));
    assert!(listed.iter().any(|token| token.token == "vk_token_2"));

    let (status, delete_json) = app
        .delete_json(
            "/vk-tokens",
            json!({
                "tokens": [
                    "vk_token_1"
                ]
            }),
            Some(&user.access_token),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        delete_json
            .get("deleted")
            .and_then(serde_json::Value::as_i64),
        Some(1)
    );

    let listed =
        find_w::vk_tokens::repo::list_vk_tokens_for_user(&pool, user.id, TEST_VK_TOKEN_ENC_KEY)
            .await
            .expect("failed to list vk tokens after delete");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].token, "vk_token_2");
}

#[sqlx::test]
async fn vk_tokens_endpoints_validate_payload_and_require_auth(pool: PgPool) {
    let app = TestApp::new(pool);
    let user = app.register_and_login().await;

    let (status, _) = app
        .post_json(
            "/vk-tokens",
            json!({ "tokens": [] }),
            Some(&user.access_token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = app
        .delete_json(
            "/vk-tokens",
            json!({ "tokens": ["   "] }),
            Some(&user.access_token),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = app
        .post_json("/vk-tokens", json!({ "tokens": ["vk_token_1"] }), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
