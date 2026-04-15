use std::net::SocketAddr;

use rust_backend_base::{config::Config, db, middleware, routes, AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize tracing
    let env_filter = if config.server.environment.is_production() {
        "rust_backend_base=info,tower_http=info"
    } else {
        "rust_backend_base=debug,tower_http=debug"
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| env_filter.into()),
        )
        .with_target(true)
        .init();

    tracing::info!(
        "Starting server in {:?} mode",
        config.server.environment
    );

    // Initialize database
    let db_pool = db::postgres::init_pool(&config.database).await?;
    db::postgres::run_migrations(&db_pool).await?;

    // Initialize Redis
    let redis_pool = db::redis::init_pool(&config.redis).await?;

    // Build application state
    let state = AppState {
        db: db_pool,
        redis: redis_pool,
        config: config.clone(),
    };

    // Build router with middleware
    let app = routes::create_router(state)
        .layer(middleware::compression_layer())
        .layer(middleware::trace_layer())
        .layer(middleware::cors_layer(&config.cors))
        .layer(middleware::body_limit_layer())
        .layer(axum::Extension(config.jwt.clone()));

    // Start server
    let addr = SocketAddr::new(
        config.server.host.parse()?,
        config.server.port,
    );
    tracing::info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received Ctrl+C, starting graceful shutdown"),
        _ = terminate => tracing::info!("Received SIGTERM, starting graceful shutdown"),
    }
}
