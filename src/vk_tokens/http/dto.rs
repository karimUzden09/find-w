use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct AddVkTokensRequest {
    pub tokens: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AddVkTokensResponse {
    pub inserted: i64,
    pub skipped: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct DeleteVkTokensRequest {
    pub tokens: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteVkTokensResponse {
    pub deleted: i64,
}
