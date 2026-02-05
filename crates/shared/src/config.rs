use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub api_host: String,
    pub api_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://forecise:forecise@localhost:5432/forecise".into()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            api_host: std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            api_port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()?,
        })
    }
}
