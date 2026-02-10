use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::core::http::handlers::health,
        crate::core::http::handlers::db_health,
        crate::core::http::handlers::me,
        crate::auth::http::handlers::register,
        crate::auth::http::handlers::login,
        crate::auth::http::handlers::refresh,
        crate::auth::http::handlers::logout,
        crate::user_settings::http::handlers::get_user_settings,
        crate::user_settings::http::handlers::update_user_settings,
        crate::notes::http::handlers::create_note,
        crate::notes::http::handlers::list_notes,
        crate::notes::http::handlers::get_note,
        crate::notes::http::handlers::delete_note,
        crate::groups::http::handlers::create_group,
        crate::groups::http::handlers::list_groups,
        crate::groups::http::handlers::delete_group,
        crate::vk_users::http::handlers::list_vk_users,
        crate::vk_tokens::http::handlers::add_vk_tokens,
        crate::vk_tokens::http::handlers::delete_vk_tokens
    ),
    components(schemas(
        crate::error::ErrorBody,
        crate::core::http::MeResponse,
        crate::auth::http::RegisterRequest,
        crate::auth::http::RegisterResponse,
        crate::auth::http::LoginRequest,
        crate::auth::http::LoginResponse,
        crate::auth::http::RefreshRequest,
        crate::auth::http::RefreshResponse,
        crate::auth::http::LogoutRequest,
        crate::user_settings::http::UserSettingsDto,
        crate::user_settings::http::UpdateUserSettingsRequest,
        crate::notes::http::CreateNoteRequest,
        crate::notes::http::NoteDto,
        crate::groups::http::CreateGroupRequest,
        crate::groups::http::GroupDto,
        crate::vk_users::http::VkUserDto,
        crate::vk_tokens::http::AddVkTokensRequest,
        crate::vk_tokens::http::AddVkTokensResponse,
        crate::vk_tokens::http::DeleteVkTokensRequest,
        crate::vk_tokens::http::DeleteVkTokensResponse
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "Core", description = "Service and account endpoints"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Settings", description = "Current user settings endpoints"),
        (name = "Notes", description = "Notes endpoints"),
        (name = "Groups", description = "User groups endpoints"),
        (name = "VK Users", description = "VK users management endpoints"),
        (name = "VK Tokens", description = "VK tokens management endpoints")
    )
)]
pub struct ApiDoc;
