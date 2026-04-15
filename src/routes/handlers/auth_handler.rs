use axum::{extract::State, Json};
use validator::Validate;

use crate::auth::jwt;
use crate::auth::password;
use crate::errors::{AppError, AppResult};
#[cfg(feature = "swagger")]
use crate::models::user::ErrorResponse;
use crate::models::user::{
    AuthResponse, CreateUserRequest, LoginRequest, RefreshRequest, TokenResponse, UserResponse,
};
use crate::services::UserService;
use crate::AppState;

#[cfg_attr(feature = "swagger", utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "Auth",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "Registration successful", body = AuthResponse),
        (status = 409, description = "Email already registered", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
    )
))]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let db = state.db.as_ref().ok_or(AppError::InternalServerError)?;
    let user = UserService::create_user(db, &req).await?;

    let access_token =
        jwt::generate_access_token(&state.config.jwt, user.id, &user.email, &user.role)?;
    let refresh_token =
        jwt::generate_refresh_token(&state.config.jwt, user.id, &user.email, &user.role)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        user: UserResponse::from(user),
    }))
}

#[cfg_attr(feature = "swagger", utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
    )
))]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let db = state.db.as_ref().ok_or(AppError::InternalServerError)?;
    let user = UserService::find_by_email(db, &req.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !user.is_active {
        return Err(AppError::Unauthorized);
    }

    let valid = password::verify_password(&req.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    let access_token =
        jwt::generate_access_token(&state.config.jwt, user.id, &user.email, &user.role)?;
    let refresh_token =
        jwt::generate_refresh_token(&state.config.jwt, user.id, &user.email, &user.role)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        user: UserResponse::from(user),
    }))
}

#[cfg_attr(feature = "swagger", utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "Auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed", body = TokenResponse),
        (status = 401, description = "Invalid refresh token", body = ErrorResponse),
    )
))]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<TokenResponse>> {
    let claims = jwt::verify_token(&state.config.jwt, &req.refresh_token)?;

    let access_token = jwt::generate_access_token(
        &state.config.jwt,
        claims.sub,
        &claims.email,
        &claims.role,
    )?;

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
    }))
}
