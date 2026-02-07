use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UpsertVkUsersResult {
    pub inserted: i64,
    pub updated: i64,
}

#[derive(Debug, Clone)]
pub struct NewVkUser {
    pub vk_user_id: i64,
    pub sex: Option<i16>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub city: Option<String>,
    pub finded_date: OffsetDateTime,
    pub is_closed: Option<bool>,
    pub screen_name: Option<String>,
    pub can_access_closed: Option<bool>,
    pub about: Option<String>,
    pub status: Option<String>,
    pub bdate: Option<String>,
    pub photo: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VkUser {
    pub user_id: Uuid,
    pub vk_user_id: i64,
    pub sex: Option<i16>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub city: Option<String>,
    pub finded_date: OffsetDateTime,
    pub is_closed: Option<bool>,
    pub screen_name: Option<String>,
    pub can_access_closed: Option<bool>,
    pub about: Option<String>,
    pub status: Option<String>,
    pub bdate: Option<String>,
    pub photo: Option<String>,
}

pub async fn upsert_vk_users(
    db: &PgPool,
    user_id: Uuid,
    vk_users: &[NewVkUser],
) -> Result<UpsertVkUsersResult, sqlx::Error> {
    if vk_users.is_empty() {
        return Ok(UpsertVkUsersResult {
            inserted: 0,
            updated: 0,
        });
    }

    let mut vk_user_ids = Vec::with_capacity(vk_users.len());
    let mut sexes = Vec::with_capacity(vk_users.len());
    let mut first_names = Vec::with_capacity(vk_users.len());
    let mut last_names = Vec::with_capacity(vk_users.len());
    let mut cities = Vec::with_capacity(vk_users.len());
    let mut finded_dates = Vec::with_capacity(vk_users.len());
    let mut is_closed_values = Vec::with_capacity(vk_users.len());
    let mut screen_names = Vec::with_capacity(vk_users.len());
    let mut can_access_closed_values = Vec::with_capacity(vk_users.len());
    let mut about_values = Vec::with_capacity(vk_users.len());
    let mut status_values = Vec::with_capacity(vk_users.len());
    let mut bdates = Vec::with_capacity(vk_users.len());
    let mut photos = Vec::with_capacity(vk_users.len());

    for vk_user in vk_users {
        vk_user_ids.push(vk_user.vk_user_id);
        sexes.push(vk_user.sex);
        first_names.push(vk_user.first_name.clone());
        last_names.push(vk_user.last_name.clone());
        cities.push(vk_user.city.clone());
        finded_dates.push(vk_user.finded_date);
        is_closed_values.push(vk_user.is_closed);
        screen_names.push(vk_user.screen_name.clone());
        can_access_closed_values.push(vk_user.can_access_closed);
        about_values.push(vk_user.about.clone());
        status_values.push(vk_user.status.clone());
        bdates.push(vk_user.bdate.clone());
        photos.push(vk_user.photo.clone());
    }

    let rows = sqlx::query!(
        r#"
        INSERT INTO vk_users (
            user_id,
            vk_user_id,
            sex,
            first_name,
            last_name,
            city,
            finded_date,
            is_closed,
            screen_name,
            can_access_closed,
            about,
            status,
            bdate,
            photo
        )
        SELECT
            $1::uuid,
            src.vk_user_id,
            src.sex,
            src.first_name,
            src.last_name,
            src.city,
            src.finded_date,
            src.is_closed,
            src.screen_name,
            src.can_access_closed,
            src.about,
            src.status,
            src.bdate,
            src.photo
        FROM UNNEST(
            $2::bigint[],
            $3::smallint[],
            $4::text[],
            $5::text[],
            $6::text[],
            $7::timestamptz[],
            $8::boolean[],
            $9::text[],
            $10::boolean[],
            $11::text[],
            $12::text[],
            $13::text[],
            $14::text[]
        ) AS src(
            vk_user_id,
            sex,
            first_name,
            last_name,
            city,
            finded_date,
            is_closed,
            screen_name,
            can_access_closed,
            about,
            status,
            bdate,
            photo
        )
        ON CONFLICT (user_id, vk_user_id)
        DO UPDATE SET
            sex = EXCLUDED.sex,
            first_name = EXCLUDED.first_name,
            last_name = EXCLUDED.last_name,
            city = EXCLUDED.city,
            finded_date = EXCLUDED.finded_date,
            is_closed = EXCLUDED.is_closed,
            screen_name = EXCLUDED.screen_name,
            can_access_closed = EXCLUDED.can_access_closed,
            about = EXCLUDED.about,
            status = EXCLUDED.status,
            bdate = EXCLUDED.bdate,
            photo = EXCLUDED.photo
        RETURNING (xmax = 0) AS "inserted!"
        "#,
        user_id,
        &vk_user_ids,
        &sexes as &[Option<i16>],
        &first_names as &[Option<String>],
        &last_names as &[Option<String>],
        &cities as &[Option<String>],
        &finded_dates,
        &is_closed_values as &[Option<bool>],
        &screen_names as &[Option<String>],
        &can_access_closed_values as &[Option<bool>],
        &about_values as &[Option<String>],
        &status_values as &[Option<String>],
        &bdates as &[Option<String>],
        &photos as &[Option<String>]
    )
    .fetch_all(db)
    .await?;

    let inserted = rows.iter().filter(|row| row.inserted).count() as i64;
    let updated = rows.len() as i64 - inserted;

    Ok(UpsertVkUsersResult { inserted, updated })
}

pub async fn delete_vk_users(
    db: &PgPool,
    user_id: Uuid,
    vk_user_ids: &[i64],
) -> Result<i64, sqlx::Error> {
    if vk_user_ids.is_empty() {
        return Ok(0);
    }

    let deleted = sqlx::query!(
        r#"
        DELETE FROM vk_users
        WHERE user_id = $1 AND vk_user_id = ANY($2::bigint[])
        "#,
        user_id,
        vk_user_ids
    )
    .execute(db)
    .await?;

    Ok(deleted.rows_affected() as i64)
}

pub async fn list_vk_users(
    db: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<VkUser>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            user_id,
            vk_user_id,
            sex,
            first_name,
            last_name,
            city,
            finded_date,
            is_closed,
            screen_name,
            can_access_closed,
            about,
            status,
            bdate,
            photo
        FROM vk_users
        WHERE user_id = $1
        ORDER BY finded_date DESC, vk_user_id DESC
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
        .map(|row| VkUser {
            user_id: row.user_id,
            vk_user_id: row.vk_user_id,
            sex: row.sex,
            first_name: row.first_name,
            last_name: row.last_name,
            city: row.city,
            finded_date: row.finded_date,
            is_closed: row.is_closed,
            screen_name: row.screen_name,
            can_access_closed: row.can_access_closed,
            about: row.about,
            status: row.status,
            bdate: row.bdate,
            photo: row.photo,
        })
        .collect())
}
