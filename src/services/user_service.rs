use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::user::{CreateUserRequest, UpdateUserRequest, User};
use crate::auth::password;

pub struct UserService;

impl UserService {
    pub async fn create_user(pool: &PgPool, req: &CreateUserRequest) -> AppResult<User> {
        // Check if email already exists
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
        )
        .bind(&req.email)
        .fetch_one(pool)
        .await?;

        if exists {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        let password_hash = password::hash_password(&req.password)?;
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'user', true, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.name)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn update_user(pool: &PgPool, id: Uuid, req: &UpdateUserRequest) -> AppResult<User> {
        let current = Self::find_by_id(pool, id).await?;

        // Check email uniqueness if changing
        if let Some(ref new_email) = req.email
            && new_email != &current.email
        {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2)",
            )
            .bind(new_email)
            .bind(id)
            .fetch_one(pool)
            .await?;

            if exists {
                return Err(AppError::Conflict("Email already registered".to_string()));
            }
        }

        let name = req.name.as_deref().unwrap_or(&current.name);
        let email = req.email.as_deref().unwrap_or(&current.email);

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users SET name = $1, email = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(email)
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn list_users(pool: &PgPool, limit: i64, offset: i64) -> AppResult<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn delete_user(pool: &PgPool, id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        Ok(())
    }
}
