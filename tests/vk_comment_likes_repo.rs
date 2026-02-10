mod common;

use crate::common::{create_user, seed_group, seed_post, seed_vk_user};
use find_w::{
    vk_comment_likes::repo::{self, NewVkCommentLike, VkCommentLikeKey},
    vk_comments::repo::{self as vk_comments_repo, NewVkComment},
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

async fn seed_comment(
    pool: &PgPool,
    user_id: Uuid,
    group_id: i64,
    post_id: i64,
    comment_id: i64,
    from_id: i64,
) {
    vk_comments_repo::upsert_vk_comments(
        pool,
        user_id,
        &[NewVkComment {
            group_id,
            post_id,
            comment_id,
            from_id,
            created_date: 1_700_040_000 + comment_id,
            comment_text: Some(format!("comment-{comment_id}")),
        }],
    )
    .await
    .expect("failed to seed comment");
}

#[sqlx::test]
async fn vk_comment_likes_upsert_inserts_and_updates_in_batch(pool: PgPool) {
    let user_id = create_user(&pool).await;
    seed_group(&pool, user_id, 10).await;
    seed_vk_user(&pool, user_id, 1000).await;
    seed_post(&pool, user_id, 10, 1000, 1, 1_700_030_001).await;
    seed_comment(&pool, user_id, 10, 1, 101, 1000).await;
    seed_comment(&pool, user_id, 10, 1, 102, 1000).await;
    seed_comment(&pool, user_id, 10, 1, 103, 1000).await;

    let first_found = OffsetDateTime::now_utc();
    let second_found = first_found + time::Duration::minutes(5);
    let third_found = first_found + time::Duration::minutes(10);

    let first_batch = vec![
        NewVkCommentLike {
            vk_user_id: 1000,
            group_id: 10,
            post_id: 1,
            comment_id: 101,
            found_date: first_found,
        },
        NewVkCommentLike {
            vk_user_id: 1000,
            group_id: 10,
            post_id: 1,
            comment_id: 102,
            found_date: second_found,
        },
    ];

    let res = repo::upsert_vk_comment_likes(&pool, user_id, &first_batch)
        .await
        .expect("failed to upsert first comment likes batch");
    assert_eq!(res.inserted, 2);
    assert_eq!(res.updated, 0);

    let second_batch = vec![
        NewVkCommentLike {
            vk_user_id: 1000,
            group_id: 10,
            post_id: 1,
            comment_id: 101,
            found_date: third_found,
        },
        NewVkCommentLike {
            vk_user_id: 1000,
            group_id: 10,
            post_id: 1,
            comment_id: 103,
            found_date: second_found,
        },
    ];

    let res = repo::upsert_vk_comment_likes(&pool, user_id, &second_batch)
        .await
        .expect("failed to upsert second comment likes batch");
    assert_eq!(res.inserted, 1);
    assert_eq!(res.updated, 1);

    let rows = repo::list_vk_comment_likes(&pool, user_id, 100, 0)
        .await
        .expect("failed to list comment likes");
    assert_eq!(rows.len(), 3);

    let updated = rows
        .iter()
        .find(|row| {
            row.group_id == 10
                && row.post_id == 1
                && row.comment_id == 101
                && row.vk_user_id == 1000
        })
        .expect("updated like not found");
    assert_eq!(updated.found_date, third_found);
}

#[sqlx::test]
async fn vk_comment_likes_delete_is_batch_and_scoped_to_user(pool: PgPool) {
    let user_one = create_user(&pool).await;
    let user_two = create_user(&pool).await;

    seed_group(&pool, user_one, 20).await;
    seed_group(&pool, user_two, 20).await;
    seed_vk_user(&pool, user_one, 2000).await;
    seed_vk_user(&pool, user_two, 2000).await;
    seed_post(&pool, user_one, 20, 2000, 11, 1_700_030_011).await;
    seed_post(&pool, user_two, 20, 2000, 11, 1_700_030_111).await;
    seed_comment(&pool, user_one, 20, 11, 501, 2000).await;
    seed_comment(&pool, user_one, 20, 11, 502, 2000).await;
    seed_comment(&pool, user_two, 20, 11, 501, 2000).await;

    repo::upsert_vk_comment_likes(
        &pool,
        user_one,
        &[
            NewVkCommentLike {
                vk_user_id: 2000,
                group_id: 20,
                post_id: 11,
                comment_id: 501,
                found_date: OffsetDateTime::now_utc(),
            },
            NewVkCommentLike {
                vk_user_id: 2000,
                group_id: 20,
                post_id: 11,
                comment_id: 502,
                found_date: OffsetDateTime::now_utc(),
            },
        ],
    )
    .await
    .expect("failed to seed user one comment likes");

    repo::upsert_vk_comment_likes(
        &pool,
        user_two,
        &[NewVkCommentLike {
            vk_user_id: 2000,
            group_id: 20,
            post_id: 11,
            comment_id: 501,
            found_date: OffsetDateTime::now_utc(),
        }],
    )
    .await
    .expect("failed to seed user two comment likes");

    let deleted = repo::delete_vk_comment_likes(
        &pool,
        user_one,
        &[
            VkCommentLikeKey {
                vk_user_id: 2000,
                group_id: 20,
                post_id: 11,
                comment_id: 501,
            },
            VkCommentLikeKey {
                vk_user_id: 2000,
                group_id: 20,
                post_id: 11,
                comment_id: 999,
            },
        ],
    )
    .await
    .expect("failed to delete comment likes");
    assert_eq!(deleted, 1);

    let user_one_rows = repo::list_vk_comment_likes(&pool, user_one, 100, 0)
        .await
        .expect("failed to list user one comment likes");
    assert_eq!(user_one_rows.len(), 1);
    assert_eq!(user_one_rows[0].comment_id, 502);

    let user_two_rows = repo::list_vk_comment_likes(&pool, user_two, 100, 0)
        .await
        .expect("failed to list user two comment likes");
    assert_eq!(user_two_rows.len(), 1);
    assert_eq!(user_two_rows[0].comment_id, 501);
}

#[sqlx::test]
async fn vk_comment_likes_cascade_from_comments_and_vk_users(pool: PgPool) {
    let user_id = create_user(&pool).await;
    seed_group(&pool, user_id, 30).await;
    seed_vk_user(&pool, user_id, 3000).await;
    seed_vk_user(&pool, user_id, 3001).await;
    seed_post(&pool, user_id, 30, 3000, 21, 1_700_030_021).await;
    seed_comment(&pool, user_id, 30, 21, 701, 3000).await;
    seed_comment(&pool, user_id, 30, 21, 702, 3001).await;

    repo::upsert_vk_comment_likes(
        &pool,
        user_id,
        &[NewVkCommentLike {
            vk_user_id: 3000,
            group_id: 30,
            post_id: 21,
            comment_id: 701,
            found_date: OffsetDateTime::now_utc(),
        }],
    )
    .await
    .expect("failed to seed comment likes for comment 701");

    repo::upsert_vk_comment_likes(
        &pool,
        user_id,
        &[NewVkCommentLike {
            vk_user_id: 3001,
            group_id: 30,
            post_id: 21,
            comment_id: 702,
            found_date: OffsetDateTime::now_utc(),
        }],
    )
    .await
    .expect("failed to seed comment likes for comment 702");

    sqlx::query!(
        "DELETE FROM vk_comments WHERE user_id = $1 AND group_id = $2 AND post_id = $3 AND comment_id = $4",
        user_id,
        30_i64,
        21_i64,
        701_i64
    )
    .execute(&pool)
    .await
    .expect("failed to delete comment");

    let left_for_deleted_comment = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM vk_comment_likes
           WHERE user_id = $1 AND group_id = $2 AND post_id = $3 AND comment_id = $4"#,
        user_id,
        30_i64,
        21_i64,
        701_i64
    )
    .fetch_one(&pool)
    .await
    .expect("failed to count likes by comment");
    assert_eq!(left_for_deleted_comment, 0);

    sqlx::query!(
        "DELETE FROM vk_users WHERE user_id = $1 AND vk_user_id = $2",
        user_id,
        3001_i64
    )
    .execute(&pool)
    .await
    .expect("failed to delete vk user");

    let left_for_deleted_vk_user = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM vk_comment_likes
           WHERE user_id = $1 AND vk_user_id = $2"#,
        user_id,
        3001_i64
    )
    .fetch_one(&pool)
    .await
    .expect("failed to count likes by vk user");
    assert_eq!(left_for_deleted_vk_user, 0);
}
