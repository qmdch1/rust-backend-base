use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
#[cfg(feature = "swagger")]
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::auth::jwt::Claims;
use crate::errors::{AppError, AppResult};
#[cfg(feature = "swagger")]
use crate::models::user::ErrorResponse;
use crate::models::user::{MessageResponse, UpdateUserRequest, UserResponse};
use crate::services::UserService;
use crate::AppState;

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/api/v1/users/me",
    tag = "Users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user info", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    )
))]
pub async fn get_me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<UserResponse>> {
    let db = state.db.as_ref().ok_or(AppError::InternalServerError)?;
    let user = UserService::find_by_id(db, claims.sub).await?;
    Ok(Json(UserResponse::from(user)))
}

#[cfg_attr(feature = "swagger", utoipa::path(
    put,
    path = "/api/v1/users/me",
    tag = "Users",
    security(("bearer_auth" = [])),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 409, description = "Email already registered", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
    )
))]
pub async fn update_me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let db = state.db.as_ref().ok_or(AppError::InternalServerError)?;
    let user = UserService::update_user(db, claims.sub, &req).await?;
    Ok(Json(UserResponse::from(user)))
}

#[derive(serde::Deserialize)]
#[cfg_attr(feature = "swagger", derive(IntoParams, ToSchema))]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(PaginationParams),
    responses(
        (status = 200, description = "List of users", body = Vec<UserResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    )
))]
pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<Vec<UserResponse>>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let db = state.db.as_ref().ok_or(AppError::InternalServerError)?;
    let users = UserService::list_users(db, limit, offset).await?;
    let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(response))
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    )
))]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let db = state.db.as_ref().ok_or(AppError::InternalServerError)?;
    let user = UserService::find_by_id(db, id).await?;
    Ok(Json(UserResponse::from(user)))
}

#[cfg_attr(feature = "swagger", utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "Users",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 200, description = "User deleted", body = MessageResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    )
))]
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    // Only admin or self can delete
    if claims.role != "admin" && claims.sub != id {
        return Err(AppError::Forbidden);
    }

    let db = state.db.as_ref().ok_or(AppError::InternalServerError)?;
    UserService::delete_user(db, id).await?;
    Ok(Json(MessageResponse {
        message: "User deleted successfully".to_string(),
    }))
}
