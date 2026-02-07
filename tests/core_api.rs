mod common;

use axum::http::StatusCode;
use sqlx::PgPool;

use crate::common::TestApp;

#[sqlx::test]
async fn health_and_db_health_return_ok(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.get_text("/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");

    let (status, body) = app.get_text("/db-health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}

#[sqlx::test]
async fn me_requires_authentication(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, error_json) = app.get_json("/me", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        error_json.get("error").and_then(serde_json::Value::as_str),
        Some("UNAUTHORIZED")
    );
}

#[sqlx::test]
async fn docs_and_openapi_are_available(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, html) = app.get_text("/docs", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(html.contains("SwaggerUIBundle"));

    let (status, openapi_json) = app.get_json("/api-docs/openapi.json", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        openapi_json
            .get("openapi")
            .and_then(serde_json::Value::as_str),
        Some("3.1.0")
    );
}
