use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UpsertVkCommentsResult {
    pub inserted: i64,
    pub updated: i64,
}

#[derive(Debug, Clone)]
pub struct NewVkComment {
    pub group_id: i64,
    pub post_id: i64,
    pub comment_id: i64,
    pub from_id: i64,
    pub created_date: i64,
    pub comment_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VkCommentKey {
    pub group_id: i64,
    pub post_id: i64,
    pub comment_id: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VkComment {
    pub user_id: Uuid,
    pub group_id: i64,
    pub post_id: i64,
    pub comment_id: i64,
    pub from_id: i64,
    pub created_date: i64,
    pub comment_text: Option<String>,
}

pub async fn upsert_vk_comments(
    db: &PgPool,
    user_id: Uuid,
    comments: &[NewVkComment],
) -> Result<UpsertVkCommentsResult, sqlx::Error> {
    if comments.is_empty() {
        return Ok(UpsertVkCommentsResult {
            inserted: 0,
            updated: 0,
        });
    }

    let mut group_ids = Vec::with_capacity(comments.len());
    let mut post_ids = Vec::with_capacity(comments.len());
    let mut comment_ids = Vec::with_capacity(comments.len());
    let mut from_ids = Vec::with_capacity(comments.len());
    let mut created_dates = Vec::with_capacity(comments.len());
    let mut comment_texts = Vec::with_capacity(comments.len());

    for comment in comments {
        group_ids.push(comment.group_id);
        post_ids.push(comment.post_id);
        comment_ids.push(comment.comment_id);
        from_ids.push(comment.from_id);
        created_dates.push(comment.created_date);
        comment_texts.push(comment.comment_text.clone());
    }

    let rows = sqlx::query!(
        r#"
        INSERT INTO vk_comments (
            user_id,
            group_id,
            post_id,
            comment_id,
            from_id,
            created_date,
            comment_text
        )
        SELECT
            $1::uuid,
            src.group_id,
            src.post_id,
            src.comment_id,
            src.from_id,
            src.created_date,
            src.comment_text
        FROM UNNEST(
            $2::bigint[],
            $3::bigint[],
            $4::bigint[],
            $5::bigint[],
            $6::bigint[],
            $7::text[]
        ) AS src(group_id, post_id, comment_id, from_id, created_date, comment_text)
        ON CONFLICT (user_id, group_id, post_id, comment_id)
        DO UPDATE SET
            from_id = EXCLUDED.from_id,
            created_date = EXCLUDED.created_date,
            comment_text = EXCLUDED.comment_text
        RETURNING (xmax = 0) AS "inserted!"
        "#,
        user_id,
        &group_ids,
        &post_ids,
        &comment_ids,
        &from_ids,
        &created_dates,
        &comment_texts as &[Option<String>]
    )
    .fetch_all(db)
    .await?;

    let inserted = rows.iter().filter(|row| row.inserted).count() as i64;
    let updated = rows.len() as i64 - inserted;

    Ok(UpsertVkCommentsResult { inserted, updated })
}

pub async fn delete_vk_comments(
    db: &PgPool,
    user_id: Uuid,
    keys: &[VkCommentKey],
) -> Result<i64, sqlx::Error> {
    if keys.is_empty() {
        return Ok(0);
    }

    let mut group_ids = Vec::with_capacity(keys.len());
    let mut post_ids = Vec::with_capacity(keys.len());
    let mut comment_ids = Vec::with_capacity(keys.len());

    for key in keys {
        group_ids.push(key.group_id);
        post_ids.push(key.post_id);
        comment_ids.push(key.comment_id);
    }

    let deleted = sqlx::query!(
        r#"
        DELETE FROM vk_comments AS c
        USING UNNEST($2::bigint[], $3::bigint[], $4::bigint[]) AS src(group_id, post_id, comment_id)
        WHERE c.user_id = $1
          AND c.group_id = src.group_id
          AND c.post_id = src.post_id
          AND c.comment_id = src.comment_id
        "#,
        user_id,
        &group_ids,
        &post_ids,
        &comment_ids
    )
    .execute(db)
    .await?;

    Ok(deleted.rows_affected() as i64)
}

#[allow(dead_code)]
pub async fn list_vk_comments(
    db: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<VkComment>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            user_id,
            group_id,
            post_id,
            comment_id,
            from_id,
            created_date,
            comment_text
        FROM vk_comments
        WHERE user_id = $1
        ORDER BY created_date DESC, comment_id DESC
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
        .map(|row| VkComment {
            user_id: row.user_id,
            group_id: row.group_id,
            post_id: row.post_id,
            comment_id: row.comment_id,
            from_id: row.from_id,
            created_date: row.created_date,
            comment_text: row.comment_text,
        })
        .collect())
}
