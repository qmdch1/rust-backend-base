use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::JwtConfig;
use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,        // user id
    pub email: String,
    pub role: String,
    pub exp: i64,         // expiration timestamp
    pub iat: i64,         // issued at
}

pub fn generate_access_token(
    config: &JwtConfig,
    user_id: Uuid,
    email: &str,
    role: &str,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        exp: now + config.access_token_expiry_secs,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| AppError::Anyhow(anyhow::anyhow!("JWT encoding error: {}", e)))
}

pub fn generate_refresh_token(
    config: &JwtConfig,
    user_id: Uuid,
    email: &str,
    role: &str,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        exp: now + config.refresh_token_expiry_secs,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .map_err(|e| AppError::Anyhow(anyhow::anyhow!("JWT encoding error: {}", e)))
}

pub fn verify_token(config: &JwtConfig, token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}
