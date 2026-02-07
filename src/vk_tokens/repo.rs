use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AddVkTokensResult {
    pub inserted: i64,
    pub skipped: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserVkToken {
    pub id: Uuid,
    pub token: String,
    pub created_at: OffsetDateTime,
}

fn hash_vk_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex::encode(digest)
}

pub async fn add_vk_tokens(
    db: &PgPool,
    user_id: Uuid,
    tokens: &[String],
    encryption_key: &str,
) -> Result<AddVkTokensResult, sqlx::Error> {
    let mut tx = db.begin().await?;
    let mut inserted = 0_i64;

    for token in tokens {
        let token_hash = hash_vk_token(token);
        let row = sqlx::query!(
            r#"
            INSERT INTO vk_tokens (user_id, token_hash, token_encrypted)
            VALUES ($1, $2, pgp_sym_encrypt($3, $4, 'cipher-algo=aes256,compress-algo=1'))
            ON CONFLICT (user_id, token_hash)
            DO NOTHING
            RETURNING id
            "#,
            user_id,
            token_hash,
            token,
            encryption_key
        )
        .fetch_optional(&mut *tx)
        .await?;

        if row.is_some() {
            inserted += 1;
        }
    }

    tx.commit().await?;

    Ok(AddVkTokensResult {
        inserted,
        skipped: tokens.len() as i64 - inserted,
    })
}

pub async fn delete_vk_tokens(
    db: &PgPool,
    user_id: Uuid,
    tokens: &[String],
) -> Result<i64, sqlx::Error> {
    let token_hashes: Vec<String> = tokens.iter().map(|token| hash_vk_token(token)).collect();

    let deleted = sqlx::query!(
        r#"
        DELETE FROM vk_tokens
        WHERE user_id = $1 AND token_hash = ANY($2)
        "#,
        user_id,
        &token_hashes
    )
    .execute(db)
    .await?;

    Ok(deleted.rows_affected() as i64)
}

#[allow(dead_code)]
pub async fn list_vk_tokens_for_user(
    db: &PgPool,
    user_id: Uuid,
    encryption_key: &str,
) -> Result<Vec<UserVkToken>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            pgp_sym_decrypt(token_encrypted, $2) AS "token!",
            created_at
        FROM vk_tokens
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id,
        encryption_key
    )
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| UserVkToken {
            id: row.id,
            token: row.token,
            created_at: row.created_at,
        })
        .collect())
}
