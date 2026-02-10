#![allow(dead_code)]

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use find_w::vk_users::repo::NewVkUser;
use find_w::{AppState, app::router::build_router};
use find_w::{
    groups::repo::NewGroup, vk_posts::repo as vk_posts_repo, vk_posts::repo::NewVkPost,
    vk_users::repo as vk_users_repo,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde_json::{Value, json};
use sqlx::PgPool;
use time::OffsetDateTime;
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

pub async fn create_user(db: &PgPool) -> Uuid {
    sqlx::query_scalar!(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id
        "#,
        format!("user-{}@example.test", Uuid::new_v4()),
        "integration-test-password-hash"
    )
    .fetch_one(db)
    .await
    .expect("failed to create test user")
}

pub fn sample_vk_user(vk_user_id: i64, first_name: &str, finded_date: OffsetDateTime) -> NewVkUser {
    NewVkUser {
        vk_user_id,
        sex: Some(1),
        first_name: Some(first_name.to_string()),
        last_name: Some("Ivanov".to_string()),
        city: Some("Moscow".to_string()),
        finded_date,
        is_closed: Some(false),
        screen_name: Some(format!("screen_{vk_user_id}")),
        can_access_closed: Some(true),
        about: Some(format!("about_{vk_user_id}")),
        status: Some(format!("status_{vk_user_id}")),
        bdate: Some("01.01.1990".to_string()),
        photo: Some(format!("https://img.test/{vk_user_id}.jpg")),
    }
}

pub async fn seed_group(db: &PgPool, user_id: Uuid, group_id: i64) {
    find_w::groups::repo::save_group(
        db,
        user_id,
        NewGroup {
            group_id,
            group_name: Some(format!("group-{group_id}")),
            screen_name: None,
            is_closed: None,
            public_type: None,
            photo_200: None,
            description: None,
            members_count: None,
        },
    )
    .await
    .expect("failed to seed group");
}

pub async fn seed_vk_user(db: &PgPool, user_id: Uuid, vk_user_id: i64) {
    vk_users_repo::upsert_vk_users(
        db,
        user_id,
        &[sample_vk_user(
            vk_user_id,
            "Ivan",
            OffsetDateTime::now_utc(),
        )],
    )
    .await
    .expect("failed to seed vk user");
}

pub async fn seed_post(
    db: &PgPool,
    user_id: Uuid,
    group_id: i64,
    from_id: i64,
    post_id: i64,
    created_date: i64,
) {
    vk_posts_repo::upsert_vk_posts(
        db,
        user_id,
        &[NewVkPost {
            post_id,
            group_id,
            from_id,
            created_date,
            post_type: Some("post".to_string()),
            post_text: Some(format!("post-{post_id}")),
        }],
    )
    .await
    .expect("failed to seed post");
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
