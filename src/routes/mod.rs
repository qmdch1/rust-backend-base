pub mod handlers;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::middleware::auth::auth_middleware;
use crate::routes::handlers::{auth_handler, health_handler, hello_handler, user_handler};

use crate::AppState;

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

    Router::new()
        .nest("/api/v1", public_routes.merge(protected_routes))
        .with_state(state)
}
