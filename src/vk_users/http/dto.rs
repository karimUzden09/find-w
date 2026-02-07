use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct VkUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct VkUserDto {
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
