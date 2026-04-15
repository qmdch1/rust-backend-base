use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

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
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[validate(length(min = 1, max = 100, message = "Name is required"))]
    pub name: String,
}

// Login request
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1))]
    pub password: String,
}

// Auth response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub user: UserResponse,
}

// Update user request
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    #[validate(email)]
    pub email: Option<String>,
}
