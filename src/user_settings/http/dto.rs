use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct UserSettingsDto {
    pub search_interval_minutes: i32,
    pub updated_at: OffsetDateTime,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserSettingsRequest {
    pub search_interval_minutes: Option<i32>,
}
