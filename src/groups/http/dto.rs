use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, ToSchema)]
pub struct CreateGroupRequest {
    pub group_id: i64,
    pub group_name: Option<String>,
    pub screen_name: Option<String>,
    pub is_closed: Option<i32>,
    pub public_type: Option<String>,
    pub photo_200: Option<String>,
    pub description: Option<String>,
    pub members_count: Option<i32>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GroupsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct GroupDto {
    pub group_id: i64,
    pub group_name: Option<String>,
    pub screen_name: Option<String>,
    pub is_closed: Option<i32>,
    pub public_type: Option<String>,
    pub photo_200: Option<String>,
    pub description: Option<String>,
    pub members_count: Option<i32>,
}
