use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::DatabaseConfig;

pub type PostgresPool = PgPool;

pub async fn init_pool(config: &DatabaseConfig) -> Result<PostgresPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect(&config.url)
        .await?;

    tracing::info!("PostgreSQL connection pool established");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("Database migrations completed");
    Ok(())
}
