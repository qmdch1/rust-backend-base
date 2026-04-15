use axum::Json;
use serde_json::{json, Value};

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/api/v1/hello",
    tag = "Health",
    responses(
        (status = 200, description = "Hello World", body = Value)
    )
))]
pub async fn hello_world() -> Json<Value> {
    Json(json!({ "message": "Hello, World!" }))
}
