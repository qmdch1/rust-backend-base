use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub cors: CorsConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: Environment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn is_production(&self) -> bool {
        *self == Environment::Production
    }
}

impl From<&str> for Environment {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stag" => Environment::Staging,
            _ => Environment::Development,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_secs: i64,
    pub refresh_token_expiry_secs: i64,
}

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        let environment = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
        let env = Environment::from(environment.as_str());

        // Production requires explicit configuration for critical values
        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(secret) => secret,
            Err(_) if env.is_production() => {
                anyhow::bail!("JWT_SECRET must be set in production")
            }
            Err(_) => "dev-only-secret-do-not-use-in-production".to_string(),
        };

        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) if env.is_production() => {
                anyhow::bail!("DATABASE_URL must be set in production")
            }
            Err(_) => "postgres://postgres:postgres@localhost:5432/rust_backend".to_string(),
        };

        Ok(Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
                environment: env,
            },
            database: DatabaseConfig {
                url: database_url,
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()?,
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            },
            jwt: JwtConfig {
                secret: jwt_secret,
                access_token_expiry_secs: env::var("JWT_ACCESS_TOKEN_EXPIRY_SECS")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()?,
                refresh_token_expiry_secs: env::var("JWT_REFRESH_TOKEN_EXPIRY_SECS")
                    .unwrap_or_else(|_| "604800".to_string())
                    .parse()?,
            },
            cors: CorsConfig {
                allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
        })
    }
}
