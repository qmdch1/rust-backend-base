use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};

use crate::auth::jwt::{verify_token, Claims};
use crate::config::JwtConfig;
use crate::errors::AppError;

pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let jwt_config = request
        .extensions()
        .get::<JwtConfig>()
        .cloned()
        .ok_or(AppError::InternalServerError)?;

    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = verify_token(&jwt_config, token)?;

    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

// Extractor for getting current user claims from request
#[allow(dead_code)]
pub async fn require_role(
    claims: &Claims,
    required_role: &str,
) -> Result<(), AppError> {
    if claims.role != required_role && claims.role != "admin" {
        return Err(AppError::Forbidden);
    }
    Ok(())
}
