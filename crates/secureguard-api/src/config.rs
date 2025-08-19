use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:password@localhost:5432/secureguard".to_string()
            }),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid PORT: {}", e))?,
        })
    }
}
