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

        // Clean up environment variables
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
        env::remove_var("SERVER_HOST");
    }

    #[test]
    fn test_server_url() {
        let config = Config {
            database_url: "dummy_db_url".to_string(),
            server_port: 1234,
            server_host: "testhost".to_string(),
        };
        assert_eq!(config.server_url(), "http://testhost:1234");
    }

    #[test]
    #[should_panic(expected = "DATABASE_URL must be set")]
    fn test_config_from_env_missing_database_url_panics() {
        // Ensure DATABASE_URL is not set
        env::remove_var("DATABASE_URL");
        // Clear other vars that might have defaults or cause other panics first
        env::remove_var("SERVER_PORT");
        env::remove_var("SERVER_HOST");

        Config::from_env(); // This should panic
    }

    #[test]
    #[should_panic(expected = "SERVER_PORT must be a number")]
    fn test_config_from_env_invalid_server_port_panics() {
        // Set required DATABASE_URL to avoid panicking on that first
        env::set_var("DATABASE_URL", "postgres://test_for_port_panic");
        // Set an invalid SERVER_PORT
        env::set_var("SERVER_PORT", "not_a_port");
        // Ensure SERVER_HOST is benign or use its default
        env::remove_var("SERVER_HOST");

        Config::from_env(); // This should panic

        // Clean up env vars used in this test
        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_PORT");
    }
}
