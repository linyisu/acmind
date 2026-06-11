use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub api_port: u16,
    pub jwt_secret: String,
    pub jwt_expires_in: i64,
    pub allow_register: bool,
    pub rate_limit_per_second: u64,
    pub rate_limit_burst: u32,
    pub llm_provider: String,
    pub llm_api_key: String,
    pub llm_base_url: String,
    pub llm_model: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Loads .env if present, then reads from environment.
        let _ = dotenvy::dotenv();
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;
        let jwt_secret =
            std::env::var("JWT_SECRET").map_err(|_| anyhow::anyhow!("JWT_SECRET is required"))?;
        let api_port = std::env::var("API_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080u16);
        let jwt_expires_in = std::env::var("JWT_EXPIRES_IN")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(86_400i64);
        let allow_register = std::env::var("ALLOW_REGISTER")
            .ok()
            .map(|s| !matches!(s.as_str(), "false" | "0" | "no"))
            .unwrap_or(true);
        let rate_limit_per_second = std::env::var("RATE_LIMIT_PER_SECOND")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10u64);
        let rate_limit_burst = std::env::var("RATE_LIMIT_BURST")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(20u32);
        let llm_provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "noop".to_string());
        let llm_api_key = std::env::var("LLM_API_KEY").unwrap_or_default();
        let llm_base_url = std::env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let llm_model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        Ok(Config {
            database_url,
            api_port,
            jwt_secret,
            jwt_expires_in,
            allow_register,
            rate_limit_per_second,
            rate_limit_burst,
            llm_provider,
            llm_api_key,
            llm_base_url,
            llm_model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_minimal() {
        std::env::set_var("DATABASE_URL", "postgres://localhost/x");
        std::env::set_var("JWT_SECRET", "secret");
        std::env::set_var("API_PORT", "9000");
        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.api_port, 9000);
        assert_eq!(cfg.database_url, "postgres://localhost/x");
    }
}
