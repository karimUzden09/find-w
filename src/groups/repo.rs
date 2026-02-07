use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Group {
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub group_id: i64,
    pub group_name: Option<String>,
    pub screen_name: Option<String>,
    pub is_closed: Option<i32>,
    pub public_type: Option<String>,
    pub photo_200: Option<String>,
    pub description: Option<String>,
    pub members_count: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct NewGroup {
    pub group_id: i64,
    pub group_name: Option<String>,
    pub screen_name: Option<String>,
    pub is_closed: Option<i32>,
    pub public_type: Option<String>,
    pub photo_200: Option<String>,
    pub description: Option<String>,
    pub members_count: Option<i32>,
}

pub async fn save_group(db: &PgPool, user_id: Uuid, group: NewGroup) -> Result<Group, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO groups (
            user_id,
            group_id,
            group_name,
            screen_name,
            is_closed,
            public_type,
            photo_200,
            description,
            members_count
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (user_id, group_id)
        DO UPDATE SET
            group_name = EXCLUDED.group_name,
            screen_name = EXCLUDED.screen_name,
            is_closed = EXCLUDED.is_closed,
            public_type = EXCLUDED.public_type,
            photo_200 = EXCLUDED.photo_200,
            description = EXCLUDED.description,
            members_count = EXCLUDED.members_count
        RETURNING
            user_id,
            group_id,
            group_name,
            screen_name,
            is_closed,
            public_type,
            photo_200,
            description,
            members_count
        "#,
        user_id,
        group.group_id,
        group.group_name,
        group.screen_name,
        group.is_closed,
        group.public_type,
        group.photo_200,
        group.description,
        group.members_count
    )
    .fetch_one(db)
    .await?;

    Ok(Group {
        user_id: row.user_id,
        group_id: row.group_id,
        group_name: row.group_name,
        screen_name: row.screen_name,
        is_closed: row.is_closed,
        public_type: row.public_type,
        photo_200: row.photo_200,
        description: row.description,
        members_count: row.members_count,
    })
}

pub async fn list_groups(
    db: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Group>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            user_id,
            group_id,
            group_name,
            screen_name,
            is_closed,
            public_type,
            photo_200,
            description,
            members_count
        FROM groups
        WHERE user_id = $1
        ORDER BY group_id DESC
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
        .map(|row| Group {
            user_id: row.user_id,
            group_id: row.group_id,
            group_name: row.group_name,
            screen_name: row.screen_name,
            is_closed: row.is_closed,
            public_type: row.public_type,
            photo_200: row.photo_200,
            description: row.description,
            members_count: row.members_count,
        })
        .collect())
}

pub async fn delete_group_owned(
    db: &PgPool,
    user_id: Uuid,
    group_id: i64,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM groups
        WHERE user_id = $1 AND group_id = $2
        "#,
        user_id,
        group_id
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected() == 1)
}
