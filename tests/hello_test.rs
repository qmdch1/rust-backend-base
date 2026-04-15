use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use serde_json::Value;
use tower::ServiceExt;

use rust_backend_base::routes::handlers::hello_handler;

fn app() -> Router {
    Router::new().route("/api/v1/hello", get(hello_handler::hello_world))
}

#[tokio::test]
async fn test_hello_world() {
    let app = app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/hello")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["message"], "Hello, World!");
}
