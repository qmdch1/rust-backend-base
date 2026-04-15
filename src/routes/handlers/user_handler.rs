use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::auth::jwt::Claims;
use crate::errors::{AppError, AppResult};
use crate::models::user::{UpdateUserRequest, UserResponse};
use crate::services::UserService;
use crate::AppState;

pub async fn get_me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<UserResponse>> {
    let user = UserService::find_by_id(&state.db, claims.sub).await?;
    Ok(Json(UserResponse::from(user)))
}

pub async fn update_me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = UserService::update_user(&state.db, claims.sub, &req).await?;
    Ok(Json(UserResponse::from(user)))
}

#[derive(serde::Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<Vec<UserResponse>>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let users = UserService::list_users(&state.db, limit, offset).await?;
    let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(response))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let user = UserService::find_by_id(&state.db, id).await?;
    Ok(Json(UserResponse::from(user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Only admin or self can delete
    if claims.role != "admin" && claims.sub != id {
        return Err(AppError::Forbidden);
    }

    UserService::delete_user(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "message": "User deleted successfully" })))
}
