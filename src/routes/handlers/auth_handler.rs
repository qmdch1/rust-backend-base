use axum::{extract::State, Json};
use validator::Validate;

use crate::auth::jwt;
use crate::auth::password;
use crate::errors::{AppError, AppResult};
use crate::models::user::{AuthResponse, CreateUserRequest, LoginRequest, UserResponse};
use crate::services::UserService;
use crate::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = UserService::create_user(&state.db, &req).await?;

    let access_token = jwt::generate_access_token(&state.config.jwt, user.id, &user.email, &user.role)?;
    let refresh_token = jwt::generate_refresh_token(&state.config.jwt, user.id, &user.email, &user.role)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        user: UserResponse::from(user),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = UserService::find_by_email(&state.db, &req.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !user.is_active {
        return Err(AppError::Unauthorized);
    }

    let valid = password::verify_password(&req.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::Unauthorized);
    }

    let access_token = jwt::generate_access_token(&state.config.jwt, user.id, &user.email, &user.role)?;
    let refresh_token = jwt::generate_refresh_token(&state.config.jwt, user.id, &user.email, &user.role)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        user: UserResponse::from(user),
    }))
}

#[derive(serde::Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let claims = jwt::verify_token(&state.config.jwt, &req.refresh_token)?;

    let access_token = jwt::generate_access_token(
        &state.config.jwt,
        claims.sub,
        &claims.email,
        &claims.role,
    )?;

    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "token_type": "Bearer"
    })))
}
