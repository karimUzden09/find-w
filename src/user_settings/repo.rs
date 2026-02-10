use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserSettings {
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub search_interval_minutes: i32,
    pub updated_at: OffsetDateTime,
}

pub async fn get_user_settings(db: &PgPool, user_id: Uuid) -> Result<UserSettings, sqlx::Error> {
    let _ = sqlx::query!(
        r#"
        INSERT INTO user_settings (user_id)
        VALUES ($1)
        ON CONFLICT (user_id) DO NOTHING
        "#,
        user_id
    )
    .execute(db)
    .await?;

    let row = sqlx::query!(
        r#"
        SELECT user_id, search_interval_minutes, updated_at
        FROM user_settings
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(db)
    .await?;

    Ok(UserSettings {
        user_id: row.user_id,
        search_interval_minutes: row.search_interval_minutes,
        updated_at: row.updated_at,
    })
}

pub async fn update_user_settings(
    db: &PgPool,
    user_id: Uuid,
    search_interval_minutes: i32,
) -> Result<UserSettings, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO user_settings (user_id, search_interval_minutes)
        VALUES ($1, $2)
        ON CONFLICT (user_id)
        DO UPDATE SET
            search_interval_minutes = EXCLUDED.search_interval_minutes,
            updated_at = now()
        RETURNING user_id, search_interval_minutes, updated_at
        "#,
        user_id,
        search_interval_minutes
    )
    .fetch_one(db)
    .await?;

    Ok(UserSettings {
        user_id: row.user_id,
        search_interval_minutes: row.search_interval_minutes,
        updated_at: row.updated_at,
    })
}
