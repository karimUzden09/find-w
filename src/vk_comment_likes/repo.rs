use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UpsertVkCommentLikesResult {
    pub inserted: i64,
    pub updated: i64,
}

#[derive(Debug, Clone)]
pub struct NewVkCommentLike {
    pub vk_user_id: i64,
    pub group_id: i64,
    pub post_id: i64,
    pub comment_id: i64,
    pub found_date: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct VkCommentLikeKey {
    pub vk_user_id: i64,
    pub group_id: i64,
    pub post_id: i64,
    pub comment_id: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VkCommentLike {
    pub user_id: Uuid,
    pub vk_user_id: i64,
    pub group_id: i64,
    pub post_id: i64,
    pub comment_id: i64,
    pub found_date: OffsetDateTime,
}

pub async fn upsert_vk_comment_likes(
    db: &PgPool,
    user_id: Uuid,
    likes: &[NewVkCommentLike],
) -> Result<UpsertVkCommentLikesResult, sqlx::Error> {
    if likes.is_empty() {
        return Ok(UpsertVkCommentLikesResult {
            inserted: 0,
            updated: 0,
        });
    }

    let mut vk_user_ids = Vec::with_capacity(likes.len());
    let mut group_ids = Vec::with_capacity(likes.len());
    let mut post_ids = Vec::with_capacity(likes.len());
    let mut comment_ids = Vec::with_capacity(likes.len());
    let mut found_dates = Vec::with_capacity(likes.len());

    for like in likes {
        vk_user_ids.push(like.vk_user_id);
        group_ids.push(like.group_id);
        post_ids.push(like.post_id);
        comment_ids.push(like.comment_id);
        found_dates.push(like.found_date);
    }

    let rows = sqlx::query!(
        r#"
        INSERT INTO vk_comment_likes (
            user_id,
            vk_user_id,
            group_id,
            post_id,
            comment_id,
            found_date
        )
        SELECT
            $1::uuid,
            src.vk_user_id,
            src.group_id,
            src.post_id,
            src.comment_id,
            src.found_date
        FROM UNNEST(
            $2::bigint[],
            $3::bigint[],
            $4::bigint[],
            $5::bigint[],
            $6::timestamptz[]
        ) AS src(vk_user_id, group_id, post_id, comment_id, found_date)
        ON CONFLICT (user_id, vk_user_id, group_id, post_id, comment_id)
        DO UPDATE SET
            found_date = EXCLUDED.found_date
        RETURNING (xmax = 0) AS "inserted!"
        "#,
        user_id,
        &vk_user_ids,
        &group_ids,
        &post_ids,
        &comment_ids,
        &found_dates
    )
    .fetch_all(db)
    .await?;

    let inserted = rows.iter().filter(|row| row.inserted).count() as i64;
    let updated = rows.len() as i64 - inserted;

    Ok(UpsertVkCommentLikesResult { inserted, updated })
}

pub async fn delete_vk_comment_likes(
    db: &PgPool,
    user_id: Uuid,
    keys: &[VkCommentLikeKey],
) -> Result<i64, sqlx::Error> {
    if keys.is_empty() {
        return Ok(0);
    }

    let mut vk_user_ids = Vec::with_capacity(keys.len());
    let mut group_ids = Vec::with_capacity(keys.len());
    let mut post_ids = Vec::with_capacity(keys.len());
    let mut comment_ids = Vec::with_capacity(keys.len());

    for key in keys {
        vk_user_ids.push(key.vk_user_id);
        group_ids.push(key.group_id);
        post_ids.push(key.post_id);
        comment_ids.push(key.comment_id);
    }

    let deleted = sqlx::query!(
        r#"
        DELETE FROM vk_comment_likes AS l
        USING UNNEST(
            $2::bigint[],
            $3::bigint[],
            $4::bigint[],
            $5::bigint[]
        ) AS src(vk_user_id, group_id, post_id, comment_id)
        WHERE l.user_id = $1
          AND l.vk_user_id = src.vk_user_id
          AND l.group_id = src.group_id
          AND l.post_id = src.post_id
          AND l.comment_id = src.comment_id
        "#,
        user_id,
        &vk_user_ids,
        &group_ids,
        &post_ids,
        &comment_ids
    )
    .execute(db)
    .await?;

    Ok(deleted.rows_affected() as i64)
}

#[allow(dead_code)]
pub async fn list_vk_comment_likes(
    db: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<VkCommentLike>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            user_id,
            vk_user_id,
            group_id,
            post_id,
            comment_id,
            found_date
        FROM vk_comment_likes
        WHERE user_id = $1
        ORDER BY found_date DESC, comment_id DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| VkCommentLike {
            user_id: row.user_id,
            vk_user_id: row.vk_user_id,
            group_id: row.group_id,
            post_id: row.post_id,
            comment_id: row.comment_id,
            found_date: row.found_date,
        })
        .collect())
}
