use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Note {
    pub id: Uuid,
    #[allow(dead_code)]
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub created_at: OffsetDateTime,
}

pub async fn create_note(
    db: &PgPool,
    user_id: Uuid,
    title: String,
    body: String,
) -> Result<Note, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO notes (user_id, title, body)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, title, body, created_at
        "#,
        user_id,
        title,
        body
    )
    .fetch_one(db)
    .await?;

    Ok(Note {
        id: row.id,
        user_id: row.user_id,
        title: row.title,
        body: row.body,
        created_at: row.created_at,
    })
}

pub async fn get_note_owned(
    db: &PgPool,
    user_id: Uuid,
    note_id: Uuid,
) -> Result<Option<Note>, sqlx::Error> {
    let may_be_recodrd = sqlx::query!(
        r#"
        SELECT id, user_id ,title, body, created_at
        FROM notes
        WHERE id = $1 AND user_id = $2
        "#,
        note_id,
        user_id
    )
    .fetch_optional(db)
    .await?;

    Ok(may_be_recodrd.map(|r| Note {
        id: r.id,
        user_id: r.user_id,
        title: r.title,
        body: r.body,
        created_at: r.created_at,
    }))
}

pub async fn list_notes(
    db: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Note>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, user_id, title, body, created_at
        FROM notes
        WHERE user_id = $1
        ORDER BY created_at DESC
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
        .map(|r| Note {
            id: r.id,
            user_id: r.user_id,
            title: r.title,
            body: r.body,
            created_at: r.created_at,
        })
        .collect())
}

pub async fn delete_note_owned(
    db: &PgPool,
    user_id: Uuid,
    note_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        r#"
        DELETE FROM notes
        WHERE id = $1 AND user_id = $2
        "#,
        note_id,
        user_id
    )
    .execute(db)
    .await?;

    Ok(res.rows_affected() == 1)
}
