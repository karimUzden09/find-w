mod common;

use crate::common::{create_user, sample_vk_user};
use find_w::{
    groups::repo::{NewGroup, save_group},
    vk_posts::repo::{self, NewVkPost, VkPostKey},
    vk_users::repo::{self as vk_users_repo},
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

async fn seed_group(pool: &PgPool, user_id: Uuid, group_id: i64) {
    save_group(
        pool,
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

async fn seed_vk_user(pool: &PgPool, user_id: Uuid, vk_user_id: i64) {
    vk_users_repo::upsert_vk_users(
        pool,
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

#[sqlx::test]
async fn vk_posts_upsert_inserts_and_updates_in_batch(pool: PgPool) {
    let user_id = create_user(&pool).await;
    seed_group(&pool, user_id, 10).await;
    seed_vk_user(&pool, user_id, 1000).await;

    let first_batch = vec![
        NewVkPost {
            post_id: 1,
            group_id: 10,
            from_id: 1000,
            created_date: 1_700_000_001,
            post_type: Some("post".to_string()),
            post_text: Some("first".to_string()),
        },
        NewVkPost {
            post_id: 2,
            group_id: 10,
            from_id: 1000,
            created_date: 1_700_000_002,
            post_type: Some("post".to_string()),
            post_text: Some("second".to_string()),
        },
    ];

    let res = repo::upsert_vk_posts(&pool, user_id, &first_batch)
        .await
        .expect("failed to upsert first vk_posts batch");
    assert_eq!(res.inserted, 2);
    assert_eq!(res.updated, 0);

    let second_batch = vec![
        NewVkPost {
            post_id: 1,
            group_id: 10,
            from_id: 1000,
            created_date: 1_700_000_111,
            post_type: Some("reply".to_string()),
            post_text: Some("first-updated".to_string()),
        },
        NewVkPost {
            post_id: 3,
            group_id: 10,
            from_id: 1000,
            created_date: 1_700_000_003,
            post_type: Some("post".to_string()),
            post_text: Some("third".to_string()),
        },
    ];

    let res = repo::upsert_vk_posts(&pool, user_id, &second_batch)
        .await
        .expect("failed to upsert second vk_posts batch");
    assert_eq!(res.inserted, 1);
    assert_eq!(res.updated, 1);

    let rows = repo::list_vk_posts(&pool, user_id, 100, 0)
        .await
        .expect("failed to list vk_posts");
    assert_eq!(rows.len(), 3);

    let updated = rows
        .iter()
        .find(|row| row.post_id == 1 && row.group_id == 10)
        .expect("updated post not found");
    assert_eq!(updated.created_date, 1_700_000_111);
    assert_eq!(updated.post_type.as_deref(), Some("reply"));
    assert_eq!(updated.post_text.as_deref(), Some("first-updated"));
}

#[sqlx::test]
async fn vk_posts_delete_is_batch_and_scoped_to_user(pool: PgPool) {
    let user_one = create_user(&pool).await;
    let user_two = create_user(&pool).await;

    seed_group(&pool, user_one, 20).await;
    seed_group(&pool, user_two, 20).await;
    seed_vk_user(&pool, user_one, 2000).await;
    seed_vk_user(&pool, user_two, 2000).await;

    repo::upsert_vk_posts(
        &pool,
        user_one,
        &[
            NewVkPost {
                post_id: 11,
                group_id: 20,
                from_id: 2000,
                created_date: 10,
                post_type: Some("post".to_string()),
                post_text: Some("u1-p11".to_string()),
            },
            NewVkPost {
                post_id: 12,
                group_id: 20,
                from_id: 2000,
                created_date: 11,
                post_type: Some("post".to_string()),
                post_text: Some("u1-p12".to_string()),
            },
        ],
    )
    .await
    .expect("failed to seed user one posts");

    repo::upsert_vk_posts(
        &pool,
        user_two,
        &[NewVkPost {
            post_id: 11,
            group_id: 20,
            from_id: 2000,
            created_date: 12,
            post_type: Some("post".to_string()),
            post_text: Some("u2-p11".to_string()),
        }],
    )
    .await
    .expect("failed to seed user two posts");

    let deleted = repo::delete_vk_posts(
        &pool,
        user_one,
        &[
            VkPostKey {
                group_id: 20,
                post_id: 11,
            },
            VkPostKey {
                group_id: 20,
                post_id: 999,
            },
        ],
    )
    .await
    .expect("failed to delete vk_posts");
    assert_eq!(deleted, 1);

    let user_one_rows = repo::list_vk_posts(&pool, user_one, 100, 0)
        .await
        .expect("failed to list user one posts");
    assert_eq!(user_one_rows.len(), 1);
    assert_eq!(user_one_rows[0].post_id, 12);

    let user_two_rows = repo::list_vk_posts(&pool, user_two, 100, 0)
        .await
        .expect("failed to list user two posts");
    assert_eq!(user_two_rows.len(), 1);
    assert_eq!(user_two_rows[0].post_id, 11);
}

#[sqlx::test]
async fn vk_posts_have_fk_to_groups_and_vk_users_with_cascade(pool: PgPool) {
    let user_id = create_user(&pool).await;

    seed_group(&pool, user_id, 30).await;
    seed_group(&pool, user_id, 31).await;
    seed_vk_user(&pool, user_id, 3000).await;
    seed_vk_user(&pool, user_id, 3001).await;

    repo::upsert_vk_posts(
        &pool,
        user_id,
        &[NewVkPost {
            post_id: 21,
            group_id: 30,
            from_id: 3000,
            created_date: 20,
            post_type: Some("post".to_string()),
            post_text: Some("cascade-group".to_string()),
        }],
    )
    .await
    .expect("failed to seed group cascade post");

    repo::upsert_vk_posts(
        &pool,
        user_id,
        &[NewVkPost {
            post_id: 22,
            group_id: 31,
            from_id: 3001,
            created_date: 21,
            post_type: Some("post".to_string()),
            post_text: Some("cascade-vk-user".to_string()),
        }],
    )
    .await
    .expect("failed to seed vk user cascade post");

    sqlx::query!(
        "DELETE FROM groups WHERE user_id = $1 AND group_id = $2",
        user_id,
        30_i64
    )
    .execute(&pool)
    .await
    .expect("failed to delete group");

    let group_post_count = sqlx::query_scalar!(
        "SELECT COUNT(*) AS \"count!\" FROM vk_posts WHERE user_id = $1 AND group_id = $2",
        user_id,
        30_i64
    )
    .fetch_one(&pool)
    .await
    .expect("failed to count posts after group delete");
    assert_eq!(group_post_count, 0);

    sqlx::query!(
        "DELETE FROM vk_users WHERE user_id = $1 AND vk_user_id = $2",
        user_id,
        3001_i64
    )
    .execute(&pool)
    .await
    .expect("failed to delete vk user");

    let vk_user_post_count = sqlx::query_scalar!(
        "SELECT COUNT(*) AS \"count!\" FROM vk_posts WHERE user_id = $1 AND from_id = $2",
        user_id,
        3001_i64
    )
    .fetch_one(&pool)
    .await
    .expect("failed to count posts after vk_user delete");
    assert_eq!(vk_user_post_count, 0);
}
