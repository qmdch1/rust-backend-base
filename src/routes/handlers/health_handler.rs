use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::AppState;

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service health status", body = Value)
    )
))]
pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let db_healthy = if let Some(ref db) = state.db {
        sqlx::query("SELECT 1").execute(db).await.is_ok()
    } else {
        false
    };

    let redis_healthy = if let Some(ref redis) = state.redis {
        redis::cmd("PING")
            .query_async::<String>(&mut redis.clone())
            .await
            .is_ok()
    } else {
        false
    };

    let db_status = match (&state.db, db_healthy) {
        (None, _) => "not configured",
        (Some(_), true) => "up",
        (Some(_), false) => "down",
    };

    let redis_status = match (&state.redis, redis_healthy) {
        (None, _) => "not configured",
        (Some(_), true) => "up",
        (Some(_), false) => "down",
    };

    let all_healthy = (state.db.is_none() || db_healthy) && (state.redis.is_none() || redis_healthy);

    Json(json!({
        "status": if all_healthy { "healthy" } else { "degraded" },
        "version": env!("CARGO_PKG_VERSION"),
        "services": {
            "database": db_status,
            "redis": redis_status,
        }
    }))
}
