#![allow(dead_code)]

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use find_w::{AppState, app::router::build_router};
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

pub const TEST_JWT_SECRET: &str = "integration-test-jwt-secret";
pub const TEST_VK_TOKEN_ENC_KEY: &str = "integration-test-vk-token-enc-key";

pub struct TestApp {
    app: Router,
}

pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl TestApp {
    pub fn new(db: PgPool) -> Self {
        let state = AppState {
            db,
            jwt_enc: EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
            jwt_dec: DecodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
            vk_token_enc_key: TEST_VK_TOKEN_ENC_KEY.to_string(),
        };

        Self {
            app: build_router(state),
        }
    }

    pub async fn post_json(
        &self,
        path: &str,
        body: Value,
        bearer: Option<&str>,
    ) -> (StatusCode, Value) {
        self.request_json(Method::POST, path, Some(body), bearer)
            .await
    }

    pub async fn delete_json(
        &self,
        path: &str,
        body: Value,
        bearer: Option<&str>,
    ) -> (StatusCode, Value) {
        self.request_json(Method::DELETE, path, Some(body), bearer)
            .await
    }

    pub async fn get_json(&self, path: &str, bearer: Option<&str>) -> (StatusCode, Value) {
        self.request_json(Method::GET, path, None, bearer).await
    }

    pub async fn get_text(&self, path: &str, bearer: Option<&str>) -> (StatusCode, String) {
        let (status, bytes) = self.request(Method::GET, path, None, bearer).await;
        let text = String::from_utf8(bytes).expect("response is not valid utf-8 text");
        (status, text)
    }

    pub async fn delete(&self, path: &str, bearer: Option<&str>) -> StatusCode {
        self.request_status(Method::DELETE, path, bearer).await
    }

    async fn request_json(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        bearer: Option<&str>,
    ) -> (StatusCode, Value) {
        let (status, bytes) = self.request(method, path, body, bearer).await;

        if bytes.is_empty() {
            return (status, Value::Null);
        }

        let value = serde_json::from_slice(&bytes).expect("response is not valid json");
        (status, value)
    }

    async fn request_status(&self, method: Method, path: &str, bearer: Option<&str>) -> StatusCode {
        let (status, _) = self.request(method, path, None, bearer).await;
        status
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
        bearer: Option<&str>,
    ) -> (StatusCode, Vec<u8>) {
        let mut req_builder = Request::builder().method(method).uri(path);

        if let Some(token) = bearer {
            req_builder = req_builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }

        let body = match body {
            Some(body) => {
                req_builder = req_builder.header(header::CONTENT_TYPE, "application/json");
                Body::from(serde_json::to_vec(&body).expect("failed to serialize request body"))
            }
            None => Body::empty(),
        };

        let req = req_builder.body(body).expect("failed to build request");
        let response = self
            .app
            .clone()
            .oneshot(req)
            .await
            .expect("request execution failed");

        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("failed to read response body");

        (status, bytes.to_vec())
    }

    pub async fn register_and_login(&self) -> TestUser {
        let email = format!("user-{}@example.test", Uuid::new_v4());
        let password = "strong-password-123";

        let (register_status, register_json) = self
            .post_json(
                "/auth/register",
                json!({
                    "email": email,
                    "password": password
                }),
                None,
            )
            .await;
        assert_eq!(register_status, StatusCode::CREATED);

        let user_id = register_json
            .get("id")
            .and_then(Value::as_str)
            .expect("register response misses user id");
        let user_id = Uuid::parse_str(user_id).expect("invalid user id from register response");

        let (login_status, login_json) = self
            .post_json(
                "/auth/login",
                json!({
                    "email": email,
                    "password": password
                }),
                None,
            )
            .await;
        assert_eq!(login_status, StatusCode::OK);

        let access_token = login_json
            .get("access_token")
            .and_then(Value::as_str)
            .expect("login response misses access token")
            .to_string();
        let refresh_token = login_json
            .get("refresh_token")
            .and_then(Value::as_str)
            .expect("login response misses refresh token")
            .to_string();

        TestUser {
            id: user_id,
            email,
            access_token,
            refresh_token,
        }
    }
}
