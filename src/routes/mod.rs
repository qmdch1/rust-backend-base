pub mod handlers;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::middleware::auth::auth_middleware;
use crate::routes::handlers::{auth_handler, health_handler, hello_handler, user_handler};

use crate::AppState;

#[cfg(feature = "swagger")]
mod swagger {
    use crate::models::user::{
        AuthResponse, CreateUserRequest, ErrorDetail, ErrorResponse, LoginRequest, MessageResponse,
        RefreshRequest, TokenResponse, UpdateUserRequest, UserResponse,
    };
    use crate::routes::handlers::{auth_handler, health_handler, hello_handler, user_handler};
    use utoipa::OpenApi;

    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "Rust Backend Base API",
            description = "Production-ready Rust backend API with JWT authentication",
            version = "0.1.0"
        ),
        paths(
            hello_handler::hello_world,
            health_handler::health_check,
            auth_handler::register,
            auth_handler::login,
            auth_handler::refresh_token,
            user_handler::get_me,
            user_handler::update_me,
            user_handler::list_users,
            user_handler::get_user,
            user_handler::delete_user,
        ),
        components(
            schemas(
                CreateUserRequest, LoginRequest, RefreshRequest,
                AuthResponse, TokenResponse, UserResponse,
                UpdateUserRequest, MessageResponse,
                ErrorResponse, ErrorDetail,
            )
        ),
        security(
            ("bearer_auth" = [])
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "Health", description = "Health check endpoints"),
            (name = "Auth", description = "Authentication endpoints"),
            (name = "Users", description = "User management endpoints"),
        )
    )]
    pub struct ApiDoc;

    pub struct SecurityAddon;

    impl utoipa::Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "bearer_auth",
                    utoipa::openapi::security::SecurityScheme::Http(
                        utoipa::openapi::security::HttpBuilder::new()
                            .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                            .bearer_format("JWT")
                            .build(),
                    ),
                );
            }
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/hello", get(hello_handler::hello_world))
        .route("/health", get(health_handler::health_check))
        .route("/auth/register", post(auth_handler::register))
        .route("/auth/login", post(auth_handler::login))
        .route("/auth/refresh", post(auth_handler::refresh_token));

    let protected_routes = Router::new()
        .route("/users/me", get(user_handler::get_me))
        .route("/users/me", put(user_handler::update_me))
        .route("/users", get(user_handler::list_users))
        .route("/users/{id}", get(user_handler::get_user))
        .route("/users/{id}", delete(user_handler::delete_user))
        .layer(middleware::from_fn(auth_middleware));

    #[allow(unused_mut)]
    let mut router = Router::new()
        .nest("/api/v1", public_routes.merge(protected_routes))
        .with_state(state);

    #[cfg(feature = "swagger")]
    {
        use utoipa::OpenApi;
        use utoipa_swagger_ui::SwaggerUi;
        router = router.merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", swagger::ApiDoc::openapi()),
        );
    }

    router
}
