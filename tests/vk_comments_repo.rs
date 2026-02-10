mod common;

use find_w::{
    groups::repo::{NewGroup, save_group},
    vk_comments::repo::{self, NewVkComment, VkCommentKey},
    vk_posts::repo::{self as vk_posts_repo, NewVkPost},
    vk_users::repo as vk_users_repo,
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::common::{create_user, sample_vk_user};

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

async fn seed_post(pool: &PgPool, user_id: Uuid, group_id: i64, from_id: i64, post_id: i64) {
    vk_posts_repo::upsert_vk_posts(
        pool,
        user_id,
        &[NewVkPost {
            post_id,
            group_id,
            from_id,
            created_date: 1_700_020_000 + post_id,
            post_type: Some("post".to_string()),
            post_text: Some(format!("post-{post_id}")),
        }],
    )
    .await
    .expect("failed to seed post");
}

#[sqlx::test]
async fn vk_comments_upsert_inserts_and_updates_in_batch(pool: PgPool) {
    let user_id = create_user(&pool).await;
    seed_group(&pool, user_id, 10).await;
    seed_vk_user(&pool, user_id, 1000).await;
    seed_post(&pool, user_id, 10, 1000, 1).await;

    let first_batch = vec![
        NewVkComment {
            group_id: 10,
            post_id: 1,
            comment_id: 456,
            from_id: 1000,
            created_date: 1_700_000_000,
            comment_text: Some("Текст комментария".to_string()),
        },
        NewVkComment {
            group_id: 10,
            post_id: 1,
            comment_id: 457,
            from_id: 1000,
            created_date: 1_700_000_100,
            comment_text: Some("Еще один комментарий".to_string()),
        },
    ];

    let res = repo::upsert_vk_comments(&pool, user_id, &first_batch)
        .await
        .expect("failed to upsert first comments batch");
    assert_eq!(res.inserted, 2);
    assert_eq!(res.updated, 0);

    let second_batch = vec![
        NewVkComment {
            group_id: 10,
            post_id: 1,
            comment_id: 456,
            from_id: 1000,
            created_date: 1_700_000_999,
            comment_text: Some("Комментарий обновлен".to_string()),
        },
        NewVkComment {
            group_id: 10,
            post_id: 1,
            comment_id: 458,
            from_id: 1000,
            created_date: 1_700_001_000,
            comment_text: Some("Новый комментарий".to_string()),
        },
    ];

    let res = repo::upsert_vk_comments(&pool, user_id, &second_batch)
        .await
        .expect("failed to upsert second comments batch");
    assert_eq!(res.inserted, 1);
    assert_eq!(res.updated, 1);

    let rows = repo::list_vk_comments(&pool, user_id, 100, 0)
        .await
        .expect("failed to list comments");
    assert_eq!(rows.len(), 3);

    let updated = rows
        .iter()
        .find(|row| row.group_id == 10 && row.post_id == 1 && row.comment_id == 456)
        .expect("updated comment not found");
    assert_eq!(updated.created_date, 1_700_000_999);
    assert_eq!(
        updated.comment_text.as_deref(),
        Some("Комментарий обновлен")
    );
}

#[sqlx::test]
async fn vk_comments_delete_is_batch_and_scoped_to_user(pool: PgPool) {
    let user_one = create_user(&pool).await;
    let user_two = create_user(&pool).await;

    seed_group(&pool, user_one, 20).await;
    seed_group(&pool, user_two, 20).await;
    seed_vk_user(&pool, user_one, 2000).await;
    seed_vk_user(&pool, user_two, 2000).await;
    seed_post(&pool, user_one, 20, 2000, 11).await;
    seed_post(&pool, user_two, 20, 2000, 11).await;

    repo::upsert_vk_comments(
        &pool,
        user_one,
        &[
            NewVkComment {
                group_id: 20,
                post_id: 11,
                comment_id: 1,
                from_id: 2000,
                created_date: 100,
                comment_text: Some("u1-c1".to_string()),
            },
            NewVkComment {
                group_id: 20,
                post_id: 11,
                comment_id: 2,
                from_id: 2000,
                created_date: 101,
                comment_text: Some("u1-c2".to_string()),
            },
        ],
    )
    .await
    .expect("failed to seed user one comments");

    repo::upsert_vk_comments(
        &pool,
        user_two,
        &[NewVkComment {
            group_id: 20,
            post_id: 11,
            comment_id: 1,
            from_id: 2000,
            created_date: 102,
            comment_text: Some("u2-c1".to_string()),
        }],
    )
    .await
    .expect("failed to seed user two comments");

    let deleted = repo::delete_vk_comments(
        &pool,
        user_one,
        &[
            VkCommentKey {
                group_id: 20,
                post_id: 11,
                comment_id: 1,
            },
            VkCommentKey {
                group_id: 20,
                post_id: 11,
                comment_id: 999,
            },
        ],
    )
    .await
    .expect("failed to delete comments");
    assert_eq!(deleted, 1);

    let user_one_rows = repo::list_vk_comments(&pool, user_one, 100, 0)
        .await
        .expect("failed to list user one comments");
    assert_eq!(user_one_rows.len(), 1);
    assert_eq!(user_one_rows[0].comment_id, 2);

    let user_two_rows = repo::list_vk_comments(&pool, user_two, 100, 0)
        .await
        .expect("failed to list user two comments");
    assert_eq!(user_two_rows.len(), 1);
    assert_eq!(user_two_rows[0].comment_id, 1);
}

#[sqlx::test]
async fn vk_comments_cascade_from_posts_and_vk_users(pool: PgPool) {
    let user_id = create_user(&pool).await;
    seed_group(&pool, user_id, 30).await;
    seed_vk_user(&pool, user_id, 3000).await;
    seed_vk_user(&pool, user_id, 3001).await;
    seed_post(&pool, user_id, 30, 3000, 21).await;
    seed_post(&pool, user_id, 30, 3001, 22).await;

    repo::upsert_vk_comments(
        &pool,
        user_id,
        &[NewVkComment {
            group_id: 30,
            post_id: 21,
            comment_id: 1,
            from_id: 3000,
            created_date: 200,
            comment_text: Some("for post 21".to_string()),
        }],
    )
    .await
    .expect("failed to seed comments for post 21");

    repo::upsert_vk_comments(
        &pool,
        user_id,
        &[NewVkComment {
            group_id: 30,
            post_id: 22,
            comment_id: 2,
            from_id: 3001,
            created_date: 201,
            comment_text: Some("for post 22".to_string()),
        }],
    )
    .await
    .expect("failed to seed comments for post 22");

    sqlx::query!(
        "DELETE FROM vk_posts WHERE user_id = $1 AND group_id = $2 AND post_id = $3",
        user_id,
        30_i64,
        21_i64
    )
    .execute(&pool)
    .await
    .expect("failed to delete post");

    let left_for_deleted_post = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM vk_comments
           WHERE user_id = $1 AND group_id = $2 AND post_id = $3"#,
        user_id,
        30_i64,
        21_i64
    )
    .fetch_one(&pool)
    .await
    .expect("failed to count comments by post");
    assert_eq!(left_for_deleted_post, 0);

    sqlx::query!(
        "DELETE FROM vk_users WHERE user_id = $1 AND vk_user_id = $2",
        user_id,
        3001_i64
    )
    .execute(&pool)
    .await
    .expect("failed to delete vk user");

    let left_for_deleted_vk_user = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM vk_comments
           WHERE user_id = $1 AND from_id = $2"#,
        user_id,
        3001_i64
    )
    .fetch_one(&pool)
    .await
    .expect("failed to count comments by vk user");
    assert_eq!(left_for_deleted_vk_user, 0);
}
