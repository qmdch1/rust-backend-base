pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;

use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: db::RedisPool,
    pub config: Config,
}
