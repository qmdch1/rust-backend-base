use axum::Json;
use serde_json::{json, Value};

pub async fn hello_world() -> Json<Value> {
    Json(json!({ "message": "Hello, World!" }))
}
