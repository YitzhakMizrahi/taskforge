use std::env;

pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub server_host: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT must be a number"),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        }
    }

    pub fn server_url(&self) -> String {
        format!("http://{}:{}", self.server_host, self.server_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        // Set required environment variables
        env::set_var("DATABASE_URL", "postgres://test");

        let config = Config::from_env();

        assert_eq!(config.database_url, "postgres://test");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.server_host, "127.0.0.1");

        // Test custom values
        env::set_var("SERVER_PORT", "3000");
        env::set_var("SERVER_HOST", "0.0.0.0");

        let config = Config::from_env();

        assert_eq!(config.server_port, 3000);
        assert_eq!(config.server_host, "0.0.0.0");
    }
}
