use std::env;

/// Application configuration settings.
///
/// These settings are typically loaded from environment variables.
pub struct Config {
    /// The full connection URL for the PostgreSQL database.
    /// Example: "postgres://user:password@host:port/database"
    pub database_url: String,
    /// The port on which the HTTP server will listen.
    /// Defaults to 8080 if `SERVER_PORT` env var is not set or invalid.
    pub server_port: u16,
    /// The host address the HTTP server will bind to.
    /// Defaults to "127.0.0.1" if `SERVER_HOST` env var is not set.
    pub server_host: String,
}

impl Config {
    /// Creates a `Config` instance by reading values from environment variables.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The `DATABASE_URL` environment variable is not set.
    /// - The `SERVER_PORT` environment variable is set but cannot be parsed as a u16 number.
    ///
    /// # Environment Variables
    ///
    /// - `DATABASE_URL`: (Required) The full PostgreSQL connection URL.
    /// - `SERVER_PORT`: (Optional) The port for the server. Defaults to "8080".
    /// - `SERVER_HOST`: (Optional) The host for the server. Defaults to "127.0.0.1".
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

    /// Constructs the full server URL (e.g., "http://127.0.0.1:8080").
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
    fn test_config_from_env_missing_database_url_panics() {
        // Store original values to restore them, ensuring other tests are not affected.
        let original_db_url = env::var("DATABASE_URL").ok();
        let original_server_port = env::var("SERVER_PORT").ok();
        let original_server_host = env::var("SERVER_HOST").ok();

        env::remove_var("DATABASE_URL"); // This is the variable we expect to cause the panic
        env::set_var("SERVER_PORT", "8080"); // Set to a known valid default
        env::set_var("SERVER_HOST", "127.0.0.1"); // Set to a known valid default
        
        let result = std::panic::catch_unwind(|| {
            Config::from_env();
        });

        // Restore original environment variables regardless of panic outcome
        if let Some(val) = original_db_url {
            env::set_var("DATABASE_URL", val);
        } else {
            env::remove_var("DATABASE_URL");
        }
        if let Some(val) = original_server_port {
            env::set_var("SERVER_PORT", val);
        } else {
            env::remove_var("SERVER_PORT");
        }
        if let Some(val) = original_server_host {
            env::set_var("SERVER_HOST", val);
        } else {
            env::remove_var("SERVER_HOST");
        }

        assert!(result.is_err(), "Config::from_env should have panicked when DATABASE_URL is missing.");
        
        // Check the panic message
        let panic_payload_err = result.err().expect("Test did not panic as expected, or panic was already handled.");
        if let Some(panic_msg_string) = panic_payload_err.downcast_ref::<String>() {
            assert!(panic_msg_string.contains("DATABASE_URL must be set"), 
                    "Panic message did not contain expected text. Got: {}", panic_msg_string);
        } else if let Some(panic_msg_str) = panic_payload_err.downcast_ref::<&str>() {
            assert!(panic_msg_str.contains("DATABASE_URL must be set"), 
                    "Panic message did not contain expected text. Got: {}", panic_msg_str);
        } else {
            panic!("Panic payload was not a String or &str. Actual payload: {:?}", panic_payload_err);
        }
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
