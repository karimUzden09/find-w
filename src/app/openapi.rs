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
        crate::notes::http::handlers::create_note,
        crate::notes::http::handlers::list_notes,
        crate::notes::http::handlers::get_note,
        crate::notes::http::handlers::delete_note
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
        crate::notes::http::CreateNoteRequest,
        crate::notes::http::NoteDto
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "Core", description = "Service and account endpoints"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Notes", description = "Notes endpoints")
    )
)]
pub struct ApiDoc;
