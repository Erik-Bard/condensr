pub struct Config {
    pub database_url: String,
    pub base_url: String,
    pub app_port: u16,
}

impl Config {
    pub fn new() -> Result<Self, anyhow::Error> {
        let database_url = std::env::var("DATABASE_URL").map_err(|_| {
            anyhow::anyhow!("DATABASE_URL must be set (see .env.example)")
        })?;
        let base_url = std::env::var("BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".into());
        let app_port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        Ok(Self {
            database_url,
            base_url,
            app_port,
        })
    }
}
