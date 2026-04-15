pub mod auth;

use std::time::Duration;

use axum::http::{header, HeaderValue, Method};
use tower_http::cors::CorsLayer;
use tower_http::classify::{SharedClassifier, ServerErrorsAsFailures};
use tower_http::trace::{self, TraceLayer};
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tracing::Level;

use crate::config::CorsConfig;

pub fn cors_layer(config: &CorsConfig) -> CorsLayer {
    let origins: Vec<HeaderValue> = config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

pub fn trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
}

pub fn compression_layer() -> CompressionLayer {
    CompressionLayer::new()
}

pub fn body_limit_layer() -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(10 * 1024 * 1024) // 10MB
}
