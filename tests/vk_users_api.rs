mod common;

use axum::http::StatusCode;
use find_w::vk_users::repo::{self, NewVkUser};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::common::TestApp;

async fn create_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar!(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id
        "#,
        format!("user-{}@example.test", Uuid::new_v4()),
        "integration-test-password-hash"
    )
    .fetch_one(pool)
    .await
    .expect("failed to create test user")
}

fn sample_vk_user(vk_user_id: i64, first_name: &str, finded_date: OffsetDateTime) -> NewVkUser {
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

#[sqlx::test]
async fn vk_users_list_is_paginated_and_scoped_to_current_user(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let user_one = app.register_and_login().await;
    let user_two = app.register_and_login().await;
    let now = OffsetDateTime::now_utc();

    repo::upsert_vk_users(
        &pool,
        user_one.id,
        &[
            sample_vk_user(1001, "A", now),
            sample_vk_user(1002, "B", now + Duration::minutes(1)),
            sample_vk_user(1003, "C", now + Duration::minutes(2)),
        ],
    )
    .await
    .expect("failed to insert user_one vk users");

    repo::upsert_vk_users(&pool, user_two.id, &[sample_vk_user(2001, "X", now)])
        .await
        .expect("failed to insert user_two vk users");

    let (status, body) = app
        .get_json("/vk-users?limit=1&offset=1", Some(&user_one.access_token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let page = body.as_array().expect("response must be array");
    assert_eq!(page.len(), 1);
    assert_eq!(
        page[0]
            .get("vk_user_id")
            .and_then(serde_json::Value::as_i64),
        Some(1002)
    );

    let (status, body) = app
        .get_json("/vk-users?limit=100&offset=0", Some(&user_one.access_token))
        .await;
    assert_eq!(status, StatusCode::OK);
    let all = body.as_array().expect("response must be array");
    assert_eq!(all.len(), 3);
    assert!(all.iter().all(|item| {
        item.get("vk_user_id")
            .and_then(serde_json::Value::as_i64)
            .is_some_and(|id| id != 2001)
    }));
}

#[sqlx::test]
async fn vk_users_list_requires_auth(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app.get_json("/vk-users", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn vk_users_repo_upsert_delete_and_cascade_behaviour(pool: PgPool) {
    let user_one = create_user(&pool).await;
    let user_two = create_user(&pool).await;
    let now = OffsetDateTime::now_utc();

    let first_batch = vec![
        sample_vk_user(101, "Ivan", now),
        sample_vk_user(102, "Anna", now + Duration::minutes(1)),
    ];
    let upsert = repo::upsert_vk_users(&pool, user_one, &first_batch)
        .await
        .expect("failed to upsert first batch");
    assert_eq!(upsert.inserted, 2);
    assert_eq!(upsert.updated, 0);

    let second_batch = vec![
        NewVkUser {
            vk_user_id: 101,
            sex: Some(2),
            first_name: Some("Ivan-updated".to_string()),
            last_name: Some("Updated".to_string()),
            city: Some("Saint Petersburg".to_string()),
            finded_date: now + Duration::hours(1),
            is_closed: Some(true),
            screen_name: Some("ivan_updated".to_string()),
            can_access_closed: Some(false),
            about: Some("updated about".to_string()),
            status: Some("updated status".to_string()),
            bdate: Some("02.02.1992".to_string()),
            photo: Some("https://img.test/ivan-updated.jpg".to_string()),
        },
        sample_vk_user(103, "Petr", now + Duration::hours(2)),
    ];
    let upsert = repo::upsert_vk_users(&pool, user_one, &second_batch)
        .await
        .expect("failed to upsert second batch");
    assert_eq!(upsert.inserted, 1);
    assert_eq!(upsert.updated, 1);

    repo::upsert_vk_users(
        &pool,
        user_two,
        &[sample_vk_user(
            101,
            "Other-user",
            now + Duration::minutes(2),
        )],
    )
    .await
    .expect("failed to upsert second user rows");

    let user_one_rows = repo::list_vk_users(&pool, user_one, 100, 0)
        .await
        .expect("failed to list user one rows");
    assert_eq!(user_one_rows.len(), 3);
    let updated = user_one_rows
        .iter()
        .find(|row| row.vk_user_id == 101)
        .expect("updated row not found");
    assert_eq!(updated.first_name.as_deref(), Some("Ivan-updated"));
    assert_eq!(updated.status.as_deref(), Some("updated status"));
    assert_eq!(updated.is_closed, Some(true));
    assert_eq!(updated.can_access_closed, Some(false));

    let page = repo::list_vk_users(&pool, user_one, 1, 1)
        .await
        .expect("failed to list paginated rows");
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].vk_user_id, 101);

    let deleted = repo::delete_vk_users(&pool, user_one, &[101, 9999])
        .await
        .expect("failed to delete rows");
    assert_eq!(deleted, 1);

    let user_one_rows = repo::list_vk_users(&pool, user_one, 100, 0)
        .await
        .expect("failed to list user one rows after delete");
    assert_eq!(user_one_rows.len(), 2);
    assert!(user_one_rows.iter().all(|row| row.vk_user_id != 101));

    let user_two_rows = repo::list_vk_users(&pool, user_two, 100, 0)
        .await
        .expect("failed to list user two rows after delete");
    assert_eq!(user_two_rows.len(), 1);
    assert_eq!(user_two_rows[0].vk_user_id, 101);

    let cascade_user = create_user(&pool).await;
    repo::upsert_vk_users(
        &pool,
        cascade_user,
        &[sample_vk_user(777, "Cascade", OffsetDateTime::now_utc())],
    )
    .await
    .expect("failed to insert cascade rows");

    sqlx::query!("DELETE FROM users WHERE id = $1", cascade_user)
        .execute(&pool)
        .await
        .expect("failed to delete owner user");

    let remaining = sqlx::query_scalar!(
        "SELECT COUNT(*) AS \"count!\" FROM vk_users WHERE user_id = $1",
        cascade_user
    )
    .fetch_one(&pool)
    .await
    .expect("failed to count cascade rows");
    assert_eq!(remaining, 0);
}
