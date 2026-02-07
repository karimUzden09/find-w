use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UpsertVkPostsResult {
    pub inserted: i64,
    pub updated: i64,
}

#[derive(Debug, Clone)]
pub struct NewVkPost {
    pub post_id: i64,
    pub group_id: i64,
    pub from_id: i64,
    pub created_date: i64,
    pub post_type: Option<String>,
    pub post_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VkPostKey {
    pub group_id: i64,
    pub post_id: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VkPost {
    pub user_id: Uuid,
    pub post_id: i64,
    pub group_id: i64,
    pub from_id: i64,
    pub created_date: i64,
    pub post_type: Option<String>,
    pub post_text: Option<String>,
}

pub async fn upsert_vk_posts(
    db: &PgPool,
    user_id: Uuid,
    posts: &[NewVkPost],
) -> Result<UpsertVkPostsResult, sqlx::Error> {
    if posts.is_empty() {
        return Ok(UpsertVkPostsResult {
            inserted: 0,
            updated: 0,
        });
    }

    let mut post_ids = Vec::with_capacity(posts.len());
    let mut group_ids = Vec::with_capacity(posts.len());
    let mut from_ids = Vec::with_capacity(posts.len());
    let mut created_dates = Vec::with_capacity(posts.len());
    let mut post_types = Vec::with_capacity(posts.len());
    let mut post_texts = Vec::with_capacity(posts.len());

    for post in posts {
        post_ids.push(post.post_id);
        group_ids.push(post.group_id);
        from_ids.push(post.from_id);
        created_dates.push(post.created_date);
        post_types.push(post.post_type.clone());
        post_texts.push(post.post_text.clone());
    }

    let rows = sqlx::query!(
        r#"
        INSERT INTO vk_posts (
            user_id,
            post_id,
            group_id,
            from_id,
            created_date,
            post_type,
            post_text
        )
        SELECT
            $1::uuid,
            src.post_id,
            src.group_id,
            src.from_id,
            src.created_date,
            src.post_type,
            src.post_text
        FROM UNNEST(
            $2::bigint[],
            $3::bigint[],
            $4::bigint[],
            $5::bigint[],
            $6::text[],
            $7::text[]
        ) AS src(
            post_id,
            group_id,
            from_id,
            created_date,
            post_type,
            post_text
        )
        ON CONFLICT (user_id, group_id, post_id)
        DO UPDATE SET
            from_id = EXCLUDED.from_id,
            created_date = EXCLUDED.created_date,
            post_type = EXCLUDED.post_type,
            post_text = EXCLUDED.post_text
        RETURNING (xmax = 0) AS "inserted!"
        "#,
        user_id,
        &post_ids,
        &group_ids,
        &from_ids,
        &created_dates,
        &post_types as &[Option<String>],
        &post_texts as &[Option<String>]
    )
    .fetch_all(db)
    .await?;

    let inserted = rows.iter().filter(|row| row.inserted).count() as i64;
    let updated = rows.len() as i64 - inserted;

    Ok(UpsertVkPostsResult { inserted, updated })
}

pub async fn delete_vk_posts(
    db: &PgPool,
    user_id: Uuid,
    keys: &[VkPostKey],
) -> Result<i64, sqlx::Error> {
    if keys.is_empty() {
        return Ok(0);
    }

    let mut group_ids = Vec::with_capacity(keys.len());
    let mut post_ids = Vec::with_capacity(keys.len());

    for key in keys {
        group_ids.push(key.group_id);
        post_ids.push(key.post_id);
    }

    let deleted = sqlx::query!(
        r#"
        DELETE FROM vk_posts AS p
        USING UNNEST($2::bigint[], $3::bigint[]) AS src(group_id, post_id)
        WHERE p.user_id = $1
          AND p.group_id = src.group_id
          AND p.post_id = src.post_id
        "#,
        user_id,
        &group_ids,
        &post_ids
    )
    .execute(db)
    .await?;

    Ok(deleted.rows_affected() as i64)
}

#[allow(dead_code)]
pub async fn list_vk_posts(
    db: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<VkPost>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            user_id,
            post_id,
            group_id,
            from_id,
            created_date,
            post_type,
            post_text
        FROM vk_posts
        WHERE user_id = $1
        ORDER BY created_date DESC, post_id DESC
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
        .map(|row| VkPost {
            user_id: row.user_id,
            post_id: row.post_id,
            group_id: row.group_id,
            from_id: row.from_id,
            created_date: row.created_date,
            post_type: row.post_type,
            post_text: row.post_text,
        })
        .collect())
}
