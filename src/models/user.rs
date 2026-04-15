use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "swagger")]
use utoipa::ToSchema;

// Database model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// API response (password_hash excluded)
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

// Registration request
#[derive(Debug, Deserialize, Validate)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email address"))]
    #[cfg_attr(feature = "swagger", schema(example = "user@example.com"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    #[cfg_attr(feature = "swagger", schema(example = "password123"))]
    pub password: String,

    #[validate(length(min = 1, max = 100, message = "Name is required"))]
    #[cfg_attr(feature = "swagger", schema(example = "John"))]
    pub name: String,
}

// Login request
#[derive(Debug, Deserialize, Validate)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct LoginRequest {
    #[validate(email)]
    #[cfg_attr(feature = "swagger", schema(example = "user@example.com"))]
    pub email: String,

    #[validate(length(min = 1))]
    #[cfg_attr(feature = "swagger", schema(example = "password123"))]
    pub password: String,
}

// Auth response
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub user: UserResponse,
}

// Refresh token request
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct RefreshRequest {
    pub refresh_token: String,
}

// Token refresh response
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
}

// Update user request
#[derive(Debug, Deserialize, Validate)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100))]
    #[cfg_attr(feature = "swagger", schema(example = "Jane"))]
    pub name: Option<String>,

    #[validate(email)]
    #[cfg_attr(feature = "swagger", schema(example = "new@example.com"))]
    pub email: Option<String>,
}

// Generic message response
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct MessageResponse {
    pub message: String,
}

// Error response
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
pub struct ErrorDetail {
    pub status: u16,
    pub message: String,
}
