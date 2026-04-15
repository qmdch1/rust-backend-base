use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let db_healthy = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    let redis_healthy = redis::cmd("PING")
        .query_async::<String>(&mut state.redis.clone())
        .await
        .is_ok();

    let all_healthy = db_healthy && redis_healthy;

    Json(json!({
        "status": if all_healthy { "healthy" } else { "degraded" },
        "version": env!("CARGO_PKG_VERSION"),
        "services": {
            "database": if db_healthy { "up" } else { "down" },
            "redis": if redis_healthy { "up" } else { "down" },
        }
    }))
}
